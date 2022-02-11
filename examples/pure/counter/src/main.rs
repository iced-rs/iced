use iced::pure::{Button, Column, Element, Sandbox, Text};
use iced::{Alignment, Settings};

pub fn main() -> iced::Result {
    Counter::run(Settings::default())
}

struct Counter {
    value: i32,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    IncrementPressed,
    DecrementPressed,
}

impl Sandbox for Counter {
    type Message = Message;

    fn new() -> Self {
        Self { value: 0 }
    }

    fn title(&self) -> String {
        String::from("Counter - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::IncrementPressed => {
                self.value += 1;
            }
            Message::DecrementPressed => {
                self.value -= 1;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        Column::new()
            .padding(20)
            .align_items(Alignment::Center)
            .push(Button::new("Increment").on_press(Message::IncrementPressed))
            .push(Text::new(self.value.to_string()).size(50))
            .push(Button::new("Decrement").on_press(Message::DecrementPressed))
            .into()
    }
}
