//! Hello world example
use iced::widget::{
    text
};
use iced::{Element, Task };

/// entry point
pub fn main() -> iced::Result {
    iced::run("Window title", HelloWorld::update, HelloWorld::view)
}

#[derive(Debug)]
#[allow(dead_code)]
enum Message {
    None
}

#[derive(Default)]
struct HelloWorld {}

impl HelloWorld {
    fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let text = text("hello world");

        text.into()
    }
}