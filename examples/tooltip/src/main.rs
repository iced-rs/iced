use iced::theme;
use iced::widget::tooltip::Position;
use iced::widget::{button, container, tooltip};
use iced::{Element, Length, Sandbox, Settings};

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
        .style(theme::Container::Box);

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
