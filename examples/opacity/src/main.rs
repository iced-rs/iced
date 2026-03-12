//! An example demonstrating opacity and fade animations.
//!
//! This example shows how to use the built-in `opacity` widget
//! to apply transparency to widgets, including animated fade effects.

use iced::time::{self, milliseconds};
use iced::widget::{button, center, column, container, image, opacity, row, slider, stack, text};
use iced::{Border, Center, Color, Element, Length, Shadow, Subscription, Theme, Vector};

use std::time::Instant;

pub fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .run()
}

struct App {
    /// Static opacity value (0.0 to 1.0)
    static_opacity: f32,
    /// Whether fade animation is running
    animation_state: AnimationState,
    /// Current animated opacity
    animated_opacity: f32,
    /// Animation duration in milliseconds
    animation_duration_ms: u64,
}

#[derive(Default, Clone, Copy, PartialEq)]
enum AnimationState {
    #[default]
    Idle,
    FadingIn {
        start: Instant,
    },
    FadingOut {
        start: Instant,
    },
}

#[derive(Debug, Clone)]
enum Message {
    StaticOpacityChanged(f32),
    FadeIn,
    FadeOut,
    AnimationTick,
    DurationChanged(f32),
    Reset,
}

impl App {
    fn new() -> Self {
        Self {
            static_opacity: 1.0,
            animation_state: AnimationState::Idle,
            animated_opacity: 1.0,
            animation_duration_ms: 500,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::StaticOpacityChanged(value) => {
                self.static_opacity = value;
            }
            Message::FadeIn => {
                self.animation_state = AnimationState::FadingIn {
                    start: Instant::now(),
                };
                self.animated_opacity = 0.0;
            }
            Message::FadeOut => {
                self.animation_state = AnimationState::FadingOut {
                    start: Instant::now(),
                };
                self.animated_opacity = 1.0;
            }
            Message::AnimationTick => {
                let duration_secs = self.animation_duration_ms as f32 / 1000.0;

                match self.animation_state {
                    AnimationState::FadingIn { start } => {
                        let elapsed = start.elapsed().as_secs_f32();
                        let progress = (elapsed / duration_secs).min(1.0);
                        // Ease-out cubic
                        self.animated_opacity = 1.0 - (1.0 - progress).powi(3);

                        if progress >= 1.0 {
                            self.animation_state = AnimationState::Idle;
                            self.animated_opacity = 1.0;
                        }
                    }
                    AnimationState::FadingOut { start } => {
                        let elapsed = start.elapsed().as_secs_f32();
                        let progress = (elapsed / duration_secs).min(1.0);
                        // Ease-in cubic
                        self.animated_opacity = 1.0 - progress.powi(3);

                        if progress >= 1.0 {
                            self.animation_state = AnimationState::Idle;
                            self.animated_opacity = 0.0;
                        }
                    }
                    AnimationState::Idle => {}
                }
            }
            Message::DurationChanged(ms) => {
                self.animation_duration_ms = ms as u64;
            }
            Message::Reset => {
                self.animated_opacity = 1.0;
                self.animation_state = AnimationState::Idle;
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        match self.animation_state {
            AnimationState::Idle => Subscription::none(),
            _ => time::every(milliseconds(16)).map(|_| Message::AnimationTick),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let title = text("Opacity Demo").size(32);

        // Static opacity section
        let static_section = {
            let label = text(format!(
                "Static Opacity: {:>3.0}%",
                self.static_opacity * 100.0
            ))
            .width(180);
            let opacity_slider = slider(
                0.0..=1.0,
                self.static_opacity,
                Message::StaticOpacityChanged,
            )
            .step(0.01)
            .width(200);

            let demo_box = opacity(
                self.static_opacity,
                demo_card("Static Opacity", Color::from_rgb(0.2, 0.6, 0.9)),
            );

            column![
                text("Static Opacity").size(24),
                row![label, opacity_slider].spacing(20).align_y(Center),
                demo_box,
            ]
            .spacing(15)
        };

        // Animation section
        let animation_section = {
            let duration_label =
                text(format!("Duration: {:>4}ms", self.animation_duration_ms)).width(150);
            let duration_slider = slider(100.0..=2000.0, self.animation_duration_ms as f32, |v| {
                Message::DurationChanged(v)
            })
            .step(50.0)
            .width(200);

            let fade_in_btn = button(text("Fade In"))
                .on_press(Message::FadeIn)
                .padding([8, 16]);

            let fade_out_btn = button(text("Fade Out"))
                .on_press(Message::FadeOut)
                .padding([8, 16]);

            let reset_btn = button(text("Reset"))
                .on_press(Message::Reset)
                .style(button::secondary)
                .padding([8, 16]);

            let status = text(format!(
                "Opacity: {:.0}% | {}",
                self.animated_opacity * 100.0,
                match self.animation_state {
                    AnimationState::Idle => "Idle",
                    AnimationState::FadingIn { .. } => "Fading In...",
                    AnimationState::FadingOut { .. } => "Fading Out...",
                }
            ));

            let animated_box = opacity(
                self.animated_opacity,
                demo_card("Animated!", Color::from_rgb(0.9, 0.4, 0.3)),
            );

            column![
                text("Fade Animation").size(24),
                row![duration_label, duration_slider]
                    .spacing(20)
                    .align_y(Center),
                row![fade_in_btn, fade_out_btn, reset_btn].spacing(10),
                status,
                animated_box,
            ]
            .spacing(15)
        };

        // Nested opacity section
        let nested_section = {
            // Demonstrate nested opacity: outer 50% * inner 50% = 25% final
            let inner = opacity(
                0.5,
                demo_card("Inner (50%)", Color::from_rgb(0.3, 0.8, 0.4)),
            );
            let outer = opacity(
                0.5,
                container(
                    column![
                        text("Outer Container (50%)").size(16),
                        inner,
                        text("Combined: 25%").size(12),
                    ]
                    .spacing(10)
                    .align_x(Center),
                )
                .padding(20)
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.15, 0.15, 0.2).into()),
                    border: Border {
                        color: Color::from_rgb(0.4, 0.4, 0.5),
                        width: 2.0,
                        radius: 12.0.into(),
                    },
                    ..Default::default()
                }),
            );

            column![
                text("Nested Opacity").size(24),
                text("Nested opacity multiplies: 50% × 50% = 25%").size(14),
                outer
            ]
            .spacing(15)
        };

        // Image opacity section
        let image_section = {
            let ferris = image(format!(
                "{}/examples/tour/images/ferris.png",
                env!("CARGO_MANIFEST_DIR").replace("/examples/opacity", "")
            ))
            .width(150);

            let image_with_opacity = opacity(self.static_opacity, ferris);

            column![
                text("Image Opacity").size(24),
                text("Images also fade with the static opacity slider above").size(14),
                image_with_opacity,
            ]
            .spacing(15)
            .align_x(Center)
        };

        // Shadow opacity section - demonstrates shadow fading with opacity
        let shadow_section = {
            let shadow_card = container(
                column![
                    text("Shadow Test").size(18),
                    text("Card with shadow").size(12),
                ]
                .spacing(8)
                .align_x(Center),
            )
            .width(200)
            .padding(20)
            .style(move |_| container::Style {
                background: Some(Color::from_rgb(0.3, 0.3, 0.35).into()),
                border: Border {
                    color: Color::from_rgb(0.5, 0.5, 0.6),
                    width: 1.0,
                    radius: 12.0.into(),
                },
                shadow: Shadow {
                    color: Color::BLACK,
                    offset: Vector::new(0.0, 8.0),
                    blur_radius: 24.0,
                },
                ..Default::default()
            });

            let with_opacity = opacity(self.static_opacity, shadow_card);

            column![
                text("Shadow Opacity").size(24),
                text("Shadow should fade with the card").size(14),
                container(with_opacity).padding(30), // Extra padding for shadow visibility
            ]
            .spacing(15)
            .align_x(Center)
        };

        // Overlapping items section - demonstrates how items composite within an opacity layer
        let overlapping_section = {
            // Blue rectangle (larger, behind)
            let blue_rect = container(text(""))
                .width(120)
                .height(80)
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.2, 0.4, 0.9).into()),
                    border: Border {
                        color: Color::WHITE,
                        width: 2.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                });

            // Red rectangle (smaller, in front, offset via alignment)
            let red_rect = container(text(""))
                .width(120)
                .height(80)
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.9, 0.2, 0.2).into()),
                    border: Border {
                        color: Color::WHITE,
                        width: 2.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                });

            // Stack both - blue aligned top-left, red aligned bottom-right
            // This creates overlap in the center
            let overlapping = container(
                stack![
                    container(blue_rect)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(iced::alignment::Horizontal::Left)
                        .align_y(iced::alignment::Vertical::Top),
                    container(red_rect)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(iced::alignment::Horizontal::Right)
                        .align_y(iced::alignment::Vertical::Bottom),
                ]
                .width(Length::Fill)
                .height(Length::Fill),
            )
            .width(180)
            .height(120);

            let with_opacity = opacity(self.static_opacity, overlapping);

            column![
                text("Overlapping Items").size(24),
                text("Items composite first, then opacity applies to the whole group.").size(14),
                with_opacity,
            ]
            .spacing(15)
            .align_x(Center)
        };

        let content = column![
            title,
            row![static_section, animation_section,].spacing(60),
            row![
                nested_section,
                image_section,
                shadow_section,
                overlapping_section
            ]
            .spacing(60),
        ]
        .padding(30)
        .spacing(30);

        center(content).into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

/// A demo card widget for showing opacity effects
fn demo_card<'a, Message: 'a>(label: &'a str, color: Color) -> Element<'a, Message> {
    container(
        column![
            text(label).size(18),
            text("This content has opacity applied").size(12),
        ]
        .spacing(8)
        .align_x(Center),
    )
    .width(200)
    .padding(20)
    .style(move |_| container::Style {
        background: Some(color.into()),
        border: Border {
            color: Color::WHITE,
            width: 2.0,
            radius: 10.0.into(),
        },
        ..Default::default()
    })
    .into()
}
