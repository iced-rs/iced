//! Record, edit, and run end-to-end tests for your iced applications.
pub use iced_test as test;
pub use iced_test::core;
pub use iced_test::program;
pub use iced_test::runtime;
pub use iced_test::runtime::futures;
pub use iced_widget as widget;

mod icon;
mod recorder;

use recorder::recorder;

use crate::core::Alignment::Center;
use crate::core::Length::Fill;
use crate::core::alignment::Horizontal::Right;
use crate::core::border;
use crate::core::mouse;
use crate::core::theme;
use crate::core::window;
use crate::core::{Color, Element, Font, Settings, Size, Theme};
use crate::futures::futures::channel::mpsc;
use crate::program::Program;
use crate::runtime::task::{self, Task};
use crate::test::emulator;
use crate::test::ice;
use crate::test::instruction;
use crate::test::{Emulator, Ice, Instruction};
use crate::widget::{
    button, center, column, combo_box, container, pick_list, row, rule,
    scrollable, slider, space, stack, text, text_editor, themer,
};

use std::ops::RangeInclusive;

/// Attaches a [`Tester`] to the given [`Program`].
pub fn attach<P: Program + 'static>(program: P) -> Attach<P> {
    Attach { program }
}

/// A [`Program`] with a [`Tester`] attached to it.
#[derive(Debug)]
pub struct Attach<P> {
    /// The original [`Program`] attached to the [`Tester`].
    pub program: P,
}

impl<P> Program for Attach<P>
where
    P: Program + 'static,
{
    type State = Tester<P>;
    type Message = Message<P>;
    type Theme = Theme;
    type Renderer = P::Renderer;
    type Executor = P::Executor;

    fn name() -> &'static str {
        P::name()
    }

    fn settings(&self) -> Settings {
        let mut settings = self.program.settings();
        settings.fonts.push(icon::FONT.into());
        settings
    }

    fn window(&self) -> Option<window::Settings> {
        Some(
            self.program
                .window()
                .map(|window| window::Settings {
                    size: window.size + Size::new(300.0, 80.0),
                    ..window
                })
                .unwrap_or_default(),
        )
    }

    fn boot(&self) -> (Self::State, Task<Self::Message>) {
        (Tester::new(&self.program), Task::none())
    }

    fn update(
        &self,
        state: &mut Self::State,
        message: Self::Message,
    ) -> Task<Self::Message> {
        state.tick(&self.program, message.0).map(Message)
    }

    fn view<'a>(
        &self,
        state: &'a Self::State,
        window: window::Id,
    ) -> Element<'a, Self::Message, Self::Theme, Self::Renderer> {
        state.view(&self.program, window).map(Message)
    }

    fn theme(&self, state: &Self::State, window: window::Id) -> Option<Theme> {
        state
            .theme(&self.program, window)
            .as_ref()
            .and_then(theme::Base::palette)
            .map(|palette| Theme::custom("Tester", palette))
    }
}

/// A tester decorates a [`Program`] definition and attaches a test recorder on top.
///
/// It can be used to both record and play [`Ice`] tests.
pub struct Tester<P: Program> {
    viewport: Size,
    mode: emulator::Mode,
    presets: combo_box::State<String>,
    preset: Option<String>,
    instructions: Vec<Instruction>,
    state: State<P>,
    edit: Option<text_editor::Content<P::Renderer>>,
}

enum State<P: Program> {
    Empty,
    Idle {
        state: P::State,
    },
    Recording {
        emulator: Emulator<P>,
    },
    Asserting {
        state: P::State,
        window: window::Id,
        last_interaction: Option<instruction::Interaction>,
    },
    Playing {
        emulator: Emulator<P>,
        current: usize,
        outcome: Outcome,
    },
}

enum Outcome {
    Running,
    Failed,
    Success,
}

/// The message of a [`Tester`].
pub struct Message<P: Program>(Tick<P>);

#[derive(Debug, Clone)]
enum Event {
    ViewportChanged(Size),
    ModeSelected(emulator::Mode),
    PresetSelected(String),
    Record,
    Stop,
    Play,
    Import,
    Export,
    Imported(Result<Ice, ice::ParseError>),
    Edit,
    Edited(text_editor::Action),
    Confirm,
}

enum Tick<P: Program> {
    Tester(Event),
    Program(P::Message),
    Emulator(emulator::Event<P>),
    Record(instruction::Interaction),
    Assert(instruction::Interaction),
}

impl<P: Program + 'static> Tester<P> {
    fn new(program: &P) -> Self {
        let (state, _) = program.boot();
        let window = program.window().unwrap_or_default();

        Self {
            mode: emulator::Mode::default(),
            viewport: window.size,
            presets: combo_box::State::new(
                program
                    .presets()
                    .iter()
                    .map(program::Preset::name)
                    .map(str::to_owned)
                    .collect(),
            ),
            preset: None,
            instructions: Vec::new(),
            state: State::Idle { state },
            edit: None,
        }
    }

    fn is_busy(&self) -> bool {
        matches!(
            self.state,
            State::Recording { .. }
                | State::Playing {
                    outcome: Outcome::Running,
                    ..
                }
        )
    }

    fn update(&mut self, program: &P, event: Event) -> Task<Tick<P>> {
        match event {
            Event::ViewportChanged(viewport) => {
                self.viewport = viewport;

                Task::none()
            }
            Event::ModeSelected(mode) => {
                self.mode = mode;

                Task::none()
            }
            Event::PresetSelected(preset) => {
                self.preset = Some(preset);

                let (state, _) = self
                    .preset(program)
                    .map(program::Preset::boot)
                    .unwrap_or_else(|| program.boot());

                self.state = State::Idle { state };

                Task::none()
            }
            Event::Record => {
                self.edit = None;
                self.instructions.clear();

                let (sender, receiver) = mpsc::channel(1);

                let emulator = Emulator::with_preset(
                    sender,
                    program,
                    self.mode,
                    self.viewport,
                    self.preset(program),
                );

                self.state = State::Recording { emulator };

                Task::run(receiver, Tick::Emulator)
            }
            Event::Stop => {
                let State::Recording { emulator } =
                    std::mem::replace(&mut self.state, State::Empty)
                else {
                    return Task::none();
                };

                while let Some(Instruction::Interact(
                    instruction::Interaction::Mouse(instruction::Mouse::Move(
                        _,
                    )),
                )) = self.instructions.last()
                {
                    let _ = self.instructions.pop();
                }

                let (state, window) = emulator.into_state();

                self.state = State::Asserting {
                    state,
                    window,
                    last_interaction: None,
                };

                Task::none()
            }
            Event::Play => {
                self.confirm();

                let (sender, receiver) = mpsc::channel(1);

                let emulator = Emulator::with_preset(
                    sender,
                    program,
                    self.mode,
                    self.viewport,
                    self.preset(program),
                );

                self.state = State::Playing {
                    emulator,
                    current: 0,
                    outcome: Outcome::Running,
                };

                Task::run(receiver, Tick::Emulator)
            }
            Event::Import => {
                use std::fs;

                let import = rfd::AsyncFileDialog::new()
                    .add_filter("ice", &["ice"])
                    .pick_file();

                Task::future(import)
                    .and_then(|file| {
                        task::blocking(move |mut sender| {
                            let _ = sender.try_send(Ice::parse(
                                &fs::read_to_string(file.path())
                                    .unwrap_or_default(),
                            ));
                        })
                    })
                    .map(Event::Imported)
                    .map(Tick::Tester)
            }
            Event::Export => {
                use std::fs;
                use std::thread;

                self.confirm();

                let ice = Ice {
                    viewport: self.viewport,
                    mode: self.mode,
                    preset: self.preset.clone(),
                    instructions: self.instructions.clone(),
                };

                let export = rfd::AsyncFileDialog::new()
                    .add_filter("ice", &["ice"])
                    .save_file();

                Task::future(async move {
                    let Some(file) = export.await else {
                        return;
                    };

                    let _ = thread::spawn(move || {
                        fs::write(file.path(), ice.to_string())
                    });
                })
                .discard()
            }
            Event::Imported(Ok(ice)) => {
                self.viewport = ice.viewport;
                self.mode = ice.mode;
                self.preset = ice.preset;
                self.instructions = ice.instructions;
                self.edit = None;

                let (state, _) = self
                    .preset(program)
                    .map(program::Preset::boot)
                    .unwrap_or_else(|| program.boot());

                self.state = State::Idle { state };

                Task::none()
            }
            Event::Edit => {
                if self.is_busy() {
                    return Task::none();
                }

                self.edit = Some(text_editor::Content::with_text(
                    &self
                        .instructions
                        .iter()
                        .map(Instruction::to_string)
                        .collect::<Vec<_>>()
                        .join("\n"),
                ));

                Task::none()
            }
            Event::Edited(action) => {
                if let Some(edit) = &mut self.edit {
                    edit.perform(action);
                }

                Task::none()
            }
            Event::Confirm => {
                self.confirm();

                Task::none()
            }
            Event::Imported(Err(error)) => {
                log::error!("{error}");

                Task::none()
            }
        }
    }

    fn confirm(&mut self) {
        let Some(edit) = &mut self.edit else {
            return;
        };

        self.instructions = edit
            .lines()
            .filter(|line| !line.text.trim().is_empty())
            .filter_map(|line| Instruction::parse(&line.text).ok())
            .collect();

        self.edit = None;
    }

    fn theme(&self, program: &P, window: window::Id) -> Option<P::Theme> {
        match &self.state {
            State::Empty => None,
            State::Idle { state } => program.theme(state, window),
            State::Recording { emulator } | State::Playing { emulator, .. } => {
                emulator.theme(program)
            }
            State::Asserting { state, window, .. } => {
                program.theme(state, *window)
            }
        }
    }

    fn preset<'a>(
        &self,
        program: &'a P,
    ) -> Option<&'a program::Preset<P::State, P::Message>> {
        self.preset.as_ref().and_then(|preset| {
            program
                .presets()
                .iter()
                .find(|candidate| candidate.name() == preset)
        })
    }

    fn tick(&mut self, program: &P, tick: Tick<P>) -> Task<Tick<P>> {
        match tick {
            Tick::Tester(message) => self.update(program, message),
            Tick::Program(message) => {
                let State::Recording { emulator } = &mut self.state else {
                    return Task::none();
                };

                emulator.update(program, message);

                Task::none()
            }
            Tick::Emulator(event) => {
                match &mut self.state {
                    State::Recording { emulator } => {
                        if let emulator::Event::Action(action) = event {
                            emulator.perform(program, action);
                        }
                    }
                    State::Playing {
                        emulator,
                        current,
                        outcome,
                    } => match event {
                        emulator::Event::Action(action) => {
                            emulator.perform(program, action);
                        }
                        emulator::Event::Failed(_instruction) => {
                            *outcome = Outcome::Failed;
                        }
                        emulator::Event::Ready => {
                            *current += 1;

                            if let Some(instruction) =
                                self.instructions.get(*current - 1).cloned()
                            {
                                emulator.run(program, instruction);
                            }

                            if *current >= self.instructions.len() {
                                *outcome = Outcome::Success;
                            }
                        }
                    },
                    State::Empty
                    | State::Idle { .. }
                    | State::Asserting { .. } => {}
                }

                Task::none()
            }
            Tick::Record(interaction) => {
                let mut interaction = Some(interaction);

                while let Some(new_interaction) = interaction.take() {
                    if let Some(Instruction::Interact(last_interaction)) =
                        self.instructions.pop()
                    {
                        let (merged_interaction, new_interaction) =
                            last_interaction.merge(new_interaction);

                        if let Some(new_interaction) = new_interaction {
                            self.instructions.push(Instruction::Interact(
                                merged_interaction,
                            ));

                            self.instructions
                                .push(Instruction::Interact(new_interaction));
                        } else {
                            interaction = Some(merged_interaction);
                        }
                    } else {
                        self.instructions
                            .push(Instruction::Interact(new_interaction));
                    }
                }

                Task::none()
            }
            Tick::Assert(interaction) => {
                let State::Asserting {
                    last_interaction, ..
                } = &mut self.state
                else {
                    return Task::none();
                };

                *last_interaction =
                    if let Some(last_interaction) = last_interaction.take() {
                        let (merged, new) = last_interaction.merge(interaction);

                        Some(new.unwrap_or(merged))
                    } else {
                        Some(interaction)
                    };

                let Some(interaction) = last_interaction.take() else {
                    return Task::none();
                };

                let instruction::Interaction::Mouse(
                    instruction::Mouse::Click {
                        button: mouse::Button::Left,
                        target: Some(instruction::Target::Text(text)),
                    },
                ) = interaction
                else {
                    *last_interaction = Some(interaction);
                    return Task::none();
                };

                self.instructions.push(Instruction::Expect(
                    instruction::Expectation::Text(text),
                ));

                Task::none()
            }
        }
    }

    fn view<'a>(
        &'a self,
        program: &P,
        window: window::Id,
    ) -> Element<'a, Tick<P>, Theme, P::Renderer> {
        let status = {
            let (icon, label) = match &self.state {
                State::Empty | State::Idle { .. } => (text(""), "Idle"),
                State::Recording { .. } => (icon::record(), "Recording"),
                State::Asserting { .. } => (icon::lightbulb(), "Asserting"),
                State::Playing { outcome, .. } => match outcome {
                    Outcome::Running => (icon::play(), "Playing"),
                    Outcome::Failed => (icon::cancel(), "Failed"),
                    Outcome::Success => (icon::check(), "Success"),
                },
            };

            container(row![icon.size(14), label].align_y(Center).spacing(8))
                .style(|theme: &Theme| {
                    let palette = theme.extended_palette();

                    container::Style {
                        text_color: Some(match &self.state {
                            State::Empty | State::Idle { .. } => {
                                palette.background.strongest.color
                            }
                            State::Recording { .. } => {
                                palette.danger.base.color
                            }
                            State::Asserting { .. } => {
                                palette.warning.base.color
                            }
                            State::Playing { outcome, .. } => match outcome {
                                Outcome::Running => theme.palette().primary,
                                Outcome::Failed => theme.palette().danger,
                                Outcome::Success => {
                                    theme
                                        .extended_palette()
                                        .success
                                        .strong
                                        .color
                                }
                            },
                        }),
                        ..container::Style::default()
                    }
                })
        };

        let view = match &self.state {
            State::Empty => Element::from(space()),
            State::Idle { state } => {
                program.view(state, window).map(Tick::Program)
            }
            State::Recording { emulator } => {
                recorder(emulator.view(program).map(Tick::Program))
                    .on_record(Tick::Record)
                    .into()
            }
            State::Asserting { state, window, .. } => {
                recorder(program.view(state, *window).map(Tick::Program))
                    .on_record(Tick::Assert)
                    .into()
            }
            State::Playing { emulator, .. } => {
                emulator.view(program).map(Tick::Program)
            }
        };

        let viewport = container(
            scrollable(
                container(themer(self.theme(program, window), view))
                    .width(self.viewport.width)
                    .height(self.viewport.height),
            )
            .direction(scrollable::Direction::Both {
                vertical: scrollable::Scrollbar::default(),
                horizontal: scrollable::Scrollbar::default(),
            }),
        )
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();

            container::Style {
                border: border::width(2.0).color(match &self.state {
                    State::Empty | State::Idle { .. } => {
                        palette.background.strongest.color
                    }
                    State::Recording { .. } => palette.danger.base.color,
                    State::Asserting { .. } => palette.warning.weak.color,
                    State::Playing { outcome, .. } => match outcome {
                        Outcome::Running => palette.primary.base.color,
                        Outcome::Failed => palette.danger.strong.color,
                        Outcome::Success => palette.success.strong.color,
                    },
                }),
                ..container::Style::default()
            }
        })
        .padding(10);

        row![
            center(column![status, viewport].spacing(10).align_x(Right))
                .padding(10),
            rule::vertical(1).style(rule::weak),
            container(self.controls().map(Tick::Tester))
                .width(250)
                .padding(10)
                .style(|theme| container::Style::default().background(
                    theme.extended_palette().background.weakest.color
                )),
        ]
        .into()
    }

    fn controls(&self) -> Element<'_, Event, Theme, P::Renderer> {
        let viewport = column![
            labeled_slider(
                "Width",
                100.0..=2000.0,
                self.viewport.width,
                |width| Event::ViewportChanged(Size {
                    width,
                    ..self.viewport
                }),
                |width| format!("{width:.0}"),
            ),
            labeled_slider(
                "Height",
                100.0..=2000.0,
                self.viewport.height,
                |height| Event::ViewportChanged(Size {
                    height,
                    ..self.viewport
                }),
                |height| format!("{height:.0}"),
            ),
        ]
        .spacing(10);

        let preset = combo_box(
            &self.presets,
            "Default",
            self.preset.as_ref(),
            Event::PresetSelected,
        )
        .size(14)
        .width(Fill);

        let mode = pick_list(
            emulator::Mode::ALL,
            Some(self.mode),
            Event::ModeSelected,
        )
        .text_size(14)
        .width(Fill);

        let player = {
            let instructions = if let Some(edit) = &self.edit {
                text_editor(edit)
                    .size(12)
                    .height(Fill)
                    .font(Font::MONOSPACE)
                    .on_action(Event::Edited)
                    .into()
            } else if self.instructions.is_empty() {
                Element::from(center(
                    text("No instructions recorded yet!")
                        .size(14)
                        .font(Font::MONOSPACE)
                        .width(Fill)
                        .center(),
                ))
            } else {
                scrollable(
                    column(self.instructions.iter().enumerate().map(
                        |(i, instruction)| {
                            text(instruction.to_string())
                                .wrapping(text::Wrapping::None) // TODO: Ellipsize?
                                .size(10)
                                .font(Font::MONOSPACE)
                                .style(move |theme: &Theme| text::Style {
                                    color: match &self.state {
                                        State::Playing {
                                            current,
                                            outcome,
                                            ..
                                        } => {
                                            if *current == i + 1 {
                                                Some(match outcome {
                                                    Outcome::Running => {
                                                        theme.palette().primary
                                                    }
                                                    Outcome::Failed => {
                                                        theme
                                                            .extended_palette()
                                                            .danger
                                                            .strong
                                                            .color
                                                    }
                                                    Outcome::Success => {
                                                        theme
                                                            .extended_palette()
                                                            .success
                                                            .strong
                                                            .color
                                                    }
                                                })
                                            } else if *current > i + 1 {
                                                Some(
                                                    theme
                                                        .extended_palette()
                                                        .success
                                                        .strong
                                                        .color,
                                                )
                                            } else {
                                                None
                                            }
                                        }
                                        _ => None,
                                    },
                                })
                                .into()
                        },
                    ))
                    .spacing(5),
                )
                .width(Fill)
                .height(Fill)
                .spacing(5)
                .into()
            };

            let control = |icon: text::Text<'static, _, _>| {
                button(icon.size(14).width(Fill).height(Fill).center())
            };

            let play = control(icon::play()).on_press_maybe(
                (!matches!(self.state, State::Recording { .. })
                    && !self.instructions.is_empty())
                .then_some(Event::Play),
            );

            let record = if let State::Recording { .. } = &self.state {
                control(icon::stop())
                    .on_press(Event::Stop)
                    .style(button::success)
            } else {
                control(icon::record())
                    .on_press_maybe((!self.is_busy()).then_some(Event::Record))
                    .style(button::danger)
            };

            let import = control(icon::folder())
                .on_press_maybe((!self.is_busy()).then_some(Event::Import))
                .style(button::secondary);

            let export = control(icon::floppy())
                .on_press_maybe(
                    (!matches!(self.state, State::Recording { .. })
                        && !self.instructions.is_empty())
                    .then_some(Event::Export),
                )
                .style(button::success);

            let controls =
                row![import, export, play, record].height(30).spacing(10);

            column![instructions, controls].spacing(10).align_x(Center)
        };

        let edit = if self.is_busy() {
            Element::from(space::horizontal())
        } else if self.edit.is_none() {
            button(icon::pencil().size(14))
                .padding(0)
                .on_press(Event::Edit)
                .style(button::text)
                .into()
        } else {
            button(icon::check().size(14))
                .padding(0)
                .on_press(Event::Confirm)
                .style(button::text)
                .into()
        };

        column![
            labeled("Viewport", viewport),
            labeled("Mode", mode),
            labeled("Preset", preset),
            labeled_with("Instructions", edit, player)
        ]
        .spacing(10)
        .into()
    }
}

fn labeled<'a, Message, Renderer>(
    fragment: impl text::IntoFragment<'a>,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: program::Renderer + 'a,
{
    column![
        text(fragment).size(14).font(Font::MONOSPACE),
        content.into()
    ]
    .spacing(5)
    .into()
}

fn labeled_with<'a, Message, Renderer>(
    fragment: impl text::IntoFragment<'a>,
    control: impl Into<Element<'a, Message, Theme, Renderer>>,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: program::Renderer + 'a,
{
    column![
        row![
            text(fragment).size(14).font(Font::MONOSPACE),
            space::horizontal(),
            control.into()
        ]
        .spacing(5)
        .align_y(Center),
        content.into()
    ]
    .spacing(5)
    .into()
}

fn labeled_slider<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    range: RangeInclusive<f32>,
    current: f32,
    on_change: impl Fn(f32) -> Message + 'a,
    to_string: impl Fn(&f32) -> String,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: core::text::Renderer + 'a,
{
    stack![
        container(
            slider(range, current, on_change)
                .step(10.0)
                .width(Fill)
                .height(24)
                .style(|theme: &core::Theme, status| {
                    let palette = theme.extended_palette();

                    slider::Style {
                        rail: slider::Rail {
                            backgrounds: (
                                match status {
                                    slider::Status::Active
                                    | slider::Status::Dragged => {
                                        palette.background.strongest.color
                                    }
                                    slider::Status::Hovered => {
                                        palette.background.stronger.color
                                    }
                                }
                                .into(),
                                Color::TRANSPARENT.into(),
                            ),
                            width: 24.0,
                            border: border::rounded(2),
                        },
                        handle: slider::Handle {
                            shape: slider::HandleShape::Circle { radius: 0.0 },
                            background: Color::TRANSPARENT.into(),
                            border_width: 0.0,
                            border_color: Color::TRANSPARENT,
                        },
                    }
                })
        )
        .style(|theme| container::Style::default()
            .background(theme.extended_palette().background.weak.color)
            .border(border::rounded(2))),
        row![
            text(label).size(14).style(|theme: &core::Theme| {
                text::Style {
                    color: Some(theme.extended_palette().background.weak.text),
                }
            }),
            space::horizontal(),
            text(to_string(&current)).size(14)
        ]
        .padding([0, 10])
        .height(Fill)
        .align_y(Center),
    ]
    .into()
}
