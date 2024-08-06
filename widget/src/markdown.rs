//! Parse and display Markdown.
//!
//! You can enable the `highlighter` feature for syntax highligting
//! in code blocks.
//!
//! Only the variants of [`Item`] are currently supported.
use crate::core::border;
use crate::core::font::{self, Font};
use crate::core::padding;
use crate::core::theme::{self, Theme};
use crate::core::{self, color, Color, Element, Length, Pixels};
use crate::{column, container, rich_text, row, scrollable, span, text};

pub use pulldown_cmark::HeadingLevel;
pub use url::Url;

/// A Markdown item.
#[derive(Debug, Clone)]
pub enum Item {
    /// A heading.
    Heading(pulldown_cmark::HeadingLevel, Vec<text::Span<'static, Url>>),
    /// A paragraph.
    Paragraph(Vec<text::Span<'static, Url>>),
    /// A code block.
    ///
    /// You can enable the `highlighter` feature for syntax highligting.
    CodeBlock(Vec<text::Span<'static, Url>>),
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
    let mut strong = false;
    let mut emphasis = false;
    let mut strikethrough = false;
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
            | pulldown_cmark::Options::ENABLE_TABLES
            | pulldown_cmark::Options::ENABLE_STRIKETHROUGH,
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
            pulldown_cmark::Tag::Strong if !metadata && !table => {
                strong = true;
                None
            }
            pulldown_cmark::Tag::Emphasis if !metadata && !table => {
                emphasis = true;
                None
            }
            pulldown_cmark::Tag::Strikethrough if !metadata && !table => {
                strikethrough = true;
                None
            }
            pulldown_cmark::Tag::Link { dest_url, .. }
                if !metadata && !table =>
            {
                match Url::parse(&dest_url) {
                    Ok(url)
                        if url.scheme() == "http"
                            || url.scheme() == "https" =>
                    {
                        link = Some(url);
                    }
                    _ => {}
                }

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
                lists
                    .last_mut()
                    .expect("list context")
                    .items
                    .push(Vec::new());
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
            pulldown_cmark::TagEnd::Heading(level) if !metadata && !table => {
                produce(
                    &mut lists,
                    Item::Heading(level, spans.drain(..).collect()),
                )
            }
            pulldown_cmark::TagEnd::Strong if !metadata && !table => {
                strong = false;
                None
            }
            pulldown_cmark::TagEnd::Emphasis if !metadata && !table => {
                emphasis = false;
                None
            }
            pulldown_cmark::TagEnd::Strikethrough if !metadata && !table => {
                strikethrough = false;
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
                let list = lists.pop().expect("list context");

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

            let span = span(text.into_string()).strikethrough(strikethrough);

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
                span.color(palette.primary).link(link.clone())
            } else {
                span
            };

            spans.push(span);

            None
        }
        pulldown_cmark::Event::Code(code) if !metadata && !table => {
            let span = span(code.into_string())
                .font(Font::MONOSPACE)
                .color(Color::WHITE)
                .background(color!(0x111111))
                .border(border::rounded(2))
                .padding(padding::left(2).right(2))
                .strikethrough(strikethrough);

            let span = if let Some(link) = link.as_ref() {
                span.color(palette.primary).link(link.clone())
            } else {
                span
            };

            spans.push(span);
            None
        }
        pulldown_cmark::Event::SoftBreak if !metadata && !table => {
            spans.push(span(" ").strikethrough(strikethrough));
            None
        }
        pulldown_cmark::Event::HardBreak if !metadata && !table => {
            spans.push(span("\n"));
            None
        }
        _ => None,
    })
}

/// Configuration controlling Markdown rendering in [`view`].
#[derive(Debug, Clone, Copy)]
pub struct Settings {
    /// The base text size.
    pub text_size: Pixels,
    /// The text size of level 1 heading.
    pub h1_size: Pixels,
    /// The text size of level 2 heading.
    pub h2_size: Pixels,
    /// The text size of level 3 heading.
    pub h3_size: Pixels,
    /// The text size of level 4 heading.
    pub h4_size: Pixels,
    /// The text size of level 5 heading.
    pub h5_size: Pixels,
    /// The text size of level 6 heading.
    pub h6_size: Pixels,
    /// The text size used in code blocks.
    pub code_size: Pixels,
}

impl Settings {
    /// Creates new [`Settings`] with the given base text size in [`Pixels`].
    ///
    /// Heading levels will be adjusted automatically. Specifically,
    /// the first level will be twice the base size, and then every level
    /// after that will be 25% smaller.
    pub fn with_text_size(text_size: impl Into<Pixels>) -> Self {
        let text_size = text_size.into();

        Self {
            text_size,
            h1_size: text_size * 2.0,
            h2_size: text_size * 1.75,
            h3_size: text_size * 1.5,
            h4_size: text_size * 1.25,
            h5_size: text_size,
            h6_size: text_size,
            code_size: text_size * 0.75,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::with_text_size(16)
    }
}

/// Display a bunch of Markdown items.
///
/// You can obtain the items with [`parse`].
pub fn view<'a, Renderer>(
    items: impl IntoIterator<Item = &'a Item>,
    settings: Settings,
) -> Element<'a, Url, Theme, Renderer>
where
    Renderer: core::text::Renderer<Font = Font> + 'a,
{
    let Settings {
        text_size,
        h1_size,
        h2_size,
        h3_size,
        h4_size,
        h5_size,
        h6_size,
        code_size,
    } = settings;

    let spacing = text_size * 0.625;

    let blocks = items.into_iter().enumerate().map(|(i, item)| match item {
        Item::Heading(level, heading) => {
            container(rich_text(heading).size(match level {
                pulldown_cmark::HeadingLevel::H1 => h1_size,
                pulldown_cmark::HeadingLevel::H2 => h2_size,
                pulldown_cmark::HeadingLevel::H3 => h3_size,
                pulldown_cmark::HeadingLevel::H4 => h4_size,
                pulldown_cmark::HeadingLevel::H5 => h5_size,
                pulldown_cmark::HeadingLevel::H6 => h6_size,
            }))
            .padding(padding::top(if i > 0 {
                text_size / 2.0
            } else {
                Pixels::ZERO
            }))
            .into()
        }
        Item::Paragraph(paragraph) => {
            rich_text(paragraph).size(text_size).into()
        }
        Item::List { start: None, items } => {
            column(items.iter().map(|items| {
                row![text("•").size(text_size), view(items, settings)]
                    .spacing(spacing)
                    .into()
            }))
            .spacing(spacing)
            .into()
        }
        Item::List {
            start: Some(start),
            items,
        } => column(items.iter().enumerate().map(|(i, items)| {
            row![
                text!("{}.", i as u64 + *start).size(text_size),
                view(items, settings)
            ]
            .spacing(spacing)
            .into()
        }))
        .spacing(spacing)
        .into(),
        Item::CodeBlock(code) => container(
            scrollable(
                container(
                    rich_text(code).font(Font::MONOSPACE).size(code_size),
                )
                .padding(spacing.0 / 2.0),
            )
            .direction(scrollable::Direction::Horizontal(
                scrollable::Scrollbar::default()
                    .width(spacing.0 / 2.0)
                    .scroller_width(spacing.0 / 2.0),
            )),
        )
        .width(Length::Fill)
        .padding(spacing.0 / 2.0)
        .style(container::dark)
        .into(),
    });

    Element::new(column(blocks).width(Length::Fill).spacing(text_size))
}
