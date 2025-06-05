use iced::Element;
use iced::widget::tooltip::Position;
use iced::widget::{button, center, checkbox, column, container, tooltip};
use iced::{alignment, time::Duration};

pub fn main() -> iced::Result {
    iced::run(Tooltip::update, Tooltip::view)
}

#[derive(Default)]
struct Tooltip {
    position: Position,
    is_immediate: bool,
}

#[derive(Debug, Clone)]
enum Message {
    ChangePosition,
    SetImmediate(bool),
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

            Message::SetImmediate(is_immediate) => {
                self.is_immediate = is_immediate;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let delay =
            Duration::from_millis(if self.is_immediate { 0 } else { 2000 });

        let tooltip = tooltip(
            button("Press to change position")
                .on_press(Message::ChangePosition),
            position_to_text(self.position),
            self.position,
        )
        .gap(10)
        .delay(delay)
        .style(container::rounded_box);

        let checkbox = checkbox("Show immediately", self.is_immediate)
            .on_toggle(Message::SetImmediate);

        center(
            column![tooltip, checkbox,]
                .align_x(alignment::Horizontal::Center)
                .spacing(7),
        )
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
