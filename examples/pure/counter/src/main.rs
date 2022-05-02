use iced::pure::{button, column, text, Element, Sandbox};
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
        column()
            .padding(20)
            .align_items(Alignment::Center)
            .push(button("Increment").on_press(Message::IncrementPressed))
            .push(text(self.value.to_string()).size(50))
            .push(button("Decrement").on_press(Message::DecrementPressed))
            .into()
    }
}
