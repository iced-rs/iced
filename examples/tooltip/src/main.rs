use iced::tooltip::{self, Tooltip};
use iced::{
    alignment, button, Alignment, Button, Column, Container, Element, Length,
    Row, Sandbox, Settings, Text,
};

pub fn main() {
    Example::run(Settings::default()).unwrap()
}

#[derive(Default)]
struct Example {
    top: button::State,
    bottom: button::State,
    right: button::State,
    left: button::State,
    follow_cursor: button::State,
}

#[derive(Debug, Clone, Copy)]
struct Message;

impl Sandbox for Example {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("Tooltip - Iced")
    }

    fn update(&mut self, _message: Message) {}

    fn view(&mut self) -> Element<Message> {
        let top =
            tooltip("Tooltip at top", &mut self.top, tooltip::Position::Top);

        let bottom = tooltip(
            "Tooltip at bottom",
            &mut self.bottom,
            tooltip::Position::Bottom,
        );

        let left =
            tooltip("Tooltip at left", &mut self.left, tooltip::Position::Left);

        let right = tooltip(
            "Tooltip at right",
            &mut self.right,
            tooltip::Position::Right,
        );

        let fixed_tooltips = Row::with_children(vec![
            top.into(),
            bottom.into(),
            left.into(),
            right.into(),
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .align_items(Alignment::Center)
        .spacing(50);

        let follow_cursor = tooltip(
            "Tooltip follows cursor",
            &mut self.follow_cursor,
            tooltip::Position::FollowCursor,
        );

        let content = Column::with_children(vec![
            Container::new(fixed_tooltips)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .into(),
            follow_cursor.into(),
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(50);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .padding(50)
            .into()
    }
}

fn tooltip<'a>(
    label: &str,
    button_state: &'a mut button::State,
    position: tooltip::Position,
) -> Element<'a, Message> {
    Tooltip::new(
        Button::new(
            button_state,
            Text::new(label)
                .size(40)
                .width(Length::Fill)
                .height(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Center)
                .vertical_alignment(alignment::Vertical::Center),
        )
        .on_press(Message)
        .width(Length::Fill)
        .height(Length::Fill),
        "Tooltip",
        position,
    )
    .gap(5)
    .padding(10)
    .style(style::Tooltip)
    .into()
}

mod style {
    use iced::container;
    use iced::Color;

    pub struct Tooltip;

    impl container::StyleSheet for Tooltip {
        fn style(&self) -> container::Style {
            container::Style {
                text_color: Some(Color::from_rgb8(0xEE, 0xEE, 0xEE)),
                background: Some(Color::from_rgb(0.11, 0.42, 0.87).into()),
                border_radius: 12.0,
                ..container::Style::default()
            }
        }
    }
}
