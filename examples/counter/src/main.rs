use iced::widget::{button, column, text, container, row};
use iced::{Alignment, Element, Sandbox, Settings, Length, Ease};
use iced::widget::row::Keyframe as rowk;
use iced::animation::Animation;
use iced::animation;
use iced::Color;

use lazy_static::lazy_static;
use std::time::Duration;

lazy_static! {
    static ref TEST_ANIMATION: container::Id = container::Id::unique();
}

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
                println!("############ value = {}", self.value);
                self.value += 1;
            }
            Message::DecrementPressed => {
                self.value -= 1;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let content: Element<_> = column![
            button("Increment").on_press(Message::IncrementPressed).width(Length::Units(100)),
            text(self.value).size(50),
            // button("Decrement").on_press(Message::DecrementPressed).animate_width(Length::Units(10), Length::Units(100), 1000, Ease::Linear)
            row![button("Decrement").on_press(Message::DecrementPressed).height(Length::Fill).width(Length::Units(100))]
                .height(Length::Units(10))
                .width(Length::Units(10))
                .animation(Animation::new()
                           .push(rowk::new(Duration::from_secs(3))
                                 .height(Length::Units(100))
                                 .width(Length::Units(100))
                           )
                           .push(rowk::new(Duration::from_secs(3))
                                 .height(Length::Units(10))
                                 .width(Length::Units(10))
                           )
                           .push(rowk::new(Duration::from_secs(3))
                                 .height(Length::Units(100))
                                 .width(Length::Units(100))
                           )
                ),
        ]
        .padding(20)
        .align_items(Alignment::Center)
        .into();
        content
    }
}
