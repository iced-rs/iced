use iced::{
    button, Align, Application, Background, Button, Color, Column, Command,
    Container, Element, HorizontalAlignment, Length, Row, Settings,
    Subscription, Text,
};
use std::time::{Duration, Instant};

pub fn main() {
    Timer::run(Settings::default())
}

struct Timer {
    duration: Duration,
    state: State,
    toggle: button::State,
    reset: button::State,
}

enum State {
    Idle,
    Ticking { last_tick: Instant },
}

#[derive(Debug, Clone)]
enum Message {
    Toggle,
    Reset,
    Tick(Instant),
}

impl Application for Timer {
    type Message = Message;

    fn new() -> (Timer, Command<Message>) {
        (
            Timer {
                duration: Duration::default(),
                state: State::Idle,
                toggle: button::State::new(),
                reset: button::State::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Timer - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
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
            Message::Tick(now) => match &mut self.state {
                State::Ticking { last_tick } => {
                    self.duration += now - *last_tick;
                    *last_tick = now;
                }
                _ => {}
            },
            Message::Reset => {
                self.duration = Duration::default();
            }
        }

        Command::none()
    }

    fn subscriptions(&self) -> Subscription<Message> {
        match self.state {
            State::Idle => Subscription::none(),
            State::Ticking { .. } => {
                time::every(Duration::from_millis(10)).map(Message::Tick)
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        const MINUTE: u64 = 60;
        const HOUR: u64 = 60 * MINUTE;

        let seconds = self.duration.as_secs();

        let duration = Text::new(format!(
            "{:0>2}:{:0>2}:{:0>2}.{:0>2}",
            seconds / HOUR,
            seconds / MINUTE,
            seconds % MINUTE,
            self.duration.subsec_millis() / 10,
        ))
        .width(Length::Shrink)
        .size(40);

        let button = |state, label, color: [f32; 3]| {
            Button::new(
                state,
                Text::new(label)
                    .color(Color::WHITE)
                    .horizontal_alignment(HorizontalAlignment::Center),
            )
            .min_width(80)
            .background(Background::Color(color.into()))
            .border_radius(10)
            .padding(10)
        };

        let toggle_button = {
            let (label, color) = match self.state {
                State::Idle => ("Start", [0.11, 0.42, 0.87]),
                State::Ticking { .. } => ("Stop", [0.9, 0.4, 0.4]),
            };

            button(&mut self.toggle, label, color).on_press(Message::Toggle)
        };

        let reset_button = button(&mut self.reset, "Reset", [0.7, 0.7, 0.7])
            .on_press(Message::Reset);

        let controls = Row::new()
            .width(Length::Shrink)
            .spacing(20)
            .push(toggle_button)
            .push(reset_button);

        let content = Column::new()
            .width(Length::Shrink)
            .align_items(Align::Center)
            .spacing(20)
            .push(duration)
            .push(controls);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

mod time {
    pub fn every(
        duration: std::time::Duration,
    ) -> iced::Subscription<std::time::Instant> {
        iced::Subscription::from_recipe(Every(duration))
    }

    struct Every(std::time::Duration);

    impl<Input> iced_native::subscription::Recipe<iced_native::Hasher, Input>
        for Every
    {
        type Output = std::time::Instant;

        fn hash(&self, state: &mut iced_native::Hasher) {
            use std::hash::Hash;

            std::any::TypeId::of::<Self>().hash(state);
        }

        fn stream(
            &self,
            _input: Input,
        ) -> futures::stream::BoxStream<'static, Self::Output> {
            use futures::stream::StreamExt;

            async_std::stream::interval(self.0)
                .map(|_| std::time::Instant::now())
                .boxed()
        }
    }
}
