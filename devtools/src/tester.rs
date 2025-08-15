mod recorder;

use recorder::recorder;

use crate::Program;
use crate::core::Alignment::Center;
use crate::core::Length::Fill;
use crate::core::alignment::Horizontal::Right;
use crate::core::border;
use crate::core::window;
use crate::core::{Element, Event, Font, Size, Theme};
use crate::executor;
use crate::futures::futures::channel::mpsc;
use crate::icon;
use crate::program;
use crate::runtime::Task;
use crate::test::emulator;
use crate::test::ice;
use crate::test::instruction;
use crate::test::{Emulator, Ice, Instruction};
use crate::widget::{
    button, center, column, combo_box, container, horizontal_space, monospace,
    pick_list, row, scrollable, text, text_editor, text_input, themer,
};

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
    Idle,
    Recording {
        emulator: Emulator<P>,
    },
    Ready {
        state: P::State,
        window: window::Id,
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

#[derive(Debug, Clone)]
pub enum Message {
    ChangeViewport(Size),
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

#[allow(missing_debug_implementations)]
pub enum Tick<P: Program> {
    Tester(Message),
    Program(P::Message),
    Recorder(Event),
    Emulator(emulator::Event<P>),
}

impl<P: Program + 'static> Tester<P> {
    pub fn new(program: &P) -> Self {
        Self {
            mode: emulator::Mode::default(),
            viewport: Size::new(512.0, 512.0),
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
            state: State::Idle,
            edit: None,
        }
    }

    pub fn is_busy(&self) -> bool {
        matches!(
            self.state,
            State::Recording { .. }
                | State::Playing {
                    outcome: Outcome::Running,
                    ..
                }
        )
    }

    pub fn update(&mut self, program: &P, message: Message) -> Task<Tick<P>> {
        match message {
            Message::ChangeViewport(viewport) => {
                self.viewport = viewport;

                Task::none()
            }
            Message::ModeSelected(mode) => {
                self.mode = mode;

                Task::none()
            }
            Message::PresetSelected(preset) => {
                self.preset = Some(preset);

                Task::none()
            }
            Message::Record => {
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
            Message::Stop => {
                let State::Recording { emulator } =
                    std::mem::replace(&mut self.state, State::Idle)
                else {
                    return Task::none();
                };

                let (state, window) = emulator.into_state();

                self.state = State::Ready { state, window };

                Task::none()
            }
            Message::Play => {
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
            Message::Import => {
                use std::fs;

                let import = rfd::AsyncFileDialog::new()
                    .add_filter("ice", &["ice"])
                    .pick_file();

                Task::future(import)
                    .and_then(|file| {
                        executor::spawn_blocking(move |mut sender| {
                            let _ = sender.try_send(Ice::parse(
                                &fs::read_to_string(file.path())
                                    .unwrap_or_default(),
                            ));
                        })
                    })
                    .map(Message::Imported)
                    .map(Tick::Tester)
            }
            Message::Export => {
                use std::fs;
                use std::thread;

                self.confirm();

                let ice = Ice {
                    viewport: Size::new(
                        self.viewport.width as u32,
                        self.viewport.height as u32,
                    ),
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
            Message::Imported(Ok(ice)) => {
                self.viewport = Size::new(
                    ice.viewport.width as f32,
                    ice.viewport.height as f32,
                );
                self.mode = ice.mode;
                self.preset = ice.preset;
                self.instructions = ice.instructions;
                self.edit = None;
                self.state = State::Idle;

                Task::none()
            }
            Message::Edit => {
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
            Message::Edited(action) => {
                if let Some(edit) = &mut self.edit {
                    edit.perform(action);
                }

                Task::none()
            }
            Message::Confirm => {
                self.confirm();

                Task::none()
            }
            Message::Imported(Err(error)) => {
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

    pub fn tick(&mut self, program: &P, tick: Tick<P>) -> Task<Tick<P>> {
        match tick {
            Tick::Tester(message) => self.update(program, message),
            Tick::Program(message) => {
                let State::Recording { emulator } = &mut self.state else {
                    return Task::none();
                };

                emulator.update(program, message);

                Task::none()
            }
            Tick::Recorder(event) => {
                let mut interaction =
                    instruction::Interaction::from_event(event);

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
                        emulator::Event::Failed => {
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
                    State::Idle | State::Ready { .. } => {}
                }

                Task::none()
            }
        }
    }

    pub fn view<'a, T: 'static>(
        &'a self,
        program: &P,
        current: impl FnOnce() -> Element<'a, T, Theme, P::Renderer>,
        emulate: impl Fn(Tick<P>) -> T + 'a,
    ) -> Element<'a, T, Theme, P::Renderer> {
        let status = {
            let (icon, label) = match &self.state {
                State::Idle => (text(""), "Idle"),
                State::Recording { .. } => (icon::record(), "Recording"),
                State::Ready { .. } => (icon::lightbulb(), "Ready"),
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
                            State::Idle => palette.background.strongest.color,
                            State::Recording { .. } => {
                                palette.danger.base.color
                            }
                            State::Ready { .. } => palette.warning.base.color,
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

        let viewport = container(
            scrollable(
                container(match &self.state {
                    State::Idle => current(),
                    State::Recording { emulator } => {
                        let theme = emulator.theme(program);
                        let view = emulator.view(program).map(Tick::Program);

                        Element::from(
                            recorder(themer(theme, view))
                                .on_event(Tick::Recorder),
                        )
                        .map(emulate)
                    }
                    State::Ready { state, window } => {
                        let theme = program.theme(state, *window);
                        let view =
                            program.view(state, *window).map(Tick::Program);

                        Element::from(themer(theme, view)).map(emulate)
                    }
                    State::Playing { emulator, .. } => {
                        let theme = emulator.theme(program);
                        let view = emulator.view(program).map(Tick::Program);

                        Element::from(themer(theme, view)).map(emulate)
                    }
                })
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
                    State::Idle => palette.background.strongest.color,
                    State::Recording { .. } => palette.danger.base.color,
                    State::Ready { .. } => palette.warning.weak.color,
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

        center(column![status, viewport].spacing(10).align_x(Right))
            .padding(10)
            .into()
    }

    pub fn controls(&self) -> Element<'_, Message, Theme, P::Renderer> {
        let viewport = row![
            text_input("Width", &self.viewport.width.to_string())
                .size(14)
                .on_input(|width| Message::ChangeViewport(Size {
                    width: width.parse().unwrap_or(self.viewport.width),
                    ..self.viewport
                })),
            monospace("x").size(14),
            text_input("Height", &self.viewport.height.to_string())
                .size(14)
                .on_input(|height| Message::ChangeViewport(Size {
                    height: height.parse().unwrap_or(self.viewport.height),
                    ..self.viewport
                })),
        ]
        .spacing(10)
        .align_y(Center);

        let preset = combo_box(
            &self.presets,
            "Default",
            self.preset.as_ref(),
            Message::PresetSelected,
        )
        .size(14)
        .width(Fill);

        let mode = pick_list(
            emulator::Mode::ALL,
            Some(self.mode),
            Message::ModeSelected,
        )
        .text_size(14)
        .width(Fill);

        let player = {
            let instructions = if let Some(edit) = &self.edit {
                text_editor(edit)
                    .size(12)
                    .height(Fill)
                    .font(Font::MONOSPACE)
                    .on_action(Message::Edited)
                    .into()
            } else if self.instructions.is_empty() {
                Element::from(center(
                    monospace("No instructions recorded yet!")
                        .size(14)
                        .width(Fill)
                        .center(),
                ))
            } else {
                scrollable(
                    column(self.instructions.iter().enumerate().map(
                        |(i, instruction)| {
                            monospace(instruction.to_string())
                                .size(10)
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
                .then_some(Message::Play),
            );

            let record = if let State::Recording { .. } = &self.state {
                control(icon::stop())
                    .on_press(Message::Stop)
                    .style(button::success)
            } else {
                control(icon::record())
                    .on_press_maybe(
                        (!self.is_busy()).then_some(Message::Record),
                    )
                    .style(button::danger)
            };

            let import = control(icon::folder())
                .on_press_maybe((!self.is_busy()).then_some(Message::Import))
                .style(button::secondary);

            let export = control(icon::floppy())
                .on_press_maybe(
                    (!matches!(self.state, State::Recording { .. })
                        && !self.instructions.is_empty())
                    .then_some(Message::Export),
                )
                .style(button::success);

            let controls =
                row![import, export, play, record].height(30).spacing(10);

            column![instructions, controls].spacing(10).align_x(Center)
        };

        let edit = if self.is_busy() {
            Element::from(horizontal_space())
        } else if self.edit.is_none() {
            button(icon::pencil().size(14))
                .padding(0)
                .on_press(Message::Edit)
                .style(button::text)
                .into()
        } else {
            button(icon::check().size(14))
                .padding(0)
                .on_press(Message::Confirm)
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
    column![monospace(fragment).size(14), content.into()]
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
            monospace(fragment).size(14),
            horizontal_space(),
            control.into()
        ]
        .spacing(5)
        .align_y(Center),
        content.into()
    ]
    .spacing(5)
    .into()
}
