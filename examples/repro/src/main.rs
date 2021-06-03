use iced::{
    button, canvas, Align, Button, Column, Length, Row, Sandbox, Settings,
};

#[derive(Debug, Clone, Copy)]
enum Message {
    Foo,
}

pub fn main() -> iced::Result {
    Repro::run(Settings::default())
}

#[derive(Default)]
struct Canvas {
    #[allow(unused)]
    inner: canvas::Cache,
}

impl canvas::Program<Message> for Canvas {
    fn draw(
        &self,
        _bounds: iced::Rectangle,
        _cursor: canvas::Cursor,
    ) -> Vec<canvas::Geometry> {
        vec![]
    }
}

struct Repro {
    one: iced::Svg,
    one_state: button::State,
    two: iced::Svg,
    two_state: button::State,
    three: iced::Svg,
    canvas: Canvas,
}

pub(crate) mod style {
    use iced::{button, container, Background, Color};

    pub struct TextContainer;

    impl container::StyleSheet for TextContainer {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(Background::Color(Color::WHITE)),
                text_color: Some(Color::BLACK),
                border_radius: 16.0,
                border_width: 4.0,
                ..container::Style::default()
            }
        }
    }

    pub struct ButtonStyle;

    impl button::StyleSheet for ButtonStyle {
        fn active(&self) -> button::Style {
            button::Style {
                background: None,
                border_radius: 1.0,
                text_color: Color::WHITE,
                ..button::Style::default()
            }
        }
    }
}

impl Sandbox for Repro {
    type Message = Message;

    fn new() -> Self {
        Repro {
            one: iced::Svg::from_path(format!(
                "{}/src/one.svg",
                env!("CARGO_MANIFEST_DIR")
            )),
            one_state: Default::default(),
            two: iced::Svg::from_path(format!(
                "{}/src/two.svg",
                env!("CARGO_MANIFEST_DIR")
            )),
            two_state: Default::default(),
            three: iced::Svg::from_path(format!(
                "{}/src/three.svg",
                env!("CARGO_MANIFEST_DIR")
            )),
            canvas: Canvas::default(),
        }
    }

    fn title(&self) -> String {
        String::from("Repro - Iced")
    }

    fn update(&mut self, _message: Message) {}
    fn view(&mut self) -> iced::Element<Message> {
        const ICON_SIZE: u16 = 64;
        let ui = Column::new()
            .push(
                Row::new()
                    .height(Length::Shrink)
                    .push(
                        Column::new()
                            .width(Length::FillPortion(2))
                            .align_items(Align::Start)
                            .push(
                                Row::new().padding(10).spacing(10).push(
                                    Button::new(
                                        &mut self.one_state,
                                        self.one
                                            .clone()
                                            .width(Length::Units(ICON_SIZE))
                                            .height(Length::Units(ICON_SIZE)),
                                    )
                                    .height(Length::Shrink)
                                    .width(Length::Shrink)
                                    .style(style::ButtonStyle)
                                    .on_press(Message::Foo),
                                ),
                            ),
                    )
                    .push(
                        Column::new()
                            .width(Length::FillPortion(2))
                            .align_items(Align::End)
                            .push(
                                Row::new()
                                    .padding(10)
                                    .spacing(10)
                                    .push(
                                        Button::new(
                                            &mut self.two_state,
                                            self.two
                                                .clone()
                                                .width(Length::Units(ICON_SIZE))
                                                .height(Length::Units(
                                                    ICON_SIZE,
                                                )),
                                        )
                                        .height(Length::Shrink)
                                        .width(Length::Shrink)
                                        .style(style::ButtonStyle)
                                        .on_press(Message::Foo),
                                    )
                                    .push(
                                        self.three
                                            .clone()
                                            .width(Length::Units(ICON_SIZE))
                                            .height(Length::Units(ICON_SIZE)),
                                    ),
                            ),
                    ),
            )
            .push(
                canvas::Canvas::new(&mut self.canvas)
                    .width(Length::Fill)
                    .height(Length::Fill),
            );
        let content: iced::Element<_> = ui.into();
        content.explain(iced::Color::WHITE)
    }
}
