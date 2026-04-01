//! This example demonstrates shadow `spread_radius` support.
//!
//! It renders several quads with different spread values, plus interactive
//! sliders so you can explore the effect at runtime.
use iced::border;
use iced::widget::{center, column, container, row, slider, text, toggler};
use iced::{Center, Color, Element, Length, Shadow, Theme, Vector};

pub fn main() -> iced::Result {
    iced::application(Example::new, Example::update, Example::view)
        .theme(Example::theme)
        .run()
}

struct Example {
    blur: f32,
    spread: f32,
    offset_x: f32,
    offset_y: f32,
    radius: f32,
    inset: bool,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    BlurChanged(f32),
    SpreadChanged(f32),
    OffsetXChanged(f32),
    OffsetYChanged(f32),
    RadiusChanged(f32),
    InsetToggled(bool),
}

impl Example {
    fn new() -> Self {
        Self {
            blur: 20.0,
            spread: 10.0,
            offset_x: 0.0,
            offset_y: 4.0,
            radius: 16.0,
            inset: false,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::BlurChanged(v) => self.blur = v,
            Message::SpreadChanged(v) => self.spread = v,
            Message::OffsetXChanged(v) => self.offset_x = v,
            Message::OffsetYChanged(v) => self.offset_y = v,
            Message::RadiusChanged(v) => self.radius = v,
            Message::InsetToggled(v) => self.inset = v,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // --- Static showcase row ------------------------------------------
        let showcase = row![
            shadow_box("No spread", 0.0, 20.0, false, 16.0),
            shadow_box("Spread 5", 5.0, 20.0, false, 16.0),
            shadow_box("Spread 15", 15.0, 20.0, false, 16.0),
            shadow_box("Spread 30", 30.0, 20.0, false, 16.0),
            shadow_box("Negative -5", -5.0, 20.0, false, 16.0),
            shadow_box("Inset 10", 10.0, 20.0, true, 16.0),
        ]
        .spacing(40)
        .align_y(Center);

        // --- Interactive controls -----------------------------------------
        let interactive_box = container("").width(200).height(200).style({
            let shadow = Shadow {
                color: Color::from_rgba(0.3, 0.5, 1.0, 0.8),
                offset: Vector::new(self.offset_x, self.offset_y),
                blur_radius: self.blur,
                spread_radius: self.spread,
                inset: self.inset,
            };
            let radius = self.radius;
            move |_t: &Theme| container::Style {
                background: Some(Color::from_rgb(0.15, 0.15, 0.2).into()),
                border: border::rounded(radius)
                    .width(1)
                    .color(Color::from_rgb(0.3, 0.3, 0.4)),
                shadow,
                ..Default::default()
            }
        });

        let controls = column![
            text!("Blur: {:.1}", self.blur),
            slider(0.0..=100.0, self.blur, Message::BlurChanged).step(0.5),
            text!("Spread: {:.1}", self.spread),
            slider(-50.0..=100.0, self.spread, Message::SpreadChanged).step(0.5),
            text!("Offset X: {:.1}", self.offset_x),
            slider(-100.0..=100.0, self.offset_x, Message::OffsetXChanged).step(0.5),
            text!("Offset Y: {:.1}", self.offset_y),
            slider(-100.0..=100.0, self.offset_y, Message::OffsetYChanged).step(0.5),
            text!("Border radius: {:.1}", self.radius),
            slider(0.0..=100.0, self.radius, Message::RadiusChanged).step(0.5),
            toggler(self.inset)
                .label("Inset shadow")
                .on_toggle(Message::InsetToggled),
        ]
        .spacing(8)
        .width(300);

        let interactive = row![
            container(interactive_box)
                .width(300)
                .height(300)
                .center(Length::Fill),
            controls,
        ]
        .spacing(40)
        .align_y(Center);

        // --- Glow demo (like Flutter PageBackground) ----------------------
        let glow_bar = container(container("").width(Length::Fill).height(2).style(
            |_t: &Theme| container::Style {
                border: border::rounded(100),
                shadow: Shadow {
                    color: Color::from_rgba(1.0, 0.094, 0.094, 0.275),
                    offset: Vector::new(0.0, 4.0),
                    blur_radius: 33.0,
                    spread_radius: 4.0,
                    inset: false,
                },
                ..Default::default()
            },
        ))
        .width(Length::Fill)
        .style(|_t: &Theme| container::Style {
            shadow: Shadow {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.098),
                offset: Vector::ZERO,
                blur_radius: 250.0,
                spread_radius: 100.0,
                inset: false,
            },
            ..Default::default()
        });

        let page = column![
            text("Shadow Spread Radius").size(24),
            text("Static showcase").size(16),
            showcase,
            text("Interactive").size(16),
            interactive,
            text("Glow bar (PageBackground style)").size(16),
            glow_bar,
        ]
        .spacing(24)
        .padding(40)
        .align_x(Center);

        center(page).into()
    }
}

impl Default for Example {
    fn default() -> Self {
        Self::new()
    }
}

impl Example {
    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

/// Helper: renders a labelled box with a given spread value.
fn shadow_box<'a, Message: 'a>(
    label: &'a str,
    spread: f32,
    blur: f32,
    inset: bool,
    radius: f32,
) -> Element<'a, Message> {
    let shadow = Shadow {
        color: Color::from_rgba(0.3, 0.5, 1.0, 0.8),
        offset: Vector::new(0.0, 4.0),
        blur_radius: blur,
        spread_radius: spread,
        inset,
    };

    column![
        container("")
            .width(120)
            .height(120)
            .style(move |_t: &Theme| container::Style {
                background: Some(Color::from_rgb(0.15, 0.15, 0.2).into()),
                border: border::rounded(radius)
                    .width(1)
                    .color(Color::from_rgb(0.3, 0.3, 0.4)),
                shadow,
                ..Default::default()
            }),
        text(label).size(12),
    ]
    .spacing(8)
    .align_x(Center)
    .into()
}
