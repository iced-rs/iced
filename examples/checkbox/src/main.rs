use iced::widget::{checkbox, column, container};
use iced::{Element, Font, Length, Sandbox, Settings};

const ICON_FONT: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("../fonts/icons.ttf"),
};

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
        String::from("Checkbox - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::DefaultChecked(value) => self.default_checkbox = value,
            Message::CustomChecked(value) => self.custom_checkbox = value,
        }
    }

    fn view(&self) -> Element<Message> {
        let default_checkbox =
            checkbox("Default", self.default_checkbox, Message::DefaultChecked);
        let custom_checkbox =
            checkbox("Custom", self.custom_checkbox, Message::CustomChecked)
                .icon(checkbox::Icon {
                    font: ICON_FONT,
                    code_point: '\u{e901}',
                    size: None,
                });

        let content = column![default_checkbox, custom_checkbox].spacing(22);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
