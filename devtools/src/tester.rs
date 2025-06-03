mod recorder;

use recorder::recorder;

use crate::Program;
use crate::core::Alignment::Center;
use crate::core::Length::Fill;
use crate::core::alignment::Horizontal::Right;
use crate::core::border;
use crate::core::window;
use crate::core::{Element, Event, Size, Theme};
use crate::futures::futures::channel::mpsc;
use crate::icon;
use crate::program;
use crate::runtime::Task;
use crate::test::emulator;
use crate::test::instruction;
use crate::test::{Emulator, Instruction};
use crate::widget::{
    button, center, column, container, monospace, row, scrollable, text,
    text_input, themer,
};

pub struct Tester<P: Program> {
    viewport: Size,
    instructions: Vec<Instruction>,
    state: State<P>,
}

enum State<P: Program> {
    Idle,
    Recording {
        state: P::State,
    },
    Playing {
        emulator: Emulator<P>,
        current: usize,
    },
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeViewport(Size),
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
    pub fn new() -> Self {
        Self {
            viewport: Size::new(512.0, 512.0),
            instructions: Vec::new(),
            state: State::Idle,
        }
    }

    pub fn is_busy(&self) -> bool {
        matches!(self.state, State::Idle | State::Playing { .. })
    }

    pub fn update(&mut self, program: &P, message: Message) -> Task<Tick<P>> {
        match message {
            Message::ChangeViewport(viewport) => {
                self.viewport = viewport;

                Task::none()
            }
            Message::Record => {
                self.instructions.clear();

                let (state, task) = program.boot();
                self.state = State::Recording { state };

                task.map(Tick::Program)
            }
            Message::Stop => {
                self.state = State::Idle;

                Task::none()
            }
            Message::Play => {
                let (sender, receiver) = mpsc::channel(1);
                let emulator = Emulator::new(program, self.viewport, sender);

                self.state = State::Playing {
                    emulator,
                    current: 0,
                };

                Task::run(receiver, Tick::Emulator)
            }
        }
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
                let State::Playing { emulator, current } = &mut self.state
                else {
                    return Task::none();
                };

                match event {
                    emulator::Event::Action(action) => {
                        emulator.perform(program, action);
                    }
                    emulator::Event::Ready => {
                        if let Some(instruction) =
                            self.instructions.get(*current).cloned()
                        {
                            emulator.run(program, instruction);
                            *current += 1;
                        }
                    }
                }

                Task::none()
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
        let status = match &self.state {
            State::Idle => monospace("Idle").style(|theme| text::Style {
                color: Some(
                    theme.extended_palette().background.strongest.color,
                ),
            }),
            State::Recording { .. } => {
                monospace("Recording").style(|theme| text::Style {
                    color: Some(theme.palette().danger),
                })
            }
            State::Playing { .. } => {
                monospace("Playing").style(|theme| text::Style {
                    color: Some(theme.palette().primary),
                })
            }
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
                    State::Playing { .. } => palette.primary.base.color,
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
            monospace("x"),
            text_input("Height", &self.viewport.height.to_string())
                .size(14)
                .on_input(|height| Message::ChangeViewport(Size {
                    height: height.parse().unwrap_or(self.viewport.height),
                    ..self.viewport
                })),
        ]
        .spacing(10)
        .align_y(Center);

        let player = {
            let events = container(if self.instructions.is_empty() {
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
                                        State::Playing { current, .. } => {
                                            if *current == i {
                                                Some(theme.palette().primary)
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
                                matches!(self.state, State::Idle)
                                    .then_some(Message::Record),
                            )
                            .style(button::danger)
                    }
                ]
                .spacing(10)
            };

            column![events, controls].spacing(10).align_x(Center)
        };

        column![labeled("Viewport", viewport), labeled("Tester", player)]
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
