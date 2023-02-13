use iced::widget::{checkbox, column, container, text_input};
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
    value: String,
    is_showing_handle: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Changed(String),
    ToggleHandle(bool),
}

impl Sandbox for Example {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("Text Input - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Changed(value) => self.value = value,
            Message::ToggleHandle(_) => {
                self.is_showing_handle = !self.is_showing_handle
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let checkbox =
            checkbox("Handle", self.is_showing_handle, Message::ToggleHandle)
                .spacing(5)
                .text_size(16);

        let mut text_input =
            text_input("Placeholder", self.value.as_str(), Message::Changed);

        if self.is_showing_handle {
            text_input = text_input.handle(text_input::Handle {
                font: ICON_FONT,
                text: String::from('\u{e900}'),
                size: Some(18),
                position: text_input::HandlePosition::Right,
            });
        }

        let content = column!["What is blazing fast?", text_input, checkbox]
            .width(Length::Units(200))
            .spacing(10);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::default()
    }

    fn style(&self) -> iced::theme::Application {
        iced::theme::Application::default()
    }

    fn scale_factor(&self) -> f64 {
        1.0
    }

    fn run(settings: Settings<()>) -> Result<(), iced::Error>
    where
        Self: 'static + Sized,
    {
        <Self as iced::Application>::run(settings)
    }
}
