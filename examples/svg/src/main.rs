use iced::widget::{center, center_x, checkbox, column, svg};
use iced::{Element, Fill, color};

pub fn main() -> iced::Result {
    iced::run(Tiger::update, Tiger::view)
}

#[derive(Debug)]
struct Tiger {
    apply_color_filter: bool,
    handle: svg::Handle,
}

impl Default for Tiger {
    fn default() -> Self {
        Self {
            apply_color_filter: false,
            handle: svg::Handle::from_path(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/resources/tiger.svg",
            )),
        }
    }
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

    fn view(&self) -> Element<'_, Message> {
        let svg = svg(self.handle.clone()).width(Fill).height(Fill).style(
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

        center(column![svg, center_x(apply_color_filter)].spacing(20))
            .padding(20)
            .into()
    }
}
