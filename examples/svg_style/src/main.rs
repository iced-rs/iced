use iced::widget::{container, svg};
use iced::{Color, Element, Length, Sandbox, Settings};
use iced_style::svg::Appearance;
use iced_style::theme::{self, Theme};

pub fn main() -> iced::Result {
    SvgStyleExample::run(Settings::default())
}

struct SvgStyleExample;

impl Sandbox for SvgStyleExample {
    type Message = ();

    fn new() -> Self {
        SvgStyleExample
    }

    fn theme(&self) -> Theme {
        Theme::Light
    }

    fn title(&self) -> String {
        String::from("SVG - Iced")
    }

    fn update(&mut self, _message: ()) {}

    fn view(&self) -> Element<()> {
        let svg1: Element<_> = svg(svg::Handle::from_path(format!(
            "{}/resources/go-next-symbolic.svg",
            env!("CARGO_MANIFEST_DIR")
        )))
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

        let svg2: Element<_> = svg(svg::Handle::from_path(format!(
            "{}/resources/go-next-symbolic.svg",
            env!("CARGO_MANIFEST_DIR")
        )))
        .style(theme::Svg::Custom(|_theme| Appearance {
            fill: Some(Color {
                r: 0.0,
                g: 0.28627452,
                b: 0.42745098,
                a: 1.0,
            }),
        }))
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

        let svg3: Element<_> = svg(svg::Handle::from_path(format!(
            "{}/resources/go-next-symbolic.svg",
            env!("CARGO_MANIFEST_DIR")
        )))
        .style(theme::Svg::Custom(|_theme| Appearance {
            fill: Some(Color {
                r: 0.5803922,
                g: 0.92156863,
                b: 0.92156863,
                a: 1.0,
            }),
        }))
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

        container(iced::widget::row!(svg1, svg2, svg3))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into()
    }
}
