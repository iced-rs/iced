use iced::Element;
use iced::widget::popover::Position;
use iced::widget::{button, center, column, container, popover, text};

pub fn main() -> iced::Result {
    iced::run(Popover::update, Popover::view)
}

#[derive(Default)]
struct Popover {
    position: Position,
}

#[derive(Debug, Clone)]
enum Message {
    ChangePosition,
}

impl Popover {
    fn update(&mut self, message: Message) {
        match message {
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
        // The base of the popover is rendered like a `button`; clicking it
        // toggles the popover. Because the base is itself a button, its content
        // should be non-interactive (here, plain text).
        let popover = popover(
            text(format!(
                "Click me ({position:?}!)",
                position = self.position
            )),
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
        .gap(10);

        center(column![popover].spacing(10)).into()
    }
}
