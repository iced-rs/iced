use iced::event::{self, Event};
use iced::keyboard;
use iced::keyboard::key;
use iced::widget::operation::{self, FocusDirection};
use iced::widget::{button, center, column, container, row, text};
use iced::{Border, Element, Length, Subscription, Task, Theme, color};

/// Demonstrates **directional (spatial) focus navigation**.
///
/// A grid of buttons is laid out in rows. Arrow keys move focus to the
/// nearest widget in that direction based on screen position, rather than
/// sequential tree order.
///
/// - **Arrow keys** → spatial directional focus
/// - **Tab / Shift+Tab** → sequential focus (next / previous in tree order)
pub fn main() -> iced::Result {
    env_logger::init();

    iced::application(App::default, App::update, App::view)
        .title("Directional Focus")
        .subscription(App::subscription)
        .run()
}

#[derive(Default)]
struct App {
    last_pressed: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    Pressed(String),
    Event(Event),
}

impl App {
    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::Event)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Pressed(label) => {
                self.last_pressed = Some(label);
                Task::none()
            }
            Message::Event(Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(named),
                modifiers,
                ..
            })) => match named {
                key::Named::ArrowUp => operation::focus_direction(FocusDirection::Up),
                key::Named::ArrowDown => operation::focus_direction(FocusDirection::Down),
                key::Named::ArrowLeft => operation::focus_direction(FocusDirection::Left),
                key::Named::ArrowRight => operation::focus_direction(FocusDirection::Right),
                key::Named::Tab => {
                    if modifiers.shift() {
                        operation::focus_previous()
                    } else {
                        operation::focus_next()
                    }
                }
                _ => Task::none(),
            },
            Message::Event(_) => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let status = match &self.last_pressed {
            Some(label) => text(format!("Last pressed: {label}")).size(14),
            None => text("Press Enter/Space on a focused button, or use arrow keys to navigate.")
                .size(14),
        };

        // A 4×4 grid of buttons — spatial navigation shines here.
        let grid = column![
            grid_row(&["1", "2", "3", "4"]),
            grid_row(&["5", "6", "7", "8"]),
            grid_row(&["9", "10", "11", "12"]),
            grid_row(&["13", "14", "15", "16"]),
        ]
        .spacing(12);

        // An offset row to show diagonal spatial targeting.
        let offset = row![
            iced::widget::Space::new().width(60),
            grid_button("A"),
            iced::widget::Space::new().width(80),
            grid_button("B"),
            iced::widget::Space::new().width(80),
            grid_button("C"),
        ]
        .spacing(0);

        let legend = column![
            text("Arrow keys → directional (spatial) focus").size(12),
            text("Tab / Shift+Tab → sequential focus").size(12),
        ]
        .spacing(4);

        center(
            column![
                text("Directional Focus").size(24),
                status,
                container(column![grid, offset].spacing(12))
                    .padding(20)
                    .style(grid_container),
                legend,
            ]
            .spacing(16)
            .align_x(iced::Center),
        )
        .padding(40)
        .into()
    }
}

fn grid_row<'a>(labels: &'a [&'a str]) -> Element<'a, Message> {
    let buttons: Vec<Element<'a, Message>> = labels.iter().map(|l| grid_button(l)).collect();

    row(buttons).spacing(12).into()
}

fn grid_button(label: &str) -> Element<'_, Message> {
    let label = label.to_string();

    button(
        text(label.clone())
            .size(16)
            .center()
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(70)
    .height(50)
    .on_press(Message::Pressed(label))
    .into()
}

fn grid_container(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(palette.background.weak.color.into()),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: color!(0x444444),
        },
        ..Default::default()
    }
}
