use iced::Element;
use iced::widget::popover::Position;
use iced::widget::{button, center, column, container, popover, text};

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
    /// Toggle the popover, emitted by the trigger button.
    Toggle,
    /// Close the popover, emitted by `on_close` when clicking outside.
    Close,
    /// Cycle the popover position.
    ChangePosition,
}

impl Popover {
    fn update(&mut self, message: Message) {
        match message {
            Message::Toggle => self.is_open = !self.is_open,
            Message::Close => self.is_open = false,
            Message::ChangePosition => {
                self.position = match self.position {
                    Position::Auto => Position::Top,
                    Position::Top => Position::Bottom,
                    Position::Bottom => Position::Left,
                    Position::Left => Position::Right,
                    Position::Right => Position::Auto,
                };
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // The base is always present. The `popover` argument is `Some` when the
        // popover is open and `None` when it is closed.
        let trigger = button(text(format!(
            "{} the popover ({position:?}!)",
            if self.is_open { "Close" } else { "Open" },
            position = self.position
        )))
        .on_press(Message::Toggle);

        center(
            popover(
                trigger,
                self.is_open.then(|| {
                    container(
                        column![
                            text("This is the popover contents!"),
                            text("Click outside to dismiss me."),
                            button("Change position").on_press(Message::ChangePosition),
                        ]
                        .spacing(10),
                    )
                    .padding(10)
                    .style(container::rounded_box)
                }),
            )
            .position(self.position)
            .gap(10)
            .on_close(Message::Close),
        )
        .into()
    }
}
