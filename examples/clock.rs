use iced::{
    Align, Application, Checkbox, Column, Command, Container, Element, Length,
    Settings, Subscription, Text,
};

pub fn main() {
    env_logger::init();

    Clock::run(Settings::default())
}

#[derive(Debug)]
struct Clock {
    time: chrono::DateTime<chrono::Local>,
    enabled: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Ticked(chrono::DateTime<chrono::Local>),
    Toggled(bool),
}

impl Application for Clock {
    type Message = Message;

    fn new() -> (Clock, Command<Message>) {
        (
            Clock {
                time: chrono::Local::now(),
                enabled: false,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Clock - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Ticked(time) => {
                self.time = time;
            }
            Message::Toggled(enabled) => {
                self.enabled = enabled;
            }
        };

        Command::none()
    }

    fn subscriptions(&self) -> Subscription<Message> {
        if self.enabled {
            time::every(std::time::Duration::from_millis(500), Message::Ticked)
        } else {
            Subscription::none()
        }
    }

    fn view(&mut self) -> Element<Message> {
        let clock = Text::new(format!("{}", self.time.format("%H:%M:%S")))
            .size(40)
            .width(Length::Shrink);

        let toggle = Checkbox::new(self.enabled, "Enabled", Message::Toggled)
            .width(Length::Shrink);

        let content = Column::new()
            .width(Length::Shrink)
            .align_items(Align::Center)
            .spacing(20)
            .push(clock)
            .push(toggle);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

mod time {
    use std::sync::Arc;

    pub fn every<Message>(
        duration: std::time::Duration,
        f: impl Fn(chrono::DateTime<chrono::Local>) -> Message
            + 'static
            + Send
            + Sync,
    ) -> iced::Subscription<Message>
    where
        Message: Send + 'static,
    {
        Tick {
            duration,
            message: Arc::new(f),
        }
        .into()
    }

    struct Tick<Message> {
        duration: std::time::Duration,
        message: Arc<
            dyn Fn(chrono::DateTime<chrono::Local>) -> Message + Send + Sync,
        >,
    }

    impl<Message> iced_native::subscription::Connection for Tick<Message>
    where
        Message: 'static,
    {
        type Input = iced_native::subscription::Input;
        type Output = Message;

        fn id(&self) -> u64 {
            0
        }

        fn stream(
            &self,
            input: iced_native::subscription::Input,
        ) -> futures::stream::BoxStream<'static, Message> {
            use futures::StreamExt;

            let function = self.message.clone();

            input.map(move |_| function(chrono::Local::now())).boxed()
        }
    }
}
