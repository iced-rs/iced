#![allow(missing_docs)]
use iced_debug as debug;
use iced_program as program;
use iced_widget as widget;
use iced_widget::core;
use iced_widget::runtime;
use iced_widget::runtime::futures;

mod executor;

use crate::core::keyboard;
use crate::core::theme::{self, Base, Theme};
use crate::core::time::seconds;
use crate::core::window;
use crate::core::{Color, Element, Length::Fill};
use crate::futures::Subscription;
use crate::program::Program;
use crate::runtime::Task;
use crate::widget::{
    bottom_right, button, center, column, container, horizontal_space, row,
    scrollable, stack, text, themer,
};

use std::fmt;
use std::io;
use std::thread;

pub fn attach(program: impl Program + 'static) -> impl Program {
    struct Attach<P> {
        program: P,
    }

    impl<P> Program for Attach<P>
    where
        P: Program + 'static,
    {
        type State = DevTools<P>;
        type Message = Event<P>;
        type Theme = P::Theme;
        type Renderer = P::Renderer;
        type Executor = P::Executor;

        fn name() -> &'static str {
            P::name()
        }

        fn boot(&self) -> (Self::State, Task<Self::Message>) {
            let (state, boot) = self.program.boot();
            let (devtools, task) = DevTools::new(state);

            (
                devtools,
                Task::batch([
                    boot.map(Event::Program),
                    task.map(Event::Message),
                ]),
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

        fn subscription(
            &self,
            state: &Self::State,
        ) -> runtime::futures::Subscription<Self::Message> {
            state.subscription(&self.program)
        }

        fn theme(
            &self,
            state: &Self::State,
            window: window::Id,
        ) -> Self::Theme {
            state.theme(&self.program, window)
        }

        fn style(
            &self,
            state: &Self::State,
            theme: &Self::Theme,
        ) -> theme::Style {
            state.style(&self.program, theme)
        }

        fn scale_factor(&self, state: &Self::State, window: window::Id) -> f64 {
            state.scale_factor(&self.program, window)
        }
    }

    Attach { program }
}

struct DevTools<P>
where
    P: Program,
{
    state: P::State,
    mode: Mode,
    show_notification: bool,
}

#[derive(Debug, Clone)]
enum Message {
    HideNotification,
    ToggleComet,
    InstallComet,
    InstallationLogged(String),
    InstallationFinished,
    CancelSetup,
}

enum Mode {
    None,
    Setup(Setup),
}

enum Setup {
    Idle,
    Running { logs: Vec<String> },
}

impl<P> DevTools<P>
where
    P: Program + 'static,
{
    pub fn new(state: P::State) -> (Self, Task<Message>) {
        (
            Self {
                state,
                mode: Mode::None,
                show_notification: true,
            },
            executor::spawn_blocking(|mut sender| {
                thread::sleep(seconds(2));
                let _ = sender.try_send(());
            })
            .map(|_| Message::HideNotification),
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
                        if matches!(setup, Setup::Idle) {
                            self.mode = Mode::None;
                        }
                    } else if let Err(error) = debug::toggle_comet() {
                        if error.kind() == io::ErrorKind::NotFound {
                            self.mode = Mode::Setup(Setup::Idle);
                        }
                    }

                    Task::none()
                }
                Message::InstallComet => {
                    self.mode =
                        Mode::Setup(Setup::Running { logs: Vec::new() });

                    executor::spawn_blocking(|mut sender| {
                        use std::io::{BufRead, BufReader};
                        use std::process::{Command, Stdio};

                        let Ok(install) = Command::new("cargo")
                            .args([
                                "install",
                                "--locked",
                                "--git",
                                "https://github.com/iced-rs/comet.git",
                                "--rev",
                                "5efd34550e42974a0e85af7560c60401bfc13919",
                            ])
                            .stdin(Stdio::null())
                            .stdout(Stdio::null())
                            .stderr(Stdio::piped())
                            .spawn()
                        else {
                            return;
                        };

                        let mut stderr = BufReader::new(
                            install.stderr.expect("stderr must be piped"),
                        );

                        let mut log = String::new();

                        while let Ok(n) = stderr.read_line(&mut log) {
                            if n == 0 {
                                break;
                            }

                            let _ = sender.try_send(
                                Message::InstallationLogged(log.clone()),
                            );

                            log.clear();
                        }

                        let _ = sender.try_send(Message::InstallationFinished);
                    })
                    .map(Event::Message)
                }
                Message::InstallationLogged(log) => {
                    if let Mode::Setup(Setup::Running { logs }) = &mut self.mode
                    {
                        logs.push(log);
                    }

                    Task::none()
                }
                Message::InstallationFinished => {
                    self.mode = Mode::None;

                    let _ = debug::toggle_comet();

                    Task::none()
                }
                Message::CancelSetup => {
                    self.mode = Mode::None;

                    Task::none()
                }
            },
            Event::Program(message) => {
                program.update(&mut self.state, message).map(Event::Program)
            }
        }
    }

    pub fn view(
        &self,
        program: &P,
        window: window::Id,
    ) -> Element<'_, Event<P>, P::Theme, P::Renderer> {
        let view = program.view(&self.state, window).map(Event::Program);
        let theme = program.theme(&self.state, window);

        let derive_theme = move || {
            theme
                .palette()
                .map(|palette| Theme::custom("DevTools".to_owned(), palette))
                .unwrap_or_default()
        };

        let mode = match &self.mode {
            Mode::None => None,
            Mode::Setup(setup) => {
                let stage: Element<'_, _, Theme, P::Renderer> = match setup {
                    Setup::Idle => {
                        let controls = row![
                            button(text("Cancel").center().width(Fill))
                                .width(100)
                                .on_press(Message::CancelSetup)
                                .style(button::danger),
                            horizontal_space(),
                            button(text("Install").center().width(Fill))
                                .width(100)
                                .on_press(Message::InstallComet)
                                .style(button::success),
                        ];

                        column![
                            text("comet is not installed!").size(20),
                            "In order to display performance metrics, the \
                            comet debugger must be installed in your system.",
                            "The comet debugger is an official companion tool \
                            that helps you debug your iced applications.",
                            "Do you wish to install it with the following \
                            command?",
                            container(
                                text(
                                    "cargo install --locked \
                                    --git https://github.com/iced-rs/comet.git"
                                )
                                .size(14)
                            )
                            .width(Fill)
                            .padding(5)
                            .style(container::dark),
                            controls,
                        ]
                        .spacing(20)
                        .into()
                    }
                    Setup::Running { logs } => column![
                        text("Installing comet...").size(20),
                        container(
                            scrollable(
                                column(
                                    logs.iter()
                                        .map(|log| text(log).size(12).into()),
                                )
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
                    .into(),
                };

                let setup = center(
                    container(stage)
                        .padding(20)
                        .width(500)
                        .style(container::bordered_box),
                )
                .padding(10)
                .style(|_theme| {
                    container::Style::default()
                        .background(Color::BLACK.scale_alpha(0.8))
                });

                Some(setup)
            }
        }
        .map(|mode| {
            themer(derive_theme(), Element::from(mode).map(Event::Message))
        });

        let notification = self.show_notification.then(|| {
            themer(
                derive_theme(),
                bottom_right(
                    container(text("Press F12 to open debug metrics"))
                        .padding(10)
                        .style(container::dark),
                ),
            )
        });

        stack![view]
            .push_maybe(mode)
            .push_maybe(notification)
            .into()
    }

    pub fn subscription(&self, program: &P) -> Subscription<Event<P>> {
        let subscription =
            program.subscription(&self.state).map(Event::Program);

        let hotkeys =
            futures::keyboard::on_key_press(|key, _modifiers| match key {
                keyboard::Key::Named(keyboard::key::Named::F12) => {
                    Some(Message::ToggleComet)
                }
                _ => None,
            })
            .map(Event::Message);

        Subscription::batch([subscription, hotkeys])
    }

    pub fn theme(&self, program: &P, window: window::Id) -> P::Theme {
        program.theme(&self.state, window)
    }

    pub fn style(&self, program: &P, theme: &P::Theme) -> theme::Style {
        program.style(&self.state, theme)
    }

    pub fn scale_factor(&self, program: &P, window: window::Id) -> f64 {
        program.scale_factor(&self.state, window)
    }
}

enum Event<P>
where
    P: Program,
{
    Message(Message),
    Program(P::Message),
}

impl<P> fmt::Debug for Event<P>
where
    P: Program,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(message) => message.fmt(f),
            Self::Program(message) => message.fmt(f),
        }
    }
}
