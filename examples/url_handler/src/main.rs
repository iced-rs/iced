use iced::event::{self, Event};
use iced::executor;
use iced::widget::{container, text};
use iced::{
    Application, Command, Element, Length, Settings, Subscription, Theme,
};

pub fn main() -> iced::Result {
    App::run(Settings::default())
}

#[derive(Debug, Default)]
struct App {
    url: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    EventOccurred(Event),
}

impl Application for App {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (App, Command<Message>) {
        (App::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Url - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
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
        };

        Command::none()
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
