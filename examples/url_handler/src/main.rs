use iced::event::{self, Event};
use iced::widget::{container, text};
use iced::{Element, Length, Subscription};

pub fn main() -> iced::Result {
    iced::sandbox("URL Handler - Iced", App::update, App::view)
        .subscription(App::subscription)
        .run()
}

#[derive(Debug, Default)]
struct App {
    url: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    EventOccurred(Event),
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::EventOccurred(event) => {
                if let Event::PlatformSpecific(
                    event::PlatformSpecific::MacOS(event::MacOS::ReceivedUrl(
                        url,
                    )),
                ) = event
                {
                    self.url = Some(url);
                }
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::EventOccurred)
    }

    fn view(&self) -> Element<Message> {
        let content = match &self.url {
            Some(url) => text(url),
            None => text("No URL received yet!"),
        };

        container(content.size(48))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
