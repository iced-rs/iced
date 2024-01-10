use iced::executor;
use iced::keyboard;
use iced::widget::{
    button, checkbox, column, container, horizontal_space, pick_list, row,
    text, vertical_rule,
};
use iced::{
    color, Alignment, Application, Color, Command, Element, Font, Length,
    Settings, Subscription, Theme,
};

pub fn main() -> iced::Result {
    Layout::run(Settings::default())
}

#[derive(Debug)]
struct Layout {
    example: Example,
    explain: bool,
    theme: Theme,
}

#[derive(Debug, Clone)]
enum Message {
    Next,
    Previous,
    ExplainToggled(bool),
    ThemeSelected(Theme),
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
                explain: false,
                theme: Theme::Light,
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
            Message::ExplainToggled(explain) => {
                self.explain = explain;
            }
            Message::ThemeSelected(theme) => {
                self.theme = theme;
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
        let header = row![
            text(self.example.title).size(20).font(Font::MONOSPACE),
            horizontal_space(Length::Fill),
            checkbox("Explain", self.explain, Message::ExplainToggled),
            pick_list(
                Theme::ALL,
                Some(self.theme.clone()),
                Message::ThemeSelected
            ),
        ]
        .spacing(20)
        .align_items(Alignment::Center);

        let example = container(if self.explain {
            self.example.view().explain(color!(0x0000ff))
        } else {
            self.example.view()
        })
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();

            container::Appearance::default()
                .with_border(palette.background.strong.color, 4.0)
        });

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
        .flatten());

        column![header, example, controls]
            .spacing(10)
            .padding(20)
            .into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
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
                .style(|theme: &Theme| {
                    let palette = theme.extended_palette();

                    container::Appearance::default().with_background(
                        if palette.is_dark {
                            Color {
                                a: 0.01,
                                ..Color::WHITE
                            }
                        } else {
                            Color {
                                a: 0.08,
                                ..Color::BLACK
                            }
                        },
                    )
                }),
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
