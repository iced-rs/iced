use iced::widget::{Container, Text};
use iced::{Sandbox, Settings};

// Application state. It is a unit struct because we are not storing any state.
struct HelloWorld;

// Message type. It is a unit struct because we are not sending any messages to
// modify the application state. Any message type should implement these three traits.
#[derive(Debug, Clone, Copy)]
struct DummyMessage;

// `Sandbox` trait allows us to easily create a simple application. It is easy to get started but
// for more complex applications, we will use the `iced::Application` trait.
impl Sandbox for HelloWorld {
    type Message = DummyMessage;

    fn new() -> Self {
        HelloWorld
    }

    fn title(&self) -> String {
        String::from("Hello World App")
    }

    fn update(&mut self, _message: Self::Message) {}

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let text = Text::new("Hello World!");
        let container = Container::new(text)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .center_x()
            .center_y();
        container.into()
    }
}

fn main() -> iced::Result {
    HelloWorld::run(Settings::default())
}
