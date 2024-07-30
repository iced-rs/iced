use iced::alignment::Vertical;
use iced::keyboard::key::Named;
use iced::widget::{center, row, text_input};
use iced::{widget, Element, Subscription, Task};

fn main() -> iced::Result {
    iced::application("Text Inputs", TextInputs::update, TextInputs::view)
        .subscription(TextInputs::keys)
        .run()
}

#[derive(Default)]
struct TextInputs {
    left_input: String,
    right_input: String,
}

#[derive(Debug, Clone)]
enum Message {
    Focus,
    LeftInputChanged(String),
    LeftInputSubmitted,
    RightInputChanged(String),
    RightInputSubmitted,
}

impl TextInputs {
    fn view(&self) -> Element<Message> {
        let left_input = text_input("Left", &self.left_input)
            .width(200)
            .on_input(Message::LeftInputChanged)
            .on_submit(Message::LeftInputSubmitted);

        let right_input = text_input("Right", &self.right_input)
            .width(200)
            .alignment(text_input::Alignment::Right)
            .on_input(Message::RightInputChanged)
            .on_submit(Message::RightInputSubmitted);

        let content = row![left_input, right_input,]
            .spacing(20)
            .align_y(Vertical::Center);

        center(content).into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Focus => {
                return widget::focus_next();
            }
            Message::LeftInputChanged(input) => {
                self.left_input = input;
            }
            Message::RightInputChanged(input) => {
                self.right_input = input;
            }
            Message::LeftInputSubmitted => {
                self.left_input = String::new();
            }
            Message::RightInputSubmitted => {
                self.right_input = String::new();
            }
        }

        Task::none()
    }

    fn keys(&self) -> Subscription<Message> {
        iced::keyboard::on_key_press(|key, _| {
            if let iced::keyboard::Key::Named(Named::Tab) = key {
                Some(Message::Focus)
            } else {
                None
            }
        })
    }
}
