use iced::highlighter;
use iced::time::{self, milliseconds};
use iced::widget::{
    self, hover, markdown, right, row, scrollable, text_editor, toggler,
};
use iced::{Element, Fill, Font, Subscription, Task, Theme};

pub fn main() -> iced::Result {
    iced::application("Markdown - Iced", Markdown::update, Markdown::view)
        .subscription(Markdown::subscription)
        .theme(Markdown::theme)
        .run_with(Markdown::new)
}

struct Markdown {
    content: text_editor::Content,
    mode: Mode,
    theme: Theme,
}

enum Mode {
    Oneshot(Vec<markdown::Item>),
    Stream {
        pending: String,
        parsed: markdown::Content,
    },
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    LinkClicked(markdown::Url),
    ToggleStream(bool),
    NextToken,
}

impl Markdown {
    fn new() -> (Self, Task<Message>) {
        const INITIAL_CONTENT: &str = include_str!("../overview.md");

        let theme = Theme::TokyoNight;

        (
            Self {
                content: text_editor::Content::with_text(INITIAL_CONTENT),
                mode: Mode::Oneshot(markdown::parse(INITIAL_CONTENT).collect()),
                theme,
            },
            widget::focus_next(),
        )
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Edit(action) => {
                let is_edit = action.is_edit();

                self.content.perform(action);

                if is_edit {
                    self.mode = match self.mode {
                        Mode::Oneshot(_) => Mode::Oneshot(
                            markdown::parse(&self.content.text()).collect(),
                        ),
                        Mode::Stream { .. } => Mode::Stream {
                            pending: self.content.text(),
                            parsed: markdown::Content::parse(""),
                        },
                    }
                }
            }
            Message::LinkClicked(link) => {
                let _ = open::that_in_background(link.to_string());
            }
            Message::ToggleStream(enable_stream) => {
                self.mode = if enable_stream {
                    Mode::Stream {
                        pending: self.content.text(),
                        parsed: markdown::Content::parse(""),
                    }
                } else {
                    Mode::Oneshot(
                        markdown::parse(&self.content.text()).collect(),
                    )
                };
            }
            Message::NextToken => match &mut self.mode {
                Mode::Oneshot(_) => {}
                Mode::Stream { pending, parsed } => {
                    if pending.is_empty() {
                        self.mode = Mode::Oneshot(parsed.items().to_vec());
                    } else {
                        let mut tokens = pending.split(' ');

                        if let Some(token) = tokens.next() {
                            parsed.push_str(&format!("{token} "));
                        }

                        *pending = tokens.collect::<Vec<_>>().join(" ");
                    }
                }
            },
        }
    }

    fn view(&self) -> Element<Message> {
        let editor = text_editor(&self.content)
            .placeholder("Type your Markdown here...")
            .on_action(Message::Edit)
            .height(Fill)
            .padding(10)
            .font(Font::MONOSPACE)
            .highlight("markdown", highlighter::Theme::Base16Ocean);

        let items = match &self.mode {
            Mode::Oneshot(items) => items.as_slice(),
            Mode::Stream { parsed, .. } => parsed.items(),
        };

        let preview = markdown(
            items,
            markdown::Settings::default(),
            markdown::Style::from_palette(self.theme.palette()),
        )
        .map(Message::LinkClicked);

        row![
            editor,
            hover(
                scrollable(preview).spacing(10).width(Fill).height(Fill),
                right(
                    toggler(matches!(self.mode, Mode::Stream { .. }))
                        .label("Stream")
                        .on_toggle(Message::ToggleStream)
                )
                .padding([0, 20])
            )
        ]
        .spacing(10)
        .padding(10)
        .into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn subscription(&self) -> Subscription<Message> {
        match self.mode {
            Mode::Oneshot(_) => Subscription::none(),
            Mode::Stream { .. } => {
                time::every(milliseconds(20)).map(|_| Message::NextToken)
            }
        }
    }
}
