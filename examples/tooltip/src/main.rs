use iced::theme;
use iced::widget::tooltip::Position;
use iced::widget::{button, column, container, progress_bar, tooltip};
use iced::{Element, Length, Sandbox, Settings};

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

struct Example {
    position: Position,
    progress: f32,
}

#[derive(Debug, Clone)]
enum Message {
    ChangePosition,
    IncreaseProgress,
}

impl Sandbox for Example {
    type Message = Message;

    fn new() -> Self {
        Self {
            position: Position::Bottom,
            progress: 0.0,
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

                self.position = position;
            }
            Message::IncreaseProgress => {
                if self.progress >= 1.0 {
                    self.progress = 0.0;
                }
                self.progress += 0.1;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let positioned_tooltip = tooltip(
            button("Press to change position")
                .on_press(Message::ChangePosition),
            position_to_text(self.position),
            self.position,
        )
        .gap(10)
        .style(theme::Container::Box);

        let progress_tooltip = tooltip(
            button("Press to increase progress")
                .on_press(Message::IncreaseProgress),
            progress_bar(0.0..=1.0, self.progress).width(Length::Fixed(100.0)),
            self.position,
        );

        let tooltips = column![positioned_tooltip, progress_tooltip]
            .spacing(100)
            .align_items(iced::Alignment::Center);

        container(tooltips)
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
