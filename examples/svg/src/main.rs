use iced::{Container, Element, Length, Sandbox, Settings, Svg};

pub fn main() {
    env_logger::init();

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
        let svg = Svg::new(format!(
            "{}/resources/tiger.svg",
            env!("CARGO_MANIFEST_DIR")
        ))
        .width(Length::Fill)
        .height(Length::Fill);

        Container::new(svg)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into()
    }
}
