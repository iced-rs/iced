use iced::{Container, Element, Length, Sandbox, Settings};

pub fn main() {
    Tiger::run(Settings::default())
}

#[derive(Default)]
struct Tiger;

impl Sandbox for Tiger {
    type Message = ();

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("SVG - Iced")
    }

    fn update(&mut self, _message: ()) {}

    fn view(&mut self) -> Element<()> {
        #[cfg(feature = "svg")]
        let content = {
            use iced::{Column, Svg};

            Column::new().padding(20).push(
                Svg::new(format!(
                    "{}/examples/resources/tiger.svg",
                    env!("CARGO_MANIFEST_DIR")
                ))
                .width(Length::Fill)
                .height(Length::Fill),
            )
        };

        #[cfg(not(feature = "svg"))]
        let content = {
            use iced::{HorizontalAlignment, Text};

            Text::new("You need to enable the `svg` feature!")
                .horizontal_alignment(HorizontalAlignment::Center)
                .size(30)
        };

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
