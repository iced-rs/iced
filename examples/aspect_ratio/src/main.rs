use iced::widget::{button, center, column, container, responsive, row, text};
use iced::{Background, Color, Element, Length, Size};

fn main() {
    iced::run("aspect ratio", AspectRatio::update, AspectRatio::view).unwrap();
}

#[derive(Default)]
struct AspectRatio;

impl AspectRatio {
    fn update(&mut self, _: ()) {}

    fn view(&self) -> Element<()> {
        const SPACING: f32 = 20.0;

        center(
            column![
                responsive(move |size| {
                    let row_height = 40.0;
                    let square_available = Size::new(
                        size.width,
                        size.height - row_height - SPACING,
                    );
                    let square_size = square_available.aspect_ratio(1.0);

                    center(
                        column![
                            container("Green 1:1 square with buttons")
                                .style(|_| container::Style {
                                    text_color: None,
                                    background: Some(Background::Color(
                                        Color::from_rgb8(0, 100, 0)
                                    )),
                                    border: Default::default(),
                                    shadow: Default::default(),
                                })
                                .width(square_size.width)
                                .height(square_size.height),
                            row![
                                button(center(text("<"))).width(Length::Fill),
                                button(center(text(">"))).width(Length::Fill),
                            ]
                            .spacing(SPACING)
                            .width(square_size.width)
                            .height(row_height)
                        ]
                        .spacing(SPACING),
                    )
                    .style(|_| container::Style {
                        text_color: None,
                        background: Some(Background::Color(Color::from_rgb8(
                            0, 70, 0,
                        ))),
                        border: Default::default(),
                        shadow: Default::default(),
                    })
                    .into()
                }),
                responsive(move |size| {
                    let ratioed = size.aspect_ratio(16.0 / 9.0);

                    center(
                        container("Blue 16:9 container")
                            .style(|_| container::Style {
                                text_color: None,
                                background: Some(Background::Color(
                                    Color::from_rgb8(0, 0, 150),
                                )),
                                border: Default::default(),
                                shadow: Default::default(),
                            })
                            .width(ratioed.width)
                            .height(ratioed.height),
                    )
                    .style(|_| container::Style {
                        text_color: None,
                        background: Some(Background::Color(Color::from_rgb8(
                            0, 0, 70,
                        ))),
                        border: Default::default(),
                        shadow: Default::default(),
                    })
                    .into()
                }),
            ]
            .spacing(SPACING),
        )
        .padding(SPACING)
        .into()
    }
}
