use iced::widget::{checkbox, column, container, svg};
use iced::{color, Element, Length};

pub fn main() -> iced::Result {
    iced::run("SVG - Iced", Tiger::update, Tiger::view)
}

#[derive(Debug, Default)]
struct Tiger {
    apply_color_filter: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    ToggleColorFilter(bool),
}

impl Tiger {
    fn update(&mut self, message: Message) {
        match message {
            Message::ToggleColorFilter(apply_color_filter) => {
                self.apply_color_filter = apply_color_filter;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let handle = svg::Handle::from_path(format!(
            "{}/resources/tiger.svg",
            env!("CARGO_MANIFEST_DIR")
        ));

        let svg = svg(handle).width(Length::Fill).height(Length::Fill).style(
            |_theme, _status| svg::Style {
                color: if self.apply_color_filter {
                    Some(color!(0x0000ff))
                } else {
                    None
                },
            },
        );

        let apply_color_filter =
            checkbox("Apply a color filter", self.apply_color_filter)
                .on_toggle(Message::ToggleColorFilter);

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
