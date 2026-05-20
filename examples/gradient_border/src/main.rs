//! Gradient Border Example
//!
//! Demonstrates the `border_only` feature for rendering gradient borders
//! where the gradient fills only the border region, not the interior.

use iced::gradient::Conic;
use iced::widget::{center, checkbox, column, container, row, slider, space, text};
use iced::{Border, Color, Element, Fill, Point, Radians, color};
use tracing::info;

pub fn main() -> iced::Result {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt::init();

    info!("Starting Gradient Border Example");

    iced::application(App::default, App::update, App::view)
        .title("Gradient Border Example")
        .run()
}

#[derive(Debug, Clone, Copy)]
struct App {
    show_background: bool,
    border_width: f32,
    border_radius: f32,
    start_angle: f32,
}

impl Default for App {
    fn default() -> Self {
        info!("Creating default App state");
        Self {
            show_background: true,
            border_width: 8.0,
            border_radius: 16.0,
            start_angle: 270.0, // Start from top
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Message {
    ToggleShowBackground(bool),
    BorderWidthChanged(f32),
    BorderRadiusChanged(f32),
    StartAngleChanged(f32),
}

impl App {
    fn update(&mut self, message: Message) {
        info!(?message, "Received message");
        match message {
            Message::ToggleShowBackground(show) => {
                info!(
                    old = self.show_background,
                    new = show,
                    "Toggling show_background"
                );
                self.show_background = show;
            }
            Message::BorderWidthChanged(width) => {
                info!(
                    old = self.border_width,
                    new = width,
                    "Changing border width"
                );
                self.border_width = width;
            }
            Message::BorderRadiusChanged(radius) => {
                info!(
                    old = self.border_radius,
                    new = radius,
                    "Changing border radius"
                );
                self.border_radius = radius;
            }
            Message::StartAngleChanged(angle) => {
                info!(old = self.start_angle, new = angle, "Changing start angle");
                self.start_angle = angle;
            }
        }
        info!(state = ?self, "State after update");
    }

    fn view(&self) -> Element<'_, Message> {
        let Self {
            show_background,
            border_width,
            border_radius,
            start_angle,
        } = *self;

        info!(
            show_background,
            border_width, border_radius, start_angle, "Building view"
        );

        // Create a conic gradient for the border
        let start_radians = Radians(start_angle.to_radians());

        // Controls
        let controls = column![
            checkbox(show_background)
                .label("Show Background")
                .on_toggle(Message::ToggleShowBackground),
            row![
                text(format!("Border Width: {:.1}px", border_width)).width(150),
                slider(1.0..=20.0, border_width, Message::BorderWidthChanged),
            ]
            .spacing(10)
            .align_y(iced::Center),
            row![
                text(format!("Border Radius: {:.0}px", border_radius)).width(150),
                slider(0.0..=50.0, border_radius, Message::BorderRadiusChanged),
            ]
            .spacing(10)
            .align_y(iced::Center),
            row![
                text(format!("Start Angle: {:.0}°", start_angle)).width(150),
                slider(0.0..=360.0, start_angle, Message::StartAngleChanged),
            ]
            .spacing(10)
            .align_y(iced::Center),
        ]
        .spacing(15);

        // First example - border_only: true with optional opaque child
        let inner_content_1: Element<'_, Message> = if show_background {
            container(text("border_only: true").size(16).color(Color::WHITE))
                .width(Fill)
                .height(Fill)
                .center(Fill)
                .style(|_theme| container::Style {
                    background: Some(color!(0x2a2a3a).into()),
                    ..Default::default()
                })
                .into()
        } else {
            container(text("border_only: true").size(16).color(Color::WHITE))
                .width(Fill)
                .height(Fill)
                .center(Fill)
                .into()
        };

        let no_bg_box = container(inner_content_1)
            .padding(border_width) // Inset children to not cover the border
            .style(move |_theme| {
                let gradient = Conic::new(Point::new(0.5, 0.5), start_radians)
                    .add_stop(0.0, color!(0x00FF00)) // Green
                    .add_stop(0.33, color!(0x0000FF)) // Blue
                    .add_stop(0.66, color!(0xFF0000)) // Red
                    .add_stop(1.0, color!(0x00FF00)); // Back to green

                container::Style {
                    background: None,
                    border: Border {
                        color: Color::WHITE,
                        width: border_width,
                        radius: border_radius.into(),
                        ..Default::default()
                    },
                    border_only: true,
                    ..Default::default()
                }
                .background(gradient)
            })
            .width(200)
            .height(150);

        // Second example - border_only: false with optional opaque child
        let inner_content_2: Element<'_, Message> = if show_background {
            container(text("border_only: false").size(16).color(Color::WHITE))
                .width(Fill)
                .height(Fill)
                .center(Fill)
                .style(|_theme| container::Style {
                    background: Some(color!(0x2a2a3a).into()),
                    ..Default::default()
                })
                .into()
        } else {
            container(text("border_only: false").size(16).color(Color::WHITE))
                .width(Fill)
                .height(Fill)
                .center(Fill)
                .into()
        };

        let solid_bg_box = container(inner_content_2)
            .padding(border_width) // Inset children to not cover the border
            .style(move |_theme| {
                let gradient = Conic::new(Point::new(0.5, 0.5), start_radians)
                    .add_stop(0.0, color!(0xFF6B6B)) // Coral
                    .add_stop(0.5, color!(0x4ECDC4)) // Teal
                    .add_stop(1.0, color!(0xFF6B6B)); // Back to coral

                container::Style {
                    background: None,
                    border: Border {
                        color: Color::WHITE,
                        width: border_width,
                        radius: border_radius.into(),
                        ..Default::default()
                    },
                    border_only: false, // Full gradient fill
                    ..Default::default()
                }
                .background(gradient)
            })
            .width(200)
            .height(150);

        let examples_row = row![no_bg_box, space().width(20), solid_bg_box].align_y(iced::Center);

        let content = column![
            text("Gradient Border Example").size(28).color(Color::WHITE),
            space().height(20),
            controls,
            space().height(30),
            text("Examples:").size(20).color(Color::WHITE),
            space().height(10),
            examples_row,
        ]
        .align_x(iced::Center);

        center(content).padding(20).into()
    }
}
