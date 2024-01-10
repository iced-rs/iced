use iced::executor;
use iced::keyboard;
use iced::widget::{
    button, column, container, horizontal_space, row, text, vertical_rule,
};
use iced::{
    color, Application, Color, Command, Element, Length, Settings,
    Subscription, Theme,
};

pub fn main() -> iced::Result {
    Layout::run(Settings::default())
}

#[derive(Debug)]
struct Layout {
    example: Example,
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
                example: Example::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        format!("{} - Layout - Iced", self.example.title)
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::Next => {
                self.example = self.example.next();
            }
            Message::Previous => {
                self.example = self.example.previous();
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        keyboard::on_key_release(|key_code, _modifiers| match key_code {
            keyboard::KeyCode::Left => Some(Message::Previous),
            keyboard::KeyCode::Right => Some(Message::Next),
            _ => None,
        })
    }

    fn view(&self) -> Element<Message> {
        let example = container(self.example.view()).style(
            container::Appearance::default().with_border(Color::BLACK, 2.0),
        );

        let controls = row([
            (!self.example.is_first()).then_some(
                button("← Previous")
                    .padding([5, 10])
                    .on_press(Message::Previous)
                    .into(),
            ),
            Some(horizontal_space(Length::Fill).into()),
            (!self.example.is_last()).then_some(
                button("Next →")
                    .padding([5, 10])
                    .on_press(Message::Next)
                    .into(),
            ),
        ]
        .into_iter()
        .filter_map(std::convert::identity));

        column![example, controls].spacing(10).padding(20).into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Example {
    title: &'static str,
    view: fn() -> Element<'static, Message>,
}

impl Example {
    const LIST: &'static [Self] = &[
        Self {
            title: "Centered",
            view: centered,
        },
        Self {
            title: "Nested Quotes",
            view: nested_quotes,
        },
    ];

    fn is_first(self) -> bool {
        Self::LIST.first() == Some(&self)
    }

    fn is_last(self) -> bool {
        Self::LIST.last() == Some(&self)
    }

    fn previous(self) -> Self {
        let Some(index) =
            Self::LIST.iter().position(|&example| example == self)
        else {
            return self;
        };

        Self::LIST
            .get(index.saturating_sub(1))
            .copied()
            .unwrap_or(self)
    }

    fn next(self) -> Self {
        let Some(index) =
            Self::LIST.iter().position(|&example| example == self)
        else {
            return self;
        };

        Self::LIST.get(index + 1).copied().unwrap_or(self)
    }

    fn view(&self) -> Element<Message> {
        (self.view)()
    }
}

impl Default for Example {
    fn default() -> Self {
        Self::LIST[0]
    }
}

fn centered<'a>() -> Element<'a, Message> {
    container(text("I am centered!").size(50))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
}

fn nested_quotes<'a>() -> Element<'a, Message> {
    let quotes =
        (1..5).fold(column![text("Original text")].padding(10), |quotes, i| {
            column![
                container(
                    row![vertical_rule(2), quotes].height(Length::Shrink)
                )
                .style(
                    container::Appearance::default()
                        .with_background(color!(0x000000, 0.05))
                ),
                text(format!("Reply {i}"))
            ]
            .spacing(10)
            .padding(10)
        });

    container(quotes)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
}
