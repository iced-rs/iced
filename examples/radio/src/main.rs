use iced::widget::{column, container, Radio};
use iced::{Element, Length, Sandbox, Settings};

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

#[derive(Default)]
struct Example {
    default_checkbox: bool,
    custom_checkbox: bool,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    DefaultChecked(bool),
    CustomChecked(bool),
}

impl Sandbox for Example {
    type Message = Message;

    fn new() -> Self {
        Default::default()
    }

    fn title(&self) -> String {
        String::from("Radio - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::DefaultChecked(value) => self.default_checkbox = value,
            Message::CustomChecked(value) => self.custom_checkbox = value,
        }
    }

    fn view(&self) -> Element<Message> {
        let selected_choice = Some(Choice::A);

	Radio::new(Choice::A, "This is A", selected_choice, Message::RadioSelected);
        Radio::new(Choice::B, "This is B", selected_choice, Message::RadioSelected);

        let content = column![selected_choice].spacing(22);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
