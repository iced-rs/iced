use iced::keyboard;
use iced::time;
use iced::widget::{button, center, column, row, text};
use iced::{Center, Element, Subscription, Theme};

use std::time::{Duration, Instant};

pub fn main() -> iced::Result {
    iced::application("Stopwatch - Iced", Stopwatch::update, Stopwatch::view)
        .subscription(Stopwatch::subscription)
        .theme(Stopwatch::theme)
        .run()
}

#[derive(Default)]
struct Stopwatch {
    duration: Duration,
    state: State,
}

#[derive(Default)]
enum State {
    #[default]
    Idle,
    Ticking {
        last_tick: Instant,
    },
}

#[derive(Debug, Clone)]
enum Message {
    Toggle,
    Reset,
    Tick(Instant),
}

impl Stopwatch {
    fn update(&mut self, message: Message) {
        match message {
            Message::Toggle => match self.state {
                State::Idle => {
                    self.state = State::Ticking {
                        last_tick: Instant::now(),
                    };
                }
                State::Ticking { .. } => {
                    self.state = State::Idle;
                }
            },
            Message::Tick(now) => {
                if let State::Ticking { last_tick } = &mut self.state {
                    self.duration += now - *last_tick;
                    *last_tick = now;
                }
            }
            Message::Reset => {
                self.duration = Duration::default();
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let tick = match self.state {
            State::Idle => Subscription::none(),
            State::Ticking { .. } => {
                time::every(Duration::from_millis(10)).map(Message::Tick)
            }
        };

        fn handle_hotkey(
            key: keyboard::Key,
            _modifiers: keyboard::Modifiers,
        ) -> Option<Message> {
            use keyboard::key;

            match key.as_ref() {
                keyboard::Key::Named(key::Named::Space) => {
                    Some(Message::Toggle)
                }
                keyboard::Key::Character("r") => Some(Message::Reset),
                _ => None,
            }
        }

        Subscription::batch(vec![tick, keyboard::on_key_press(handle_hotkey)])
    }

    fn view(&self) -> Element<Message> {
        const MINUTE: u64 = 60;
        const HOUR: u64 = 60 * MINUTE;

        let seconds = self.duration.as_secs();

        let duration = text!(
            "{:0>2}:{:0>2}:{:0>2}.{:0>2}",
            seconds / HOUR,
            (seconds % HOUR) / MINUTE,
            seconds % MINUTE,
            self.duration.subsec_millis() / 10,
        )
        .size(40);

        let button =
            |label| button(text(label).align_x(Center)).padding(10).width(80);

        let toggle_button = {
            let label = match self.state {
                State::Idle => "Start",
                State::Ticking { .. } => "Stop",
            };

            button(label).on_press(Message::Toggle)
        };

        let reset_button = button("Reset")
            .style(button::danger)
            .on_press(Message::Reset);

        let controls = row![toggle_button, reset_button].spacing(20);

        let content = column![duration, controls].align_x(Center).spacing(20);

        center(content).into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
