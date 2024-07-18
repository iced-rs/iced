//! Parse and display Markdown.
//!
//! You can enable the `highlighter` feature for syntax highligting
//! in code blocks.
//!
//! Only the variants of [`Item`] are currently supported.
use crate::core::font::{self, Font};
use crate::core::padding;
use crate::core::theme::{self, Theme};
use crate::core::{self, Element, Length};
use crate::{column, container, rich_text, row, span, text};

/// A Markdown item.
#[derive(Debug, Clone)]
pub enum Item {
    /// A heading.
    Heading(Vec<text::Span<'static>>),
    /// A paragraph.
    Paragraph(Vec<text::Span<'static>>),
    /// A code block.
    ///
    /// You can enable the `highlighter` feature for syntax highligting.
    CodeBlock(Vec<text::Span<'static>>),
    /// A list.
    List {
        /// The first number of the list, if it is ordered.
        start: Option<u64>,
        /// The items of the list.
        items: Vec<Vec<text::Span<'static>>>,
    },
}

/// Parse the given Markdown content.
pub fn parse(
    markdown: &str,
    palette: theme::Palette,
) -> impl Iterator<Item = Item> + '_ {
    let mut spans = Vec::new();
    let mut heading = None;
    let mut strong = false;
    let mut emphasis = false;
    let mut link = false;
    let mut list = Vec::new();
    let mut list_start = None;

    #[cfg(feature = "highlighter")]
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
                pulldown_cmark::CodeBlockKind::Fenced(_language),
            ) => {
                #[cfg(feature = "highlighter")]
                {
                    use iced_highlighter::{self, Highlighter};
                    use text::Highlighter as _;

                    highlighter =
                        Some(Highlighter::new(&iced_highlighter::Settings {
                            theme: iced_highlighter::Theme::Base16Ocean,
                            token: _language.to_string(),
                        }));
                }

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
                #[cfg(feature = "highlighter")]
                {
                    highlighter = None;
                }

                Some(Item::CodeBlock(spans.drain(..).collect()))
            }
            _ => None,
        },
        pulldown_cmark::Event::Text(text) => {
            #[cfg(feature = "highlighter")]
            if let Some(highlighter) = &mut highlighter {
                use text::Highlighter as _;

                for (range, highlight) in
                    highlighter.highlight_line(text.as_ref())
                {
                    let span = span(text[range].to_owned())
                        .color_maybe(highlight.color())
                        .font_maybe(highlight.font());

                    spans.push(span);
                }

                return None;
            }

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

            let span = span.color_maybe(link.then_some(palette.primary));

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
    })
}

/// Display a bunch of Markdown items.
///
/// You can obtain the items with [`parse`].
pub fn view<'a, Message, Renderer>(
    items: impl IntoIterator<Item = &'a Item>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: core::text::Renderer<Font = Font> + 'a,
{
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
                .width(Length::Fill)
                .padding(10)
                .style(container::rounded_box)
                .into()
        }
    });

    Element::new(column(blocks).width(Length::Fill).spacing(16))
}
