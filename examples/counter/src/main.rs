use iced::widget::{button, column, text};
use iced::{Alignment, Element, Sandbox, Settings, Length, Ease};

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
        column![
            button("Increment").on_press(Message::IncrementPressed).width(Length::Units(100)),
            text(self.value).size(50),
            button("Decrement").on_press(Message::DecrementPressed).animate_width(Length::Units(10), Length::Units(100), 1000, Ease::Linear)
        ]
        .padding(20)
        .align_items(Alignment::Center)
        .into()
    }
}
