//! Compositor-driven tooltip example.
//!
//! Demonstrates both tooltip modes using the zcosmic_tooltip_v1 Wayland protocol:
//!
//! 1. **Immediate (cursor-following)**: Tooltip appears instantly and follows the pointer.
//!    Hover over the blue box to see it.
//! 2. **Delayed (fixed-position)**: Tooltip appears after a configurable delay at the
//!    pointer's position when the delay expires, then stays fixed.
//!    Hover over the green box to see it.
//!
//! Requires a compositor that supports `zcosmic_tooltip_manager_v1`.
//!
//! Run with: `RUST_LOG=debug cargo run -p compositor_tooltip`

use iced::widget::{button, center, column, container, mouse_area, row, slider, text};
use iced::window;
use iced::{Color, Element, Length, Task};

#[cfg(all(
    feature = "wayland",
    unix,
    not(any(target_os = "macos", target_os = "ios", target_os = "android"))
))]
use iced::wayland::popup::{self, Anchor, Gravity, PopupSettings, Positioner};

fn main() -> iced::Result {
    // Enable logging so we can see debug output from winit/iced
    env_logger::init();

    iced::daemon(App::boot, App::update, App::view)
        .title(title)
        .run()
}

fn title(_state: &App, _id: window::Id) -> String {
    String::from("Compositor Tooltip Demo")
}

#[derive(Debug)]
struct App {
    main_window: window::Id,
    immediate_popup: Option<window::Id>,
    delayed_popup: Option<window::Id>,
    anchor: u32,
    delay_ms: u32,
    offset_x: i32,
    offset_y: i32,
}

#[derive(Debug, Clone)]
enum Message {
    WindowOpened(window::Id),
    // Hover triggers
    ImmediateHoverEnter,
    ImmediateHoverExit,
    DelayedHoverEnter,
    DelayedHoverExit,
    // Settings
    CycleAnchor,
    DelayChanged(u32),
    OffsetXChanged(i32),
    OffsetYChanged(i32),
}

impl App {
    fn boot() -> (Self, Task<Message>) {
        let (id, open_task) = window::open(window::Settings {
            size: iced::Size::new(700.0, 600.0),
            ..Default::default()
        });

        (
            Self {
                main_window: id,
                immediate_popup: None,
                delayed_popup: None,
                anchor: 3,
                delay_ms: 500,
                offset_x: 16,
                offset_y: 16,
            },
            open_task.map(Message::WindowOpened),
        )
    }

    #[cfg(all(
        feature = "wayland",
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ))]
    fn show_tooltip(
        &self,
        popup_id: window::Id,
        size: (u32, u32),
        delay_ms: Option<u32>,
    ) -> Task<Message> {
        let settings = PopupSettings {
            parent: self.main_window,
            id: popup_id,
            positioner: Positioner {
                size: Some(size),
                anchor_rect: iced::Rectangle {
                    x: 0,
                    y: 0,
                    width: 1,
                    height: 1,
                },
                anchor: Anchor::TopLeft,
                gravity: Gravity::BottomRight,
                ..Default::default()
            },
            grab: false,
            input_passthrough: true,
            tooltip_offset: Some((self.offset_x, self.offset_y)),
            tooltip_anchor: Some(self.anchor),
            tooltip_delay_ms: delay_ms,
        };

        popup::show(settings)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowOpened(id) => {
                self.main_window = id;
                Task::none()
            }
            #[cfg(all(
                feature = "wayland",
                unix,
                not(any(target_os = "macos", target_os = "ios", target_os = "android"))
            ))]
            Message::ImmediateHoverEnter => {
                let mut tasks = Vec::new();
                if let Some(id) = self.immediate_popup.take() {
                    tasks.push(popup::hide(id));
                }

                let popup_id = window::Id::unique();
                self.immediate_popup = Some(popup_id);
                tasks.push(self.show_tooltip(popup_id, (220, 36), None));
                Task::batch(tasks)
            }
            #[cfg(all(
                feature = "wayland",
                unix,
                not(any(target_os = "macos", target_os = "ios", target_os = "android"))
            ))]
            Message::ImmediateHoverExit => {
                if let Some(id) = self.immediate_popup.take() {
                    popup::hide(id)
                } else {
                    Task::none()
                }
            }
            #[cfg(all(
                feature = "wayland",
                unix,
                not(any(target_os = "macos", target_os = "ios", target_os = "android"))
            ))]
            Message::DelayedHoverEnter => {
                let mut tasks = Vec::new();
                if let Some(id) = self.delayed_popup.take() {
                    tasks.push(popup::hide(id));
                }

                let popup_id = window::Id::unique();
                self.delayed_popup = Some(popup_id);
                tasks.push(self.show_tooltip(popup_id, (260, 52), Some(self.delay_ms)));
                Task::batch(tasks)
            }
            #[cfg(all(
                feature = "wayland",
                unix,
                not(any(target_os = "macos", target_os = "ios", target_os = "android"))
            ))]
            Message::DelayedHoverExit => {
                if let Some(id) = self.delayed_popup.take() {
                    popup::hide(id)
                } else {
                    Task::none()
                }
            }
            Message::CycleAnchor => {
                self.anchor = (self.anchor + 1) % 4;
                Task::none()
            }
            Message::DelayChanged(ms) => {
                self.delay_ms = ms;
                Task::none()
            }
            Message::OffsetXChanged(x) => {
                self.offset_x = x;
                Task::none()
            }
            Message::OffsetYChanged(y) => {
                self.offset_y = y;
                Task::none()
            }
            #[cfg(not(all(
                feature = "wayland",
                unix,
                not(any(target_os = "macos", target_os = "ios", target_os = "android"))
            )))]
            _ => Task::none(),
        }
    }

    fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if Some(window_id) == self.immediate_popup {
            return self.view_immediate_tooltip();
        }
        if Some(window_id) == self.delayed_popup {
            return self.view_delayed_tooltip();
        }
        self.view_main()
    }

    fn view_main(&self) -> Element<'_, Message> {
        let anchor_label = match self.anchor {
            0 => "TopLeft",
            1 => "TopRight",
            2 => "BottomLeft",
            3 => "BottomRight",
            _ => "Unknown",
        };

        // Hover target for immediate tooltip
        let immediate_target = mouse_area(
            container(
                column![
                    text("Hover me!").size(16).color(Color::WHITE),
                    text("Immediate tooltip").size(12).color([0.8, 0.8, 1.0]),
                ]
                .spacing(4)
                .align_x(iced::Alignment::Center),
            )
            .padding(20)
            .width(250)
            .height(80)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .style(|_theme: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.2, 0.3, 0.6))),
                border: iced::Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
        .on_enter(Message::ImmediateHoverEnter)
        .on_exit(Message::ImmediateHoverExit);

        // Hover target for delayed tooltip
        let delayed_target = mouse_area(
            container(
                column![
                    text("Hover me!").size(16).color(Color::WHITE),
                    text(format!("Delayed tooltip ({}ms)", self.delay_ms))
                        .size(12)
                        .color([0.8, 1.0, 0.8]),
                ]
                .spacing(4)
                .align_x(iced::Alignment::Center),
            )
            .padding(20)
            .width(250)
            .height(80)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .style(|_theme: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.2, 0.5, 0.3))),
                border: iced::Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
        .on_enter(Message::DelayedHoverEnter)
        .on_exit(Message::DelayedHoverExit);

        let controls = column![
            text("Compositor-Driven Tooltips").size(28),
            text("Using zcosmic_tooltip_v1 Wayland protocol").size(14),
            text(""),
            text("Hover over the colored boxes to trigger tooltips:").size(16),
            text(""),
            row![immediate_target, delayed_target,].spacing(20),
            text(""),
            text("Settings").size(20),
            row![
                text(format!("Anchor: {anchor_label}")).width(160),
                button("Cycle Anchor").on_press(Message::CycleAnchor),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center),
            row![
                text(format!("Delay: {}ms", self.delay_ms)).width(160),
                slider(0..=2000, self.delay_ms, Message::DelayChanged).width(250),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center),
            row![
                text(format!("Offset X: {}", self.offset_x)).width(160),
                slider(-100..=100, self.offset_x, Message::OffsetXChanged).width(250),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center),
            row![
                text(format!("Offset Y: {}", self.offset_y)).width(160),
                slider(-100..=100, self.offset_y, Message::OffsetYChanged).width(250),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(10)
        .padding(30);

        center(controls)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_immediate_tooltip(&self) -> Element<'_, Message> {
        container(
            text("Follows cursor!")
                .size(14)
                .color(Color::WHITE),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(8)
        .style(|_theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
            border: iced::Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    }

    fn view_delayed_tooltip(&self) -> Element<'_, Message> {
        container(
            column![
                text("Delayed tooltip").size(14).color(Color::WHITE),
                text(format!("Appeared after {}ms", self.delay_ms))
                    .size(12)
                    .color([0.8, 0.8, 0.8]),
            ]
            .spacing(4),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(8)
        .style(|_theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.15, 0.3, 0.5))),
            border: iced::Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    }
}
