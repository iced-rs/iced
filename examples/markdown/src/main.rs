use iced::font;
use iced::padding;
use iced::widget::{
    self, column, container, rich_text, row, span, text_editor,
};
use iced::{Element, Fill, Font, Task, Theme};

pub fn main() -> iced::Result {
    iced::application("Markdown - Iced", Markdown::update, Markdown::view)
        .theme(Markdown::theme)
        .run_with(Markdown::new)
}

struct Markdown {
    content: text_editor::Content,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
}

impl Markdown {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                content: text_editor::Content::with_text(
                    "# Markdown Editor\nType your Markdown here...",
                ),
            },
            widget::focus_next(),
        )
    }
    fn update(&mut self, message: Message) {
        match message {
            Message::Edit(action) => {
                self.content.perform(action);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let editor = text_editor(&self.content)
            .on_action(Message::Edit)
            .height(Fill)
            .padding(10)
            .font(Font::MONOSPACE);

        let preview = {
            let markdown = self.content.text();
            let parser = pulldown_cmark::Parser::new(&markdown);

            let mut strong = false;
            let mut emphasis = false;
            let mut heading = None;
            let mut spans = Vec::new();

            let items = parser.filter_map(|event| match event {
                pulldown_cmark::Event::Start(tag) => match tag {
                    pulldown_cmark::Tag::Strong => {
                        strong = true;
                        None
                    }
                    pulldown_cmark::Tag::Emphasis => {
                        emphasis = true;
                        None
                    }
                    pulldown_cmark::Tag::Heading { level, .. } => {
                        heading = Some(level);
                        None
                    }
                    _ => None,
                },
                pulldown_cmark::Event::End(tag) => match tag {
                    pulldown_cmark::TagEnd::Emphasis => {
                        emphasis = false;
                        None
                    }
                    pulldown_cmark::TagEnd::Strong => {
                        strong = false;
                        None
                    }
                    pulldown_cmark::TagEnd::Heading(_) => {
                        heading = None;
                        Some(
                            container(rich_text(spans.drain(..)))
                                .padding(padding::bottom(5))
                                .into(),
                        )
                    }
                    pulldown_cmark::TagEnd::Paragraph => Some(
                        container(rich_text(spans.drain(..)))
                            .padding(padding::bottom(15))
                            .into(),
                    ),
                    pulldown_cmark::TagEnd::CodeBlock => Some(
                        container(
                            container(
                                rich_text(spans.drain(..))
                                    .font(Font::MONOSPACE),
                            )
                            .width(Fill)
                            .padding(10)
                            .style(container::rounded_box),
                        )
                        .padding(padding::bottom(15))
                        .into(),
                    ),
                    _ => None,
                },
                pulldown_cmark::Event::Text(text) => {
                    let span = span(text.into_string());

                    let span = match heading {
                        None => span,
                        Some(heading) => span.size(match heading {
                            pulldown_cmark::HeadingLevel::H1 => 32,
                            pulldown_cmark::HeadingLevel::H2 => 28,
                            pulldown_cmark::HeadingLevel::H3 => 24,
                            pulldown_cmark::HeadingLevel::H4 => 20,
                            pulldown_cmark::HeadingLevel::H5 => 16,
                            pulldown_cmark::HeadingLevel::H6 => 16,
                        }),
                    };

                    let span = if strong || emphasis {
                        span.font(Font {
                            weight: if strong {
                                font::Weight::Bold
                            } else {
                                font::Weight::Normal
                            },
                            style: if emphasis {
                                font::Style::Italic
                            } else {
                                font::Style::Normal
                            },
                            ..Font::default()
                        })
                    } else {
                        span
                    };

                    spans.push(span);

                    None
                }
                pulldown_cmark::Event::Code(code) => {
                    spans.push(span(code.into_string()).font(Font::MONOSPACE));
                    None
                }
                pulldown_cmark::Event::SoftBreak => {
                    spans.push(span(" "));
                    None
                }
                pulldown_cmark::Event::HardBreak => {
                    spans.push(span("\n"));
                    None
                }
                _ => None,
            });

            column(items).width(Fill)
        };

        row![editor, preview].spacing(10).padding(10).into()
    }

    fn theme(&self) -> Theme {
        Theme::TokyoNight
    }
}
