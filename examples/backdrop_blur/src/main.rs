//! Backdrop Blur Example
//!
//! Demonstrates the backdrop blur effect for glassmorphism UI.

use iced::widget::{Canvas, backdrop_blur, canvas, column, container, row, slider, space, text};
use iced::{Background, Color, Element, Fill, Gradient, Point, Rectangle, Renderer, Theme, Vector};

pub fn main() -> iced::Result {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt::init();

    iced::application(App::default, App::update, App::view)
        .title("Backdrop Blur Demo")
        .run()
}

#[derive(Debug, Clone, Copy)]
struct App {
    blur_radius: f32,
}

impl Default for App {
    fn default() -> Self {
        Self { blur_radius: 20.0 }
    }
}

#[derive(Debug, Clone, Copy)]
enum Message {
    BlurRadiusChanged(f32),
}

// A colorful pattern background for testing blur
struct PatternBackground;

impl canvas::Program<Message> for PatternBackground {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // Draw colorful circles as a test pattern
        let colors = [
            Color::from_rgb(1.0, 0.2, 0.2), // Red
            Color::from_rgb(0.2, 1.0, 0.2), // Green
            Color::from_rgb(0.2, 0.2, 1.0), // Blue
            Color::from_rgb(1.0, 1.0, 0.2), // Yellow
            Color::from_rgb(1.0, 0.2, 1.0), // Magenta
            Color::from_rgb(0.2, 1.0, 1.0), // Cyan
            Color::from_rgb(1.0, 0.5, 0.0), // Orange
            Color::from_rgb(0.5, 0.0, 1.0), // Purple
        ];

        // Grid of circles
        let circle_size = 60.0;
        let spacing = 80.0;

        for row in 0..20 {
            for col in 0..20 {
                let x = col as f32 * spacing + 40.0;
                let y = row as f32 * spacing + 40.0;
                let color_idx = (row + col) % colors.len();

                frame.fill(
                    &canvas::Path::circle(Point::new(x, y), circle_size / 2.0),
                    colors[color_idx],
                );
            }
        }

        // Also draw some text-like rectangles
        for i in 0..15 {
            let y = i as f32 * 100.0 + 20.0;
            frame.fill_rectangle(
                Point::new(50.0, y),
                iced::Size::new(300.0, 20.0),
                Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            );
        }

        vec![frame.into_geometry()]
    }
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::BlurRadiusChanged(radius) => {
                self.blur_radius = radius;
                println!("Blur radius changed to: {}", radius);
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // Colorful pattern background using canvas
        let pattern_bg: Canvas<PatternBackground, Message, Theme, Renderer> =
            canvas(PatternBackground).width(Fill).height(Fill);

        // Glass card with backdrop blur
        // CSS styling:
        // background: radial-gradient(41.71% 33.33% at 47.15% 100%, rgba(255, 255, 255, 0.20) 0%, rgba(255, 255, 255, 0.00) 100%),
        //             linear-gradient(0deg, rgba(255, 255, 255, 0.20) 0%, rgba(255, 255, 255, 0.20) 100%);
        // box-shadow: 0 2px 15px 0 rgba(0, 0, 0, 0.10) inset, 0 4px 10px 0 rgba(0, 0, 0, 0.05);
        // backdrop-filter: blur(25px);

        let content = column![
            text("Glassmorphism Effect").size(24).color(Color::WHITE),
            space::vertical().height(10),
            text("This card should blur the background")
                .size(14)
                .color(Color::from_rgb(0.9, 0.9, 0.9)),
            space::vertical().height(20),
            row![
                text("Blur Radius:").size(14).color(Color::WHITE),
                slider(0.0..=50.0, self.blur_radius, Message::BlurRadiusChanged)
                    .width(200.0)
                    .step(1.0),
                text(format!("{:.0}px", self.blur_radius))
                    .size(14)
                    .color(Color::WHITE),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(5)
        .padding(30);

        // Inner container: Gradient background + inset shadow
        // CSS: radial-gradient + 0 2px 15px 0 rgba(0, 0, 0, 0.10) inset
        let inner_card = container(content).width(Fill).style(|_| {
            let radial = Gradient::Radial(
                iced::gradient::Radial::elliptical(Point::new(0.4715, 1.0), 0.4171, 0.3333)
                    .add_stop(0.0, Color::from_rgba(1.0, 1.0, 1.0, 0.20))
                    .add_stop(1.0, Color::from_rgba(1.0, 1.0, 1.0, 0.0)),
            );

            container::Style {
                background: Some(Background::Gradient(radial)),
                border: iced::Border {
                    color: Color::from_rgba(1.0, 1.0, 1.0, 1.0),
                    width: 1.0,
                    radius: 20.0.into(),
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.9),
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 15.0,
                    spread_radius: 0.0,
                    inset: true,
                },
                ..Default::default()
            }
        });

        // Outer container: Outset shadow (drop shadow)
        // CSS: 0 4px 10px 0 rgba(0, 0, 0, 0.05)
        let glass_card =
            backdrop_blur(
                container(inner_card)
                    .width(450.0)
                    .style(|_| container::Style {
                        background: None,
                        border: iced::Border {
                            color: Color::TRANSPARENT,
                            width: 0.0,
                            radius: 20.0.into(),
                        },
                        shadow: iced::Shadow {
                            color: Color::from_rgba(0.0, 0.0, 0.0, 0.9),
                            offset: Vector::new(0.0, 4.0),
                            blur_radius: 10.0,
                            spread_radius: 0.0,
                            inset: false,
                        },
                        ..Default::default()
                    }),
            )
            .blur_radius(self.blur_radius)
            .border_radius(20.0);

        // Stack everything
        container(
            iced::widget::Stack::new()
                .push(pattern_bg)
                .push(container(glass_card).center(Fill).padding(40)),
        )
        .width(Fill)
        .height(Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.15))),
            ..Default::default()
        })
        .into()
    }
}
