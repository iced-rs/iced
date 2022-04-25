use iced::pure::{
    button, container, tooltip, widget::tooltip::Position, Element, Sandbox,
};
use iced::{Length, Settings};

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

struct Example {
    position: Position,
}

#[derive(Debug, Clone)]
enum Message {
    ChangePosition,
}

impl Sandbox for Example {
    type Message = Message;

    fn new() -> Self {
        Self {
            position: Position::Bottom,
        }
    }

    fn title(&self) -> String {
        String::from("Tooltip - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ChangePosition => {
                let position = match &self.position {
                    Position::FollowCursor => Position::Top,
                    Position::Top => Position::Bottom,
                    Position::Bottom => Position::Left,
                    Position::Left => Position::Right,
                    Position::Right => Position::FollowCursor,
                };

                self.position = position
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let tooltip = tooltip(
            button("Press to change position")
                .on_press(Message::ChangePosition),
            position_to_text(self.position),
            self.position,
        )
        .gap(10)
        .style(style::Tooltip);

        container(tooltip)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

fn position_to_text<'a>(position: Position) -> &'a str {
    match position {
        Position::FollowCursor => "Follow Cursor",
        Position::Top => "Top",
        Position::Bottom => "Bottom",
        Position::Left => "Left",
        Position::Right => "Right",
    }
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
