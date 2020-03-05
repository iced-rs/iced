use iced::{
    panes, Application, Command, Element, Panes, Settings, Subscription,
};
use iced_native::input::keyboard;

use clock::{self, Clock};
use stopwatch::{self, Stopwatch};

pub fn main() {
    Launcher::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

#[derive(Debug)]
struct Launcher {
    panes: panes::State<Example>,
}

#[derive(Debug)]
enum Example {
    Clock(Clock),
    Stopwatch(Stopwatch),
}

#[derive(Debug, Clone)]
enum Message {
    Clock(panes::Pane, clock::Message),
    Stopwatch(panes::Pane, stopwatch::Message),
    Split(panes::Split),
}

impl Application for Launcher {
    type Executor = iced::executor::Default;
    type Message = Message;

    fn new() -> (Self, Command<Message>) {
        let (clock, _) = Clock::new();
        let (panes, _) = panes::State::new(Example::Clock(clock));

        (Self { panes }, Command::none())
    }

    fn title(&self) -> String {
        String::from("Panes - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Clock(pane, message) => {
                if let Some(Example::Clock(clock)) = self.panes.get_mut(&pane) {
                    let _ = clock.update(message);
                }
            }
            Message::Stopwatch(pane, message) => {
                if let Some(Example::Stopwatch(stopwatch)) =
                    self.panes.get_mut(&pane)
                {
                    let _ = stopwatch.update(message);
                }
            }
            Message::Split(kind) => {
                if let Some(pane) = self.panes.focused_pane() {
                    let state = if pane.index() % 2 == 0 {
                        let (stopwatch, _) = Stopwatch::new();

                        Example::Stopwatch(stopwatch)
                    } else {
                        let (clock, _) = Clock::new();

                        Example::Clock(clock)
                    };

                    self.panes.split(kind, &pane, state);
                }
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        let panes_subscriptions =
            Subscription::batch(self.panes.iter().map(|(pane, example)| {
                match example {
                    Example::Clock(clock) => clock
                        .subscription()
                        .with(pane)
                        .map(|(pane, message)| Message::Clock(pane, message)),

                    Example::Stopwatch(stopwatch) => {
                        stopwatch.subscription().with(pane).map(
                            |(pane, message)| Message::Stopwatch(pane, message),
                        )
                    }
                }
            }));

        Subscription::batch(vec![
            events::key_released(keyboard::KeyCode::H)
                .map(|_| Message::Split(panes::Split::Horizontal)),
            events::key_released(keyboard::KeyCode::V)
                .map(|_| Message::Split(panes::Split::Vertical)),
            panes_subscriptions,
        ])
    }

    fn view(&mut self) -> Element<Message> {
        let Self { panes } = self;

        Panes::new(panes, |pane, example| match example {
            Example::Clock(clock) => clock
                .view()
                .map(move |message| Message::Clock(pane, message)),

            Example::Stopwatch(stopwatch) => stopwatch
                .view()
                .map(move |message| Message::Stopwatch(pane, message)),
        })
        .into()
    }
}

mod events {
    use iced_native::{
        futures::{
            self,
            stream::{BoxStream, StreamExt},
        },
        input::{keyboard, ButtonState},
        subscription, Event, Hasher, Subscription,
    };

    pub fn key_released(key_code: keyboard::KeyCode) -> Subscription<()> {
        Subscription::from_recipe(KeyReleased { key_code })
    }

    struct KeyReleased {
        key_code: keyboard::KeyCode,
    }

    impl subscription::Recipe<Hasher, Event> for KeyReleased {
        type Output = ();

        fn hash(&self, state: &mut Hasher) {
            use std::hash::Hash;

            std::any::TypeId::of::<Self>().hash(state);
            self.key_code.hash(state);
        }

        fn stream(
            self: Box<Self>,
            events: subscription::EventStream,
        ) -> BoxStream<'static, Self::Output> {
            events
                .filter(move |event| match event {
                    Event::Keyboard(keyboard::Event::Input {
                        key_code,
                        state: ButtonState::Released,
                        ..
                    }) if *key_code == self.key_code => {
                        futures::future::ready(true)
                    }
                    _ => futures::future::ready(false),
                })
                .map(|_| ())
                .boxed()
        }
    }
}
