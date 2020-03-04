use iced::{
    panes, Application, Command, Element, Panes, Settings, Subscription,
};

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
        }

        dbg!(self);

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(self.panes.iter().map(|(pane, example)| {
            match example {
                Example::Clock(clock) => clock
                    .subscription()
                    .map(move |message| Message::Clock(pane, message)),

                Example::Stopwatch(stopwatch) => stopwatch
                    .subscription()
                    .map(move |message| Message::Stopwatch(pane, message)),
            }
        }))
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
