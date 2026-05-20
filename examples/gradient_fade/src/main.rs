//! Gradient Fade Widget Example
//!
//! This example demonstrates the `gradient_fade` widget which applies
//! smooth opacity transitions at widget edges. Useful for scrollable
//! content that should fade out at the boundaries.

use iced::widget::{
    button, checkbox, column, container, gradient_fade, image, row, scrollable, slider, svg, text,
    FadeEdge, Space,
};
use iced::{Background, Color, Element, Length, Theme};

pub fn main() -> iced::Result {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt::init();

    iced::application(App::new, App::update, App::view)
        .title("Gradient Fade Widget")
        .theme(App::theme)
        .run()
}

struct App {
    edge: FadeEdge,
    fade_height: f32,
    use_custom_stops: bool,
    custom_start: f32,
    custom_end: f32,
}

#[derive(Debug, Clone)]
enum Message {
    SetEdge(FadeEdge),
    SetFadeHeight(f32),
    ToggleCustomStops(bool),
    SetCustomStart(f32),
    SetCustomEnd(f32),
}

impl App {
    fn new() -> Self {
        Self {
            edge: FadeEdge::Bottom,
            fade_height: 80.0,
            use_custom_stops: false,
            custom_start: 0.7,
            custom_end: 1.0,
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::SetEdge(edge) => {
                self.edge = edge;
            }
            Message::SetFadeHeight(height) => {
                self.fade_height = height;
            }
            Message::ToggleCustomStops(enabled) => {
                self.use_custom_stops = enabled;
            }
            Message::SetCustomStart(start) => {
                self.custom_start = start;
            }
            Message::SetCustomEnd(end) => {
                self.custom_end = end;
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // Edge selection buttons
        let edge_buttons = row![
            edge_button("Bottom", FadeEdge::Bottom, self.edge),
            edge_button("Top", FadeEdge::Top, self.edge),
            edge_button("Left", FadeEdge::Left, self.edge),
            edge_button("Right", FadeEdge::Right, self.edge),
            edge_button("Vertical", FadeEdge::Vertical, self.edge),
            edge_button("Horizontal", FadeEdge::Horizontal, self.edge),
        ]
        .spacing(10);

        // Fade height slider
        let height_controls = row![
            text("Fade Height:").size(14),
            slider(20.0..=150.0, self.fade_height, Message::SetFadeHeight)
                .width(200.0)
                .step(5.0),
            text(format!("{:.0}px", self.fade_height)).size(14),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center);

        // Custom stops controls
        let custom_stops_toggle = checkbox(self.use_custom_stops)
            .label("Use Custom Stops")
            .on_toggle(Message::ToggleCustomStops);

        let custom_stops_controls = if self.use_custom_stops {
            row![
                text("Start:").size(12),
                slider(0.0..=1.0, self.custom_start, Message::SetCustomStart)
                    .width(100.0)
                    .step(0.05),
                text(format!("{:.0}%", self.custom_start * 100.0)).size(12),
                Space::new().width(20.0),
                text("End:").size(12),
                slider(0.0..=1.0, self.custom_end, Message::SetCustomEnd)
                    .width(100.0)
                    .step(0.05),
                text(format!("{:.0}%", self.custom_end * 100.0)).size(12),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center)
        } else {
            row![].spacing(0)
        };

        // Create scrollable content
        let items: Vec<Element<'_, Message>> = (1..=25)
            .map(|i| {
                container(
                    row![
                        // Colored square placeholder
                        container(Space::new())
                            .width(36.0)
                            .height(36.0)
                            .style(move |_: &Theme| container::Style {
                                background: Some(Background::Color(Color::from_rgb(
                                    0.2 + (i as f32 * 0.025),
                                    0.5,
                                    0.8 - (i as f32 * 0.02),
                                ))),
                                border: iced::Border {
                                    radius: 6.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }),
                        column![
                            text(format!("List Item {i}")).size(14),
                            text("Content with gradient fade at edges")
                                .size(11)
                                .color(Color::from_rgb(0.6, 0.6, 0.6)),
                        ]
                        .spacing(2),
                    ]
                    .spacing(12)
                    .align_y(iced::Alignment::Center),
                )
                .padding(8)
                .style(|_: &Theme| container::Style {
                    background: Some(Background::Color(Color::from_rgba(0.3, 0.3, 0.35, 0.5))),
                    border: iced::Border {
                        radius: 8.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .into()
            })
            .collect();

        let scrollable_content = scrollable(column(items).spacing(6).padding(8))
            .width(Length::Fixed(350.0))
            .height(Length::Fixed(280.0));

        // Apply gradient fade to the scrollable content
        let faded_content = if self.use_custom_stops {
            gradient_fade(scrollable_content)
                .edge(self.edge)
                .stops(self.custom_start, self.custom_end)
        } else {
            gradient_fade(scrollable_content)
                .edge(self.edge)
                .height(self.fade_height)
        };

        let demo = container(faded_content).style(|_: &Theme| container::Style {
            border: iced::Border {
                color: Color::from_rgb(0.4, 0.4, 0.4),
                width: 1.0,
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        // Side panel with additional demo
        let horizontal_content = row((1..=15)
            .map(|i| {
                container(
                    column![
                        text(format!("{i}")).size(16),
                        container(Space::new())
                            .width(50.0)
                            .height(50.0)
                            .style(move |_: &Theme| container::Style {
                                background: Some(Background::Color(Color::from_rgb(
                                    0.8 - (i as f32 * 0.04),
                                    0.4 + (i as f32 * 0.02),
                                    0.3,
                                ))),
                                border: iced::Border {
                                    radius: 25.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }),
                    ]
                    .spacing(4)
                    .align_x(iced::Alignment::Center),
                )
                .padding(8)
                .into()
            })
            .collect::<Vec<_>>())
        .spacing(8);

        let horizontal_scroll = scrollable(horizontal_content)
            .direction(scrollable::Direction::Horizontal(
                scrollable::Scrollbar::default(),
            ))
            .width(Length::Fixed(300.0));

        let horizontal_faded = gradient_fade(horizontal_scroll)
            .edge(FadeEdge::Horizontal)
            .height(60.0);

        let horizontal_demo = container(horizontal_faded).style(|_: &Theme| container::Style {
            border: iced::Border {
                color: Color::from_rgb(0.4, 0.4, 0.4),
                width: 1.0,
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        // Image and SVG demo list
        let image_items: Vec<Element<'_, Message>> = (1..=15)
            .map(|i| {
                let media: Element<'_, Message> = if i % 2 == 0 {
                    // Even items: SVG
                    svg(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/tiger.svg"))
                        .width(40.0)
                        .height(40.0)
                        .into()
                } else {
                    // Odd items: Image
                    image(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/ferris.png"))
                        .width(40.0)
                        .height(40.0)
                        .into()
                };

                container(
                    row![
                        media,
                        column![
                            text(format!(
                                "Item {i} ({})",
                                if i % 2 == 0 { "SVG" } else { "Image" }
                            ))
                            .size(14),
                            text("Testing image/svg rendering")
                                .size(11)
                                .color(Color::from_rgb(0.6, 0.6, 0.6)),
                        ]
                        .spacing(2),
                    ]
                    .spacing(12)
                    .align_y(iced::Alignment::Center),
                )
                .padding(8)
                .style(|_: &Theme| container::Style {
                    background: Some(Background::Color(Color::from_rgba(0.3, 0.3, 0.35, 0.5))),
                    border: iced::Border {
                        radius: 8.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .into()
            })
            .collect();

        let image_scrollable = scrollable(column(image_items).spacing(6).padding(8))
            .width(Length::Fixed(280.0))
            .height(Length::Fixed(280.0));

        let image_faded = gradient_fade(image_scrollable)
            .edge(FadeEdge::Vertical)
            .height(60.0);

        let image_demo = container(image_faded).style(|_: &Theme| container::Style {
            border: iced::Border {
                color: Color::from_rgb(0.4, 0.4, 0.4),
                width: 1.0,
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        let demos = row![
            column![text("Vertical Scroll").size(14), demo,].spacing(8),
            column![
                text("Horizontal Scroll (always both edges)").size(14),
                horizontal_demo,
            ]
            .spacing(8),
            column![text("Images & SVGs").size(14), image_demo,].spacing(8),
        ]
        .spacing(20);

        column![
            text("Gradient Fade Widget Demo").size(24),
            text("The gradient_fade widget applies smooth opacity transitions at edges")
                .size(14)
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
            Space::new().height(15.0),
            text("Select fade edge for left demo:").size(12),
            edge_buttons,
            height_controls,
            custom_stops_toggle,
            custom_stops_controls,
            Space::new().height(10.0),
            demos,
        ]
        .spacing(10)
        .padding(20)
        .into()
    }
}

fn edge_button(label: &str, edge: FadeEdge, current: FadeEdge) -> Element<'_, Message> {
    let btn = button(text(label).size(12));
    if edge == current {
        btn.style(button::primary).into()
    } else {
        btn.on_press(Message::SetEdge(edge))
            .style(button::secondary)
            .into()
    }
}
