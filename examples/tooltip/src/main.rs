use iced::widget::tooltip::Position;
use iced::widget::{button, center, container, tooltip};
use iced::Element;

pub fn main() -> iced::Result {
    iced::run("Tooltip - Iced", Tooltip::update, Tooltip::view)
}

#[derive(Default)]
struct Tooltip {
    position: Position,
}

#[derive(Debug, Clone)]
enum Message {
    ChangePosition,
}

impl Tooltip {
    fn update(&mut self, message: Message) {
        match message {
            Message::ChangePosition => {
                let position = match &self.position {
                    Position::Top => Position::Bottom,
                    Position::Bottom => Position::Left,
                    Position::Left => Position::Right,
                    Position::Right => Position::FollowCursor,
                    Position::FollowCursor => Position::Top,
                };

                self.position = position;
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
        .style(container::rounded_box);

        center(tooltip).into()
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
