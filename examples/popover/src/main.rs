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
    /// Toggle the popover, emitted by the base button.
    Toggle,
    /// Close the popover, emitted when the user clicks outside of its bounds.
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
                    Position::Top => Position::Bottom,
                    Position::Bottom => Position::Left,
                    Position::Left => Position::Right,
                    Position::Right => Position::Top,
                };
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // The popover is a *controlled* widget: the application tracks whether
        // it is open and provides that state on creation. The base is a plain
        // button the application uses to open (and close) the popover, and the
        // popover notifies the application of a close request through its
        // `on_close` handler when the user clicks outside of its bounds.
        let popover = popover(
            self.is_open,
            button(text(format!(
                "Click me ({position:?}!)",
                position = self.position
            )))
            .on_press(Message::Toggle),
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
        .on_close(Message::Close);

        center(column![popover].spacing(10)).into()
    }
}
