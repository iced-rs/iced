use iced::Element;
use iced::alignment;
use iced::time::seconds;
use iced::widget::tooltip::Position;
use iced::widget::{button, center, checkbox, column, container, tooltip};

pub fn main() -> iced::Result {
    iced::run(Tooltip::update, Tooltip::view)
}

#[derive(Default)]
struct Tooltip {
    position: Position,
    delay: bool,
}

#[derive(Debug, Clone)]
enum Message {
    ChangePosition,
    ToggleDelay(bool),
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
            Message::ToggleDelay(is_immediate) => {
                self.delay = is_immediate;
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let tooltip = tooltip(
            button("Press to change position").on_press(Message::ChangePosition),
            position_to_text(self.position),
            self.position,
        )
        .gap(10)
        .delay(seconds(if self.delay { 1 } else { 0 }))
        .style(container::rounded_box);

        let checkbox = checkbox(self.delay)
            .label("Delay")
            .on_toggle(Message::ToggleDelay);

        center(
            column![tooltip, checkbox]
                .align_x(alignment::Horizontal::Center)
                .spacing(10),
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
