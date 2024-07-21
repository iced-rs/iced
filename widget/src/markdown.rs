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
    Heading(Vec<text::Span<'static, String>>),
    /// A paragraph.
    Paragraph(Vec<text::Span<'static, String>>),
    /// A code block.
    ///
    /// You can enable the `highlighter` feature for syntax highligting.
    CodeBlock(Vec<text::Span<'static, String>>),
    /// A list.
    List {
        /// The first number of the list, if it is ordered.
        start: Option<u64>,
        /// The items of the list.
        items: Vec<Vec<Item>>,
    },
}

/// Parse the given Markdown content.
pub fn parse(
    markdown: &str,
    palette: theme::Palette,
) -> impl Iterator<Item = Item> + '_ {
    struct List {
        start: Option<u64>,
        items: Vec<Vec<Item>>,
    }

    let mut spans = Vec::new();
    let mut heading = None;
    let mut strong = false;
    let mut emphasis = false;
    let mut metadata = false;
    let mut table = false;
    let mut link = None;
    let mut lists = Vec::new();

    #[cfg(feature = "highlighter")]
    let mut highlighter = None;

    let parser = pulldown_cmark::Parser::new_ext(
        markdown,
        pulldown_cmark::Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
            | pulldown_cmark::Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS
            | pulldown_cmark::Options::ENABLE_TABLES,
    );

    let produce = |lists: &mut Vec<List>, item| {
        if lists.is_empty() {
            Some(item)
        } else {
            lists
                .last_mut()
                .expect("list context")
                .items
                .last_mut()
                .expect("item context")
                .push(item);

            None
        }
    };

    // We want to keep the `spans` capacity
    #[allow(clippy::drain_collect)]
    parser.filter_map(move |event| match event {
        pulldown_cmark::Event::Start(tag) => match tag {
            pulldown_cmark::Tag::Heading { level, .. }
                if !metadata && !table =>
            {
                heading = Some(level);
                None
            }
            pulldown_cmark::Tag::Strong if !metadata && !table => {
                strong = true;
                None
            }
            pulldown_cmark::Tag::Emphasis if !metadata && !table => {
                emphasis = true;
                None
            }
            pulldown_cmark::Tag::Link { dest_url, .. }
                if !metadata && !table =>
            {
                link = Some(dest_url);
                None
            }
            pulldown_cmark::Tag::List(first_item) if !metadata && !table => {
                lists.push(List {
                    start: first_item,
                    items: Vec::new(),
                });

                None
            }
            pulldown_cmark::Tag::Item => {
                lists.last_mut().expect("List").items.push(Vec::new());
                None
            }
            pulldown_cmark::Tag::CodeBlock(
                pulldown_cmark::CodeBlockKind::Fenced(_language),
            ) if !metadata && !table => {
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
            pulldown_cmark::Tag::MetadataBlock(_) => {
                metadata = true;
                None
            }
            pulldown_cmark::Tag::Table(_) => {
                table = true;
                None
            }
            _ => None,
        },
        pulldown_cmark::Event::End(tag) => match tag {
            pulldown_cmark::TagEnd::Heading(_) if !metadata && !table => {
                heading = None;
                produce(&mut lists, Item::Heading(spans.drain(..).collect()))
            }
            pulldown_cmark::TagEnd::Emphasis if !metadata && !table => {
                emphasis = false;
                None
            }
            pulldown_cmark::TagEnd::Strong if !metadata && !table => {
                strong = false;
                None
            }
            pulldown_cmark::TagEnd::Link if !metadata && !table => {
                link = None;
                None
            }
            pulldown_cmark::TagEnd::Paragraph if !metadata && !table => {
                produce(&mut lists, Item::Paragraph(spans.drain(..).collect()))
            }
            pulldown_cmark::TagEnd::Item if !metadata && !table => {
                if spans.is_empty() {
                    None
                } else {
                    produce(
                        &mut lists,
                        Item::Paragraph(spans.drain(..).collect()),
                    )
                }
            }
            pulldown_cmark::TagEnd::List(_) if !metadata && !table => {
                let list = lists.pop().expect("List");

                produce(
                    &mut lists,
                    Item::List {
                        start: list.start,
                        items: list.items,
                    },
                )
            }
            pulldown_cmark::TagEnd::CodeBlock if !metadata && !table => {
                #[cfg(feature = "highlighter")]
                {
                    highlighter = None;
                }

                produce(&mut lists, Item::CodeBlock(spans.drain(..).collect()))
            }
            pulldown_cmark::TagEnd::MetadataBlock(_) => {
                metadata = false;
                None
            }
            pulldown_cmark::TagEnd::Table => {
                table = false;
                None
            }
            _ => None,
        },
        pulldown_cmark::Event::Text(text) if !metadata && !table => {
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

            let span = if let Some(link) = link.as_ref() {
                span.color(palette.primary).link(link.to_string())
            } else {
                span
            };

            spans.push(span);

            None
        }
        pulldown_cmark::Event::Code(code) if !metadata && !table => {
            spans.push(span(code.into_string()).font(Font::MONOSPACE));
            None
        }
        pulldown_cmark::Event::SoftBreak if !metadata && !table => {
            spans.push(span(" "));
            None
        }
        pulldown_cmark::Event::HardBreak if !metadata && !table => {
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
    on_link: impl Fn(String) -> Message + Copy + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: core::text::Renderer<Font = Font> + 'a,
{
    let blocks = items.into_iter().enumerate().map(|(i, item)| match item {
        Item::Heading(heading) => {
            container(rich_text(heading).on_link(on_link))
                .padding(padding::top(if i > 0 { 8 } else { 0 }))
                .into()
        }
        Item::Paragraph(paragraph) => {
            rich_text(paragraph).on_link(on_link).into()
        }
        Item::List { start: None, items } => {
            column(items.iter().map(|items| {
                row!["•", view(items, on_link)].spacing(10).into()
            }))
            .spacing(10)
            .into()
        }
        Item::List {
            start: Some(start),
            items,
        } => column(items.iter().enumerate().map(|(i, items)| {
            row![text!("{}.", i as u64 + *start), view(items, on_link)]
                .spacing(10)
                .into()
        }))
        .spacing(10)
        .into(),
        Item::CodeBlock(code) => container(
            rich_text(code)
                .font(Font::MONOSPACE)
                .size(12)
                .on_link(on_link),
        )
        .width(Length::Fill)
        .padding(10)
        .style(container::rounded_box)
        .into(),
    });

    Element::new(column(blocks).width(Length::Fill).spacing(16))
}
