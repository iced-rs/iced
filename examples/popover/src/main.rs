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
                    Position::Top => Position::Bottom,
                    Position::Bottom => Position::Left,
                    Position::Left => Position::Right,
                    Position::Right => Position::Top,
                };
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // The popover always displays its overlay, so the application decides
        // when it is "open" by including it in the view, and "closes" it by
        // removing it. The same trigger button is used in both cases: when
        // closed it stands alone, and when open it anchors the popover.
        let trigger = button(text(format!(
            "{} the popover ({position:?}!)",
            if self.is_open { "Close" } else { "Open" },
            position = self.position
        )))
        .on_press(Message::Toggle);

        let content: Element<'_, Message> = if self.is_open {
            popover(
                trigger,
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
            trigger.into()
        };

        center(content).into()
    }
}
