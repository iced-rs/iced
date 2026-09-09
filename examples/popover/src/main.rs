use iced::Element;
use iced::widget::popover::Position;
use iced::widget::{button, column, container, popover, text};

pub fn main() -> iced::Result {
    iced::run(Popover::update, Popover::view)
}

#[derive(Default)]
struct Popover {
    is_open: bool,
    position: Position,
}

#[derive(Debug, Clone)]
enum Message {
    /// Open the popover.
    Open,
    /// Close the popover (emitted by the base button or `on_close`).
    Close,
    /// Cycle the popover position.
    ChangePosition,
}

impl Popover {
    fn update(&mut self, message: Message) {
        match message {
            Message::Open => self.is_open = true,
            Message::Close => self.is_open = false,
            Message::ChangePosition => {
                self.position = match self.position {
                    Position::Top => Position::Bottom,
                    Position::Bottom => Position::Left,
                    Position::Left => Position::Right,
                    Position::Right => Position::Top,
                };
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // The popover always displays its overlay. The application decides when
        // it is "open" by including it in the view, and "closes" it by removing
        // it (i.e. not including it).
        if self.is_open {
            popover(
                button(text(format!(
                    "Click to close ({position:?}!)",
                    position = self.position
                )))
                .on_press(Message::Close),
                container(
                    column![
                        text("This is the popover contents!"),
                        text("Click outside to dismiss me."),
                        button("Change position").on_press(Message::ChangePosition),
                    ]
                    .spacing(10),
                )
                .padding(10)
                .style(container::rounded_box),
                self.position,
            )
            .gap(10)
            .on_close(Message::Close)
            .into()
        } else {
            button(text(format!(
                "Open the popover ({position:?}!)",
                position = self.position
            )))
            .on_press(Message::Open)
            .into()
        }
    }
}
