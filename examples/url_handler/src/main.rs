use iced::event;
use iced::widget::{center, text};
use iced::{Element, Subscription};

pub fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .subscription(App::subscription)
        .run()
}

#[derive(Debug, Default)]
struct App {
    url: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    UrlReceived(String),
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::UrlReceived(url) => {
                self.url = Some(url);
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen_url().map(Message::UrlReceived)
    }

    fn view(&self) -> Element<Message> {
        let content = match &self.url {
            Some(url) => text(url),
            None => text("No URL received yet!"),
        };

        center(content.size(48)).into()
    }
}
