use iced::widget::{
    self, column, container, rich_text, row, scrollable, span, text,
    text_editor,
};
use iced::{Element, Fill, Font, Task, Theme};

pub fn main() -> iced::Result {
    iced::application("Markdown - Iced", Markdown::update, Markdown::view)
        .theme(Markdown::theme)
        .run_with(Markdown::new)
}

struct Markdown {
    content: text_editor::Content,
    items: Vec<Item>,
    theme: Theme,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
}

impl Markdown {
    fn new() -> (Self, Task<Message>) {
        const INITIAL_CONTENT: &str = include_str!("../overview.md");

        let theme = Theme::TokyoNight;

        (
            Self {
                content: text_editor::Content::with_text(INITIAL_CONTENT),
                items: parse(INITIAL_CONTENT, &theme).collect(),
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
                    self.items =
                        parse(&self.content.text(), &self.theme).collect();
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let editor = text_editor(&self.content)
            .on_action(Message::Edit)
            .height(Fill)
            .padding(10)
            .font(Font::MONOSPACE);

        let preview = markdown(&self.items);

        row![
            editor,
            scrollable(preview).spacing(10).width(Fill).height(Fill)
        ]
        .spacing(10)
        .padding(10)
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::TokyoNight
    }
}

fn markdown<'a>(
    items: impl IntoIterator<Item = &'a Item>,
) -> Element<'a, Message> {
    use iced::padding;

    let blocks = items.into_iter().enumerate().map(|(i, item)| match item {
        Item::Heading(heading) => container(rich_text(heading))
            .padding(padding::top(if i > 0 { 8 } else { 0 }))
            .into(),
        Item::Paragraph(paragraph) => rich_text(paragraph).into(),
        Item::List { start: None, items } => column(
            items
                .iter()
                .map(|item| row!["•", rich_text(item)].spacing(10).into()),
        )
        .spacing(10)
        .into(),
        Item::List {
            start: Some(start),
            items,
        } => column(items.iter().enumerate().map(|(i, item)| {
            row![text!("{}.", i as u64 + *start), rich_text(item)]
                .spacing(10)
                .into()
        }))
        .spacing(10)
        .into(),
        Item::CodeBlock(code) => {
            container(rich_text(code).font(Font::MONOSPACE).size(12))
                .width(Fill)
                .padding(10)
                .style(container::rounded_box)
                .into()
        }
    });

    column(blocks).width(Fill).spacing(16).into()
}

#[derive(Debug, Clone)]
enum Item {
    Heading(Vec<text::Span<'static>>),
    Paragraph(Vec<text::Span<'static>>),
    CodeBlock(Vec<text::Span<'static>>),
    List {
        start: Option<u64>,
        items: Vec<Vec<text::Span<'static>>>,
    },
}

fn parse<'a>(
    markdown: &'a str,
    theme: &'a Theme,
) -> impl Iterator<Item = Item> + 'a {
    use iced::font;
    use iced::highlighter::{self, Highlighter};
    use text::Highlighter as _;

    let mut spans = Vec::new();
    let mut heading = None;
    let mut strong = false;
    let mut emphasis = false;
    let mut link = false;
    let mut list = Vec::new();
    let mut list_start = None;
    let mut highlighter = None;

    let parser = pulldown_cmark::Parser::new(markdown);

    // We want to keep the `spans` capacity
    #[allow(clippy::drain_collect)]
    parser.filter_map(move |event| match event {
        pulldown_cmark::Event::Start(tag) => match tag {
            pulldown_cmark::Tag::Heading { level, .. } => {
                heading = Some(level);
                None
            }
            pulldown_cmark::Tag::Strong => {
                strong = true;
                None
            }
            pulldown_cmark::Tag::Emphasis => {
                emphasis = true;
                None
            }
            pulldown_cmark::Tag::Link { .. } => {
                link = true;
                None
            }
            pulldown_cmark::Tag::List(first_item) => {
                list_start = first_item;
                None
            }
            pulldown_cmark::Tag::CodeBlock(
                pulldown_cmark::CodeBlockKind::Fenced(language),
            ) => {
                highlighter = Some(Highlighter::new(&highlighter::Settings {
                    theme: highlighter::Theme::Base16Ocean,
                    token: language.to_string(),
                }));

                None
            }
            _ => None,
        },
        pulldown_cmark::Event::End(tag) => match tag {
            pulldown_cmark::TagEnd::Heading(_) => {
                heading = None;
                Some(Item::Heading(spans.drain(..).collect()))
            }
            pulldown_cmark::TagEnd::Emphasis => {
                emphasis = false;
                None
            }
            pulldown_cmark::TagEnd::Strong => {
                strong = false;
                None
            }
            pulldown_cmark::TagEnd::Link => {
                link = false;
                None
            }
            pulldown_cmark::TagEnd::Paragraph => {
                Some(Item::Paragraph(spans.drain(..).collect()))
            }
            pulldown_cmark::TagEnd::List(_) => Some(Item::List {
                start: list_start,
                items: list.drain(..).collect(),
            }),
            pulldown_cmark::TagEnd::Item => {
                list.push(spans.drain(..).collect());
                None
            }
            pulldown_cmark::TagEnd::CodeBlock => {
                highlighter = None;
                Some(Item::CodeBlock(spans.drain(..).collect()))
            }
            _ => None,
        },
        pulldown_cmark::Event::Text(text) => {
            if let Some(highlighter) = &mut highlighter {
                for (range, highlight) in
                    highlighter.highlight_line(text.as_ref())
                {
                    let span = span(text[range].to_owned())
                        .color_maybe(highlight.color())
                        .font_maybe(highlight.font());

                    spans.push(span);
                }
            } else {
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

                let span =
                    span.color_maybe(link.then(|| theme.palette().primary));

                spans.push(span);
            }

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
    })
}
