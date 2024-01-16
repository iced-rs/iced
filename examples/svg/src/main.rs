use iced::theme;
use iced::widget::{checkbox, column, container, svg};
use iced::{color, Element, Length, Sandbox, Settings};

pub fn main() -> iced::Result {
    Tiger::run(Settings::default())
}

#[derive(Debug, Default)]
struct Tiger {
    apply_color_filter: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    ToggleColorFilter(bool),
}

impl Sandbox for Tiger {
    type Message = Message;

    fn new() -> Self {
        Tiger::default()
    }

    fn title(&self) -> String {
        String::from("SVG - Iced")
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::ToggleColorFilter(apply_color_filter) => {
                self.apply_color_filter = apply_color_filter;
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let handle = svg::Handle::from_path(format!(
            "{}/resources/tiger.svg",
            env!("CARGO_MANIFEST_DIR")
        ));

        let svg = svg(handle).width(Length::Fill).height(Length::Fill).style(
            if self.apply_color_filter {
                theme::Svg::custom_fn(|_theme| svg::Appearance {
                    color: Some(color!(0x0000ff)),
                })
            } else {
                theme::Svg::Default
            },
        );

        let apply_color_filter = checkbox(
            "Apply a color filter",
            self.apply_color_filter,
            Message::ToggleColorFilter,
        );

        container(
            column![
                svg,
                container(apply_color_filter).width(Length::Fill).center_x()
            ]
            .spacing(20)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .center_x()
        .center_y()
        .into()
    }
}
