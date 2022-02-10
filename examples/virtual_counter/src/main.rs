use iced::{Alignment, Element, Sandbox, Settings};
use iced_virtual::{Button, Column, Text, Virtual};

pub fn main() -> iced::Result {
    Counter::run(Settings::default())
}

struct Counter {
    value: i32,
    state: iced_virtual::State<Message, iced::Renderer>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    IncrementPressed,
    DecrementPressed,
}

impl Sandbox for Counter {
    type Message = Message;

    fn new() -> Self {
        Self {
            value: 0,
            state: iced_virtual::State::new(),
        }
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

    fn view(&mut self) -> Element<Message> {
        let content = Column::new()
            .padding(20)
            .align_items(Alignment::Center)
            .push(Button::new("Increment").on_press(Message::IncrementPressed))
            .push(Text::new(self.value.to_string()).size(50))
            .push(Button::new("Decrement").on_press(Message::DecrementPressed));

        Virtual::new(&mut self.state, content).into()
    }
}
