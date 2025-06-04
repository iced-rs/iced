mod recorder;

use recorder::recorder;

use crate::Program;
use crate::core::Alignment::Center;
use crate::core::Length::Fill;
use crate::core::alignment::Horizontal::Right;
use crate::core::border;
use crate::core::window;
use crate::core::{Element, Event, Size, Theme};
use crate::futures::Subscription;
use crate::futures::futures::channel::mpsc;
use crate::icon;
use crate::program;
use crate::runtime::Task;
use crate::test::emulator;
use crate::test::instruction;
use crate::test::{Emulator, Instruction};
use crate::widget::{
    button, center, column, combo_box, container, monospace, pick_list, row,
    scrollable, text, text_input, themer,
};

pub struct Tester<P: Program> {
    viewport: Size,
    mode: emulator::Mode,
    presets: combo_box::State<String>,
    preset: Option<String>,
    instructions: Vec<Instruction>,
    state: State<P>,
}

enum State<P: Program> {
    Idle,
    Recording {
        state: P::State,
    },
    Ready {
        state: P::State,
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
}

#[allow(missing_debug_implementations)]
pub enum Tick<P: Program> {
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
        }
    }

    pub fn is_idle(&self) -> bool {
        matches!(self.state, State::Idle)
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
                self.instructions.clear();

                let (state, task) = if let Some(preset) = self.preset(program) {
                    preset.boot()
                } else {
                    program.boot()
                };

                self.state = State::Recording { state };

                task.map(Tick::Program)
            }
            Message::Stop => {
                let State::Recording { state } =
                    std::mem::replace(&mut self.state, State::Idle)
                else {
                    return Task::none();
                };

                self.state = State::Ready { state };

                Task::none()
            }
            Message::Play => {
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

    pub fn tick(&mut self, program: &P, tick: Tick<P>) -> Task<Tick<P>> {
        match tick {
            Tick::Program(message) => {
                let State::Recording { state } = &mut self.state else {
                    return Task::none();
                };

                program.update(state, message).map(Tick::Program)
            }
            Tick::Recorder(event) => {
                let Some(interaction) =
                    instruction::Interaction::from_event(event)
                else {
                    return Task::none();
                };

                if let Some(Instruction::Interact(last_interaction)) =
                    self.instructions.pop()
                {
                    let (last_interaction, new_interaction) =
                        last_interaction.merge(interaction);

                    self.instructions
                        .push(Instruction::Interact(last_interaction));

                    if let Some(new_interaction) = new_interaction {
                        self.instructions
                            .push(Instruction::Interact(new_interaction));
                    }
                } else {
                    self.instructions.push(Instruction::Interact(interaction));
                }

                Task::none()
            }
            Tick::Emulator(event) => {
                let State::Playing {
                    emulator,
                    current,
                    outcome,
                } = &mut self.state
                else {
                    return Task::none();
                };

                match event {
                    emulator::Event::Action(action) => {
                        emulator.perform(program, action);
                    }
                    emulator::Event::Failed => {
                        *outcome = Outcome::Failed;
                    }
                    emulator::Event::Ready => {
                        if let Some(instruction) =
                            self.instructions.get(*current).cloned()
                        {
                            emulator.run(program, instruction);
                            *current += 1;
                        }

                        if *current >= self.instructions.len() {
                            *outcome = Outcome::Success;
                        }
                    }
                }

                Task::none()
            }
        }
    }

    pub fn subscription(&self, program: &P) -> Subscription<Tick<P>> {
        match &self.state {
            State::Idle | State::Playing { .. } | State::Ready { .. } => {
                Subscription::none()
            }
            State::Recording { state } => {
                program.subscription(state).map(Tick::Program)
            }
        }
    }

    pub fn view<'a, T: 'static>(
        &'a self,
        program: &P,
        window: window::Id,
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
                    State::Recording { state } => {
                        let theme = program.theme(state, window);
                        let view =
                            program.view(state, window).map(Tick::Program);

                        Element::from(
                            recorder(themer(theme, view))
                                .on_event(Tick::Recorder),
                        )
                        .map(emulate)
                    }
                    State::Ready { state } => {
                        let theme = program.theme(state, window);
                        let view =
                            program.view(state, window).map(Tick::Program);

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
            let instructions = container(if self.instructions.is_empty() {
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
                                            if *current == i {
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
                                            } else if *current > i {
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
                .spacing(5)
                .into()
            })
            .width(Fill)
            .height(Fill)
            .padding(5);

            let controls = {
                row![
                    button(icon::play().size(14).width(Fill).center())
                        .on_press_maybe(
                            (!matches!(self.state, State::Recording { .. })
                                && !self.instructions.is_empty())
                            .then_some(Message::Play),
                        ),
                    if let State::Recording { .. } = &self.state {
                        button(icon::stop().size(14).width(Fill).center())
                            .on_press(Message::Stop)
                            .style(button::success)
                    } else {
                        button(icon::record().size(14).width(Fill).center())
                            .on_press_maybe(
                                (!self.is_busy()).then_some(Message::Record),
                            )
                            .style(button::danger)
                    }
                ]
                .spacing(10)
            };

            column![instructions, controls].spacing(10).align_x(Center)
        };

        column![
            labeled("Viewport", viewport),
            labeled("Mode", mode),
            labeled("Preset", preset),
            labeled("Instructions", player)
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
