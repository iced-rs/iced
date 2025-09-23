#![allow(missing_docs)]
use iced_debug as debug;
use iced_program as program;
use iced_program::runtime;
use iced_program::runtime::futures;
use iced_widget as widget;
use iced_widget::core;

mod comet;
mod time_machine;

use crate::core::border;
use crate::core::keyboard;
use crate::core::theme::{self, Theme};
use crate::core::time::seconds;
use crate::core::window;
use crate::core::{
    Alignment::Center, Color, Element, Font, Length::Fill, Settings,
};
use crate::futures::Subscription;
use crate::program::Program;
use crate::program::message;
use crate::runtime::task::{self, Task};
use crate::time_machine::TimeMachine;
use crate::widget::{
    bottom_right, button, center, column, container, opaque, row, scrollable,
    space, stack, text, themer,
};

use std::fmt;
use std::thread;

pub fn attach<P: Program + 'static>(program: P) -> Attach<P> {
    Attach { program }
}

/// A [`Program`] with some devtools attached to it.
#[derive(Debug)]
pub struct Attach<P> {
    /// The original [`Program`] managed by these devtools.
    pub program: P,
}

impl<P> Program for Attach<P>
where
    P: Program + 'static,
    P::Message: std::fmt::Debug + message::MaybeClone,
{
    type State = DevTools<P>;
    type Message = Event<P>;
    type Theme = P::Theme;
    type Renderer = P::Renderer;
    type Executor = P::Executor;

    fn name() -> &'static str {
        P::name()
    }

    fn settings(&self) -> Settings {
        self.program.settings()
    }

    fn window(&self) -> Option<window::Settings> {
        self.program.window()
    }

    fn boot(&self) -> (Self::State, Task<Self::Message>) {
        let (state, boot) = self.program.boot();
        let (devtools, task) = DevTools::new(state);

        (
            devtools,
            Task::batch([boot.map(Event::Program), task.map(Event::Message)]),
        )
    }

    fn update(
        &self,
        state: &mut Self::State,
        message: Self::Message,
    ) -> Task<Self::Message> {
        state.update(&self.program, message)
    }

    fn view<'a>(
        &self,
        state: &'a Self::State,
        window: window::Id,
    ) -> Element<'a, Self::Message, Self::Theme, Self::Renderer> {
        state.view(&self.program, window)
    }

    fn title(&self, state: &Self::State, window: window::Id) -> String {
        state.title(&self.program, window)
    }

    fn subscription(&self, state: &Self::State) -> Subscription<Self::Message> {
        state.subscription(&self.program)
    }

    fn theme(
        &self,
        state: &Self::State,
        window: window::Id,
    ) -> Option<Self::Theme> {
        state.theme(&self.program, window)
    }

    fn style(&self, state: &Self::State, theme: &Self::Theme) -> theme::Style {
        state.style(&self.program, theme)
    }

    fn scale_factor(&self, state: &Self::State, window: window::Id) -> f32 {
        state.scale_factor(&self.program, window)
    }
}

/// The state of the devtools.
pub struct DevTools<P>
where
    P: Program,
{
    state: P::State,
    show_notification: bool,
    time_machine: TimeMachine<P>,
    mode: Mode,
}

#[derive(Debug, Clone)]
pub enum Message {
    HideNotification,
    ToggleComet,
    CometLaunched(comet::launch::Result),
    InstallComet,
    Installing(comet::install::Result),
    CancelSetup,
}

enum Mode {
    Hidden,
    Setup(Setup),
}

enum Setup {
    Idle { goal: Goal },
    Running { logs: Vec<String> },
}

enum Goal {
    Installation,
    Update { revision: Option<String> },
}

impl<P> DevTools<P>
where
    P: Program + 'static,
    P::Message: std::fmt::Debug + message::MaybeClone,
{
    pub fn new(state: P::State) -> (Self, Task<Message>) {
        (
            Self {
                state,
                mode: Mode::Hidden,
                show_notification: true,
                time_machine: TimeMachine::new(),
            },
            Task::batch([task::blocking(|mut sender| {
                thread::sleep(seconds(2));
                let _ = sender.try_send(());
            })
            .map(|_| Message::HideNotification)]),
        )
    }

    pub fn title(&self, program: &P, window: window::Id) -> String {
        program.title(&self.state, window)
    }

    pub fn update(&mut self, program: &P, event: Event<P>) -> Task<Event<P>> {
        match event {
            Event::Message(message) => match message {
                Message::HideNotification => {
                    self.show_notification = false;

                    Task::none()
                }
                Message::ToggleComet => {
                    if let Mode::Setup(setup) = &self.mode {
                        if matches!(setup, Setup::Idle { .. }) {
                            self.mode = Mode::Hidden;
                        }

                        Task::none()
                    } else if debug::quit() {
                        Task::none()
                    } else {
                        comet::launch()
                            .map(Message::CometLaunched)
                            .map(Event::Message)
                    }
                }
                Message::CometLaunched(Ok(())) => Task::none(),
                Message::CometLaunched(Err(error)) => {
                    match error {
                        comet::launch::Error::NotFound => {
                            self.mode = Mode::Setup(Setup::Idle {
                                goal: Goal::Installation,
                            });
                        }
                        comet::launch::Error::Outdated { revision } => {
                            self.mode = Mode::Setup(Setup::Idle {
                                goal: Goal::Update { revision },
                            });
                        }
                        comet::launch::Error::IoFailed(error) => {
                            log::error!("comet failed to run: {error}");
                        }
                    }

                    Task::none()
                }
                Message::InstallComet => {
                    self.mode =
                        Mode::Setup(Setup::Running { logs: Vec::new() });

                    comet::install()
                        .map(Message::Installing)
                        .map(Event::Message)
                }
                Message::Installing(Ok(installation)) => {
                    let Mode::Setup(Setup::Running { logs }) = &mut self.mode
                    else {
                        return Task::none();
                    };

                    match installation {
                        comet::install::Event::Logged(log) => {
                            logs.push(log);
                            Task::none()
                        }
                        comet::install::Event::Finished => {
                            self.mode = Mode::Hidden;
                            comet::launch().discard()
                        }
                    }
                }
                Message::Installing(Err(error)) => {
                    let Mode::Setup(Setup::Running { logs }) = &mut self.mode
                    else {
                        return Task::none();
                    };

                    match error {
                        comet::install::Error::ProcessFailed(status) => {
                            logs.push(format!("process failed with {status}"));
                        }
                        comet::install::Error::IoFailed(error) => {
                            logs.push(error.to_string());
                        }
                    }

                    Task::none()
                }
                Message::CancelSetup => {
                    self.mode = Mode::Hidden;

                    Task::none()
                }
            },
            Event::Program(message) => {
                self.time_machine.push(&message);

                if self.time_machine.is_rewinding() {
                    debug::enable();
                }

                let span = debug::update(&message);
                let task = program.update(&mut self.state, message);
                debug::tasks_spawned(task.units());
                span.finish();

                if self.time_machine.is_rewinding() {
                    debug::disable();
                }

                task.map(Event::Program)
            }
            Event::Command(command) => {
                match command {
                    debug::Command::RewindTo { message } => {
                        self.time_machine.rewind(program, message);
                    }
                    debug::Command::GoLive => {
                        self.time_machine.go_to_present();
                    }
                }

                Task::none()
            }
            Event::Discard => Task::none(),
        }
    }

    pub fn view(
        &self,
        program: &P,
        window: window::Id,
    ) -> Element<'_, Event<P>, P::Theme, P::Renderer> {
        let state = self.state();

        let view = {
            let view = program.view(state, window);

            if self.time_machine.is_rewinding() {
                view.map(|_| Event::Discard)
            } else {
                view.map(Event::Program)
            }
        };

        let theme = || {
            program
                .theme(state, window)
                .as_ref()
                .and_then(theme::Base::palette)
                .map(|palette| Theme::custom("iced devtools", palette))
        };

        let setup = if let Mode::Setup(setup) = &self.mode {
            let stage: Element<'_, _, Theme, P::Renderer> = match setup {
                Setup::Idle { goal } => self::setup(goal),
                Setup::Running { logs } => installation(logs),
            };

            let setup = center(
                container(stage)
                    .padding(20)
                    .max_width(500)
                    .style(container::bordered_box),
            )
            .padding(10)
            .style(|_theme| {
                container::Style::default()
                    .background(Color::BLACK.scale_alpha(0.8))
            });

            Some(themer(theme(), Element::from(setup).map(Event::Message)))
        } else {
            None
        };

        let notification = self
            .show_notification
            .then(|| text("Press F12 to open debug metrics"))
            .or_else(|| {
                debug::is_stale().then(|| {
                    text(
                        "Types have changed. Restart to re-enable hotpatching.",
                    )
                })
            })
            .map(|notification| {
                themer(
                    theme(),
                    bottom_right(opaque(
                        container(notification)
                            .padding(10)
                            .style(container::dark),
                    )),
                )
            });

        stack![view, setup, notification]
            .width(Fill)
            .height(Fill)
            .into()
    }

    pub fn subscription(&self, program: &P) -> Subscription<Event<P>> {
        let subscription =
            program.subscription(&self.state).map(Event::Program);
        debug::subscriptions_tracked(subscription.units());

        let hotkeys =
            futures::keyboard::on_key_press(|key, _modifiers| match key {
                keyboard::Key::Named(keyboard::key::Named::F12) => {
                    Some(Message::ToggleComet)
                }
                _ => None,
            })
            .map(Event::Message);

        let commands = debug::commands().map(Event::Command);

        Subscription::batch([subscription, hotkeys, commands])
    }

    pub fn theme(&self, program: &P, window: window::Id) -> Option<P::Theme> {
        program.theme(self.state(), window)
    }

    pub fn style(&self, program: &P, theme: &P::Theme) -> theme::Style {
        program.style(self.state(), theme)
    }

    pub fn scale_factor(&self, program: &P, window: window::Id) -> f32 {
        program.scale_factor(self.state(), window)
    }

    pub fn state(&self) -> &P::State {
        self.time_machine.state().unwrap_or(&self.state)
    }
}

pub enum Event<P>
where
    P: Program,
{
    Message(Message),
    Program(P::Message),
    Command(debug::Command),
    Discard,
}

impl<P> fmt::Debug for Event<P>
where
    P: Program,
    P::Message: std::fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(message) => message.fmt(f),
            Self::Program(message) => message.fmt(f),
            Self::Command(command) => command.fmt(f),
            Self::Discard => f.write_str("Discard"),
        }
    }
}

fn setup<Renderer>(goal: &Goal) -> Element<'_, Message, Theme, Renderer>
where
    Renderer: program::Renderer + 'static,
{
    let controls = row![
        button(text("Cancel").center().width(Fill))
            .width(100)
            .on_press(Message::CancelSetup)
            .style(button::danger),
        space::horizontal(),
        button(
            text(match goal {
                Goal::Installation => "Install",
                Goal::Update { .. } => "Update",
            })
            .center()
            .width(Fill)
        )
        .width(100)
        .on_press(Message::InstallComet)
        .style(button::success),
    ];

    let command = container(
        text!(
            "cargo install --locked \\
    --git https://github.com/iced-rs/comet.git \\
    --rev {}",
            comet::COMPATIBLE_REVISION
        )
        .size(14)
        .font(Font::MONOSPACE),
    )
    .width(Fill)
    .padding(5)
    .style(container::dark);

    Element::from(match goal {
        Goal::Installation => column![
            text("comet is not installed!").size(20),
            "In order to display performance \
                metrics, the  comet debugger must \
                be installed in your system.",
            "The comet debugger is an official \
                companion tool that helps you debug \
                your iced applications.",
            column![
                "Do you wish to install it with the \
                    following command?",
                command
            ]
            .spacing(10),
            controls,
        ]
        .spacing(20),
        Goal::Update { revision } => {
            let comparison = column![
                row![
                    "Installed revision:",
                    space::horizontal(),
                    inline_code(revision.as_deref().unwrap_or("Unknown"))
                ]
                .align_y(Center),
                row![
                    "Compatible revision:",
                    space::horizontal(),
                    inline_code(comet::COMPATIBLE_REVISION),
                ]
                .align_y(Center)
            ]
            .spacing(5);

            column![
                text("comet is out of date!").size(20),
                comparison,
                column![
                    "Do you wish to update it with the following \
                        command?",
                    command
                ]
                .spacing(10),
                controls,
            ]
            .spacing(20)
        }
    })
}

fn installation<'a, Renderer>(
    logs: &'a [String],
) -> Element<'a, Message, Theme, Renderer>
where
    Renderer: program::Renderer + 'a,
{
    column![
        text("Installing comet...").size(20),
        container(
            scrollable(
                column(logs.iter().map(|log| {
                    text(log).size(12).font(Font::MONOSPACE).into()
                }))
                .spacing(3),
            )
            .spacing(10)
            .width(Fill)
            .height(300)
            .anchor_bottom(),
        )
        .padding(10)
        .style(container::dark)
    ]
    .spacing(20)
    .into()
}

fn inline_code<'a, Renderer>(
    code: impl text::IntoFragment<'a>,
) -> Element<'a, Message, Theme, Renderer>
where
    Renderer: program::Renderer + 'a,
{
    container(text(code).size(12).font(Font::MONOSPACE))
        .style(|_theme| {
            container::Style::default()
                .background(Color::BLACK)
                .border(border::rounded(2))
        })
        .padding([2, 4])
        .into()
}
