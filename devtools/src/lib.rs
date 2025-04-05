#![allow(missing_docs)]
use iced_debug as debug;
use iced_program as program;
use iced_widget as widget;
use iced_widget::core;
use iced_widget::runtime;
use iced_widget::runtime::futures;

use crate::core::Element;
use crate::core::keyboard;
use crate::core::theme::{self, Base, Theme};
use crate::core::time::seconds;
use crate::core::window;
use crate::futures::Subscription;
use crate::program::Program;
use crate::runtime::Task;
use crate::widget::{bottom_right, container, stack, text, themer};

use std::fmt;

pub fn attach(program: impl Program + 'static) -> impl Program {
    struct Attach<P> {
        program: P,
    }

    impl<P> Program for Attach<P>
    where
        P: Program + 'static,
    {
        type State = DevTools<P>;
        type Message = Message<P>;
        type Theme = P::Theme;
        type Renderer = P::Renderer;
        type Executor = P::Executor;

        fn name() -> &'static str {
            P::name()
        }

        fn boot(&self) -> (Self::State, Task<Self::Message>) {
            let (state, boot) = self.program.boot();
            let (devtools, task) = DevTools::new(state);

            (devtools, Task::batch([boot.map(Message::Program), task]))
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
    show_notification: bool,
}

impl<P> DevTools<P>
where
    P: Program + 'static,
{
    pub fn new(state: P::State) -> (Self, Task<Message<P>>) {
        (
            Self {
                state,
                show_notification: true,
            },
            Task::perform(smol::Timer::after(seconds(2)), |_| {
                Message::HideNotification
            }),
        )
    }

    pub fn title(&self, program: &P, window: window::Id) -> String {
        program.title(&self.state, window)
    }

    pub fn update(
        &mut self,
        program: &P,
        message: Message<P>,
    ) -> Task<Message<P>> {
        match message {
            Message::HideNotification => {
                self.show_notification = false;

                Task::none()
            }
            Message::ToggleComet => {
                debug::toggle_comet();

                Task::none()
            }
            Message::Program(message) => program
                .update(&mut self.state, message)
                .map(Message::Program),
        }
    }

    pub fn view(
        &self,
        program: &P,
        window: window::Id,
    ) -> Element<'_, Message<P>, P::Theme, P::Renderer> {
        let view = program.view(&self.state, window).map(Message::Program);
        let theme = program.theme(&self.state, window);

        let notification = themer(
            theme
                .palette()
                .map(|palette| Theme::custom("DevTools".to_owned(), palette))
                .unwrap_or_default(),
            bottom_right(
                container(text("Press F12 to open debug metrics"))
                    .padding(10)
                    .style(container::dark),
            ),
        );

        stack![view]
            .push_maybe(self.show_notification.then_some(notification))
            .into()
    }

    pub fn subscription(&self, program: &P) -> Subscription<Message<P>> {
        let subscription =
            program.subscription(&self.state).map(Message::Program);

        let hotkeys =
            futures::keyboard::on_key_press(|key, _modifiers| match key {
                keyboard::Key::Named(keyboard::key::Named::F12) => {
                    Some(Message::ToggleComet)
                }
                _ => None,
            });

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

#[derive(Clone)]
enum Message<P>
where
    P: Program,
{
    HideNotification,
    ToggleComet,
    Program(P::Message),
}

impl<P> fmt::Debug for Message<P>
where
    P: Program,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::HideNotification => {
                f.write_str("DevTools(HideNotification)")
            }
            Message::ToggleComet => f.write_str("DevTools(ToggleComet)"),
            Message::Program(message) => message.fmt(f),
        }
    }
}
