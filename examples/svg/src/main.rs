use iced::widget::{center, checkbox, column, container, svg};
use iced::{color, Element, Fill};

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

        let svg =
            svg(handle)
                .width(Fill)
                .height(Fill)
                .style(|_theme, _status| svg::Style {
                    color: if self.apply_color_filter {
                        Some(color!(0x0000ff))
                    } else {
                        None
                    },
                });

        let apply_color_filter =
            checkbox("Apply a color filter", self.apply_color_filter)
                .on_toggle(Message::ToggleColorFilter);

        center(
            column![svg, container(apply_color_filter).center_x(Fill)]
                .spacing(20)
                .height(Fill),
        )
        .padding(20)
        .into()
    }
}
