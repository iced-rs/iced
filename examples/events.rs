use iced::{
    Align, Application, Checkbox, Column, Command, Container, Element, Length,
    Settings, Subscription, Text,
};

pub fn main() {
    env_logger::init();

    Events::run(Settings::default())
}

#[derive(Debug, Default)]
struct Events {
    last: Vec<iced_native::Event>,
    enabled: bool,
}

#[derive(Debug, Clone)]
enum Message {
    EventOccurred(iced_native::Event),
    Toggled(bool),
}

impl Application for Events {
    type Message = Message;

    fn new() -> (Events, Command<Message>) {
        (Events::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Events - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::EventOccurred(event) => {
                self.last.push(event);

                if self.last.len() > 5 {
                    let _ = self.last.remove(0);
                }
            }
            Message::Toggled(enabled) => {
                self.enabled = enabled;
            }
        };

        Command::none()
    }

    fn subscriptions(&self) -> Subscription<Message> {
        if self.enabled {
            events::all(Message::EventOccurred)
        } else {
            Subscription::none()
        }
    }

    fn view(&mut self) -> Element<Message> {
        let events = self.last.iter().fold(
            Column::new().width(Length::Shrink).spacing(10),
            |column, event| {
                column.push(
                    Text::new(format!("{:?}", event))
                        .size(40)
                        .width(Length::Shrink),
                )
            },
        );

        let toggle = Checkbox::new(self.enabled, "Enabled", Message::Toggled)
            .width(Length::Shrink);

        let content = Column::new()
            .width(Length::Shrink)
            .align_items(Align::Center)
            .spacing(20)
            .push(events)
            .push(toggle);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

mod events {
    use std::sync::Arc;

    pub fn all<Message>(
        f: impl Fn(iced_native::Event) -> Message + 'static + Send + Sync,
    ) -> iced::Subscription<Message>
    where
        Message: Send + 'static,
    {
        All(Arc::new(f)).into()
    }

    struct All<Message>(
        Arc<dyn Fn(iced_native::Event) -> Message + Send + Sync>,
    );

    impl<Message> iced_native::subscription::Connection for All<Message>
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

            let function = self.0.clone();

            input.map(move |event| function(event)).boxed()
        }
    }
}
