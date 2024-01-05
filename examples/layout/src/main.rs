use iced::executor;
use iced::widget::{column, container, row, text, vertical_rule};
use iced::{
    Application, Command, Element, Length, Settings, Subscription, Theme,
};

pub fn main() -> iced::Result {
    Layout::run(Settings::default())
}

#[derive(Debug)]
struct Layout {
    previous: Vec<Example>,
    current: Example,
    next: Vec<Example>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Next,
    Previous,
}

impl Application for Layout {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                previous: vec![],
                current: Example::Centered,
                next: vec![Example::NestedQuotes],
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Counter - Iced")
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::Next => {
                if !self.next.is_empty() {
                    let previous = std::mem::replace(
                        &mut self.current,
                        self.next.remove(0),
                    );

                    self.previous.push(previous);
                }
            }
            Message::Previous => {
                if let Some(previous) = self.previous.pop() {
                    let next = std::mem::replace(&mut self.current, previous);

                    self.next.insert(0, next);
                }
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        use iced::event::{self, Event};
        use iced::keyboard;

        event::listen_with(|event, status| match event {
            Event::Keyboard(keyboard::Event::KeyReleased {
                key_code, ..
            }) if status == event::Status::Ignored => match key_code {
                keyboard::KeyCode::Left => Some(Message::Previous),
                keyboard::KeyCode::Right => Some(Message::Next),
                _ => None,
            },
            _ => None,
        })
    }

    fn view(&self) -> Element<Message> {
        self.current.view()
    }
}

#[derive(Debug)]
enum Example {
    Centered,
    NestedQuotes,
}

impl Example {
    fn view(&self) -> Element<Message> {
        match self {
            Self::Centered => container(text("I am centered!").size(50))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .into(),
            Self::NestedQuotes => container((1..5).fold(
                column![text("Original text")].padding(10),
                |quotes, i| {
                    column![
                        row![vertical_rule(2), quotes].height(Length::Shrink),
                        text(format!("Reply {i}"))
                    ]
                    .spacing(10)
                    .padding(10)
                },
            ))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into(),
        }
    }
}
