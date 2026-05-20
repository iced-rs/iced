//! Example showcasing new visual effects: radial gradients and inset shadows.
//!
//! This example demonstrates:
//! - Radial gradient backgrounds (elliptical and circular)
//! - Inset shadows for inner glow/depth effects
//! - Glass-like effect combining gradients and shadows

use iced::gradient;
use iced::widget::{column, container, row, slider, space, text};
use iced::{Center, Color, Element, Fill, Length, Point, Shadow, Vector};

pub fn main() -> iced::Result {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt::init();

    iced::application(Effects::default, Effects::update, Effects::view)
        .title("Effects Demo")
        .run()
}

#[derive(Debug, Clone, Copy)]
struct Effects {
    // Radial gradient controls
    center_x: f32,
    center_y: f32,
    radius_x: f32,
    radius_y: f32,
    // Inset shadow controls
    shadow_blur: f32,
    shadow_offset_x: f32,
    shadow_offset_y: f32,
    shadow_opacity: f32,
}

impl Default for Effects {
    fn default() -> Self {
        Self {
            center_x: 0.5,
            center_y: 1.0,
            radius_x: 0.5,
            radius_y: 0.4,
            shadow_blur: 15.0,
            shadow_offset_x: 0.0,
            shadow_offset_y: 2.0,
            shadow_opacity: 0.15,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Message {
    CenterXChanged(f32),
    CenterYChanged(f32),
    RadiusXChanged(f32),
    RadiusYChanged(f32),
    ShadowBlurChanged(f32),
    ShadowOffsetXChanged(f32),
    ShadowOffsetYChanged(f32),
    ShadowOpacityChanged(f32),
}

impl Effects {
    fn update(&mut self, message: Message) {
        match message {
            Message::CenterXChanged(v) => self.center_x = v,
            Message::CenterYChanged(v) => self.center_y = v,
            Message::RadiusXChanged(v) => self.radius_x = v,
            Message::RadiusYChanged(v) => self.radius_y = v,
            Message::ShadowBlurChanged(v) => self.shadow_blur = v,
            Message::ShadowOffsetXChanged(v) => self.shadow_offset_x = v,
            Message::ShadowOffsetYChanged(v) => self.shadow_offset_y = v,
            Message::ShadowOpacityChanged(v) => self.shadow_opacity = v,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let center_x = self.center_x;
        let center_y = self.center_y;
        let radius_x = self.radius_x;
        let radius_y = self.radius_y;
        let shadow_blur = self.shadow_blur;
        let shadow_offset_x = self.shadow_offset_x;
        let shadow_offset_y = self.shadow_offset_y;
        let shadow_opacity = self.shadow_opacity;

        // Controls panel
        let controls = container(
            column![
                text("Radial Gradient").size(18),
                labeled_slider(
                    "Center X",
                    0.0..=1.0,
                    self.center_x,
                    Message::CenterXChanged
                ),
                labeled_slider(
                    "Center Y",
                    0.0..=1.0,
                    self.center_y,
                    Message::CenterYChanged
                ),
                labeled_slider(
                    "Radius X",
                    0.1..=2.0,
                    self.radius_x,
                    Message::RadiusXChanged
                ),
                labeled_slider(
                    "Radius Y",
                    0.1..=2.0,
                    self.radius_y,
                    Message::RadiusYChanged
                ),
                space::vertical().height(20),
                text("Inset Shadow").size(18),
                labeled_slider(
                    "Blur",
                    0.0..=50.0,
                    self.shadow_blur,
                    Message::ShadowBlurChanged
                ),
                labeled_slider(
                    "Offset X",
                    -20.0..=20.0,
                    self.shadow_offset_x,
                    Message::ShadowOffsetXChanged
                ),
                labeled_slider(
                    "Offset Y",
                    -20.0..=20.0,
                    self.shadow_offset_y,
                    Message::ShadowOffsetYChanged
                ),
                labeled_slider(
                    "Opacity",
                    0.0..=1.0,
                    self.shadow_opacity,
                    Message::ShadowOpacityChanged
                ),
            ]
            .spacing(8)
            .padding(16),
        )
        .style(|_| container::Style {
            background: Some(Color::from_rgb(0.15, 0.15, 0.18).into()),
            ..Default::default()
        })
        .width(320)
        .height(Fill);

        // Demo boxes
        let radial_demo = container(
            column![
                text("Radial Gradient").size(14).color(Color::WHITE),
                text("Elliptical gradient from center")
                    .size(12)
                    .color(Color::from_rgba(1.0, 1.0, 1.0, 0.7)),
            ]
            .spacing(4)
            .align_x(Center),
        )
        .padding(20)
        .width(200)
        .height(200)
        .style(move |_| {
            let gradient =
                gradient::Radial::elliptical(Point::new(center_x, center_y), radius_x, radius_y)
                    .add_stop(0.0, Color::from_rgba(1.0, 1.0, 1.0, 0.3))
                    .add_stop(1.0, Color::from_rgba(1.0, 1.0, 1.0, 0.0));

            container::Style {
                background: Some(iced::Gradient::from(gradient).into()),
                border: iced::Border {
                    color: Color::from_rgba(1.0, 1.0, 1.0, 0.4),
                    width: 1.0,
                    radius: 12.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        });

        let inset_shadow_demo = container(
            column![
                text("Inset Shadow").size(14).color(Color::WHITE),
                text("Inner shadow for depth")
                    .size(12)
                    .color(Color::from_rgba(1.0, 1.0, 1.0, 0.7)),
            ]
            .spacing(4)
            .align_x(Center),
        )
        .padding(20)
        .width(200)
        .height(200)
        .style(move |_| container::Style {
            background: Some(Color::from_rgba(0.2, 0.2, 0.25, 0.8).into()),
            border: iced::Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.3),
                width: 1.0,
                radius: 12.0.into(),
                ..Default::default()
            },
            shadow: Shadow::inset(
                Color::from_rgba(0.0, 0.0, 0.0, shadow_opacity),
                Vector::new(shadow_offset_x, shadow_offset_y),
                shadow_blur,
            ),
            ..Default::default()
        });

        let glass_demo = container(
            column![
                text("Glass Effect").size(14).color(Color::WHITE),
                text("Radial + Inset Shadow")
                    .size(12)
                    .color(Color::from_rgba(1.0, 1.0, 1.0, 0.7)),
            ]
            .spacing(4)
            .align_x(Center),
        )
        .padding(20)
        .width(200)
        .height(200)
        .style(move |_| {
            let gradient = gradient::Radial::elliptical(Point::new(0.47, 1.0), 0.42, 0.33)
                .add_stop(0.0, Color::from_rgba(1.0, 1.0, 1.0, 0.2))
                .add_stop(1.0, Color::from_rgba(1.0, 1.0, 1.0, 0.0));

            container::Style {
                background: Some(iced::Gradient::from(gradient).into()),
                border: iced::Border {
                    color: Color::from_rgba(1.0, 1.0, 1.0, 0.4),
                    width: 1.0,
                    radius: 12.0.into(),
                    ..Default::default()
                },
                shadow: Shadow::inset(
                    Color::from_rgba(0.0, 0.0, 0.0, 0.10),
                    Vector::new(0.0, 2.0),
                    15.0,
                ),
                ..Default::default()
            }
        });

        let outer_shadow_demo = container(
            column![
                text("Outer Shadow").size(14).color(Color::WHITE),
                text("Traditional drop shadow")
                    .size(12)
                    .color(Color::from_rgba(1.0, 1.0, 1.0, 0.7)),
            ]
            .spacing(4)
            .align_x(Center),
        )
        .padding(20)
        .width(200)
        .height(200)
        .style(move |_| container::Style {
            background: Some(Color::from_rgba(0.25, 0.25, 0.3, 0.9).into()),
            border: iced::Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.2),
                width: 1.0,
                radius: 12.0.into(),
                ..Default::default()
            },
            shadow: Shadow::new(
                Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                Vector::new(4.0, 8.0),
                20.0,
            ),
            ..Default::default()
        });

        // Demo area with dark background
        let demos = container(
            column![
                row![radial_demo, inset_shadow_demo].spacing(20),
                row![glass_demo, outer_shadow_demo].spacing(20),
            ]
            .spacing(20)
            .align_x(Center),
        )
        .padding(40)
        .width(Fill)
        .height(Fill)
        .align_x(Center)
        .align_y(Center)
        .style(|_| container::Style {
            background: Some(Color::from_rgb(0.1, 0.1, 0.12).into()),
            ..Default::default()
        });

        row![controls, demos].into()
    }
}

fn labeled_slider<'a>(
    label: &'a str,
    range: std::ops::RangeInclusive<f32>,
    value: f32,
    on_change: impl Fn(f32) -> Message + 'a,
) -> Element<'a, Message> {
    row![
        text(label).width(80).color(Color::from_rgb(0.8, 0.8, 0.8)),
        slider(range, value, on_change)
            .step(0.01)
            .width(Length::Fixed(160.0)),
        text(format!("{:.2}", value))
            .width(50)
            .color(Color::from_rgb(0.6, 0.6, 0.6)),
    ]
    .spacing(8)
    .align_y(Center)
    .into()
}
