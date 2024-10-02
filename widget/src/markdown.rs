//! Markdown widgets can parse and display Markdown.
//!
//! You can enable the `highlighter` feature for syntax highlighting
//! in code blocks.
//!
//! Only the variants of [`Item`] are currently supported.
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! #
//! use iced::widget::markdown;
//! use iced::Theme;
//!
//! struct State {
//!    markdown: Vec<markdown::Item>,
//! }
//!
//! enum Message {
//!     LinkClicked(markdown::Url),
//! }
//!
//! impl State {
//!     pub fn new() -> Self {
//!         Self {
//!             markdown: markdown::parse("This is some **Markdown**!").collect(),
//!         }
//!     }
//!
//!     fn view(&self) -> Element<'_, Message> {
//!         markdown::view(
//!             &self.markdown,
//!             markdown::Settings::default(),
//!             markdown::Style::from_palette(Theme::TokyoNightStorm.palette()),
//!         )
//!         .map(Message::LinkClicked)
//!         .into()
//!     }
//!
//!     fn update(state: &mut State, message: Message) {
//!         match message {
//!             Message::LinkClicked(url) => {
//!                 println!("The following url was clicked: {url}");
//!             }
//!         }
//!     }
//! }
//! ```
use crate::core::border;
use crate::core::font::{self, Font};
use crate::core::padding;
use crate::core::theme;
use crate::core::{
    self, color, Color, Element, Length, Padding, Pixels, Theme,
};
use crate::{column, container, rich_text, row, scrollable, span, text};

use std::cell::{Cell, RefCell};
use std::sync::Arc;

pub use core::text::Highlight;
pub use pulldown_cmark::HeadingLevel;
pub use url::Url;

/// A Markdown item.
#[derive(Debug, Clone)]
pub enum Item {
    /// A heading.
    Heading(pulldown_cmark::HeadingLevel, Text),
    /// A paragraph.
    Paragraph(Text),
    /// A code block.
    ///
    /// You can enable the `highlighter` feature for syntax highlighting.
    CodeBlock(Text),
    /// A list.
    List {
        /// The first number of the list, if it is ordered.
        start: Option<u64>,
        /// The items of the list.
        items: Vec<Vec<Item>>,
    },
}

/// A bunch of parsed Markdown text.
#[derive(Debug, Clone)]
pub struct Text {
    spans: Vec<Span>,
    last_style: Cell<Option<Style>>,
    last_styled_spans: RefCell<Arc<[text::Span<'static, Url>]>>,
}

impl Text {
    fn new(spans: Vec<Span>) -> Self {
        Self {
            spans,
            last_style: Cell::default(),
            last_styled_spans: RefCell::default(),
        }
    }

    /// Returns the [`rich_text()`] spans ready to be used for the given style.
    ///
    /// This method performs caching for you. It will only reallocate if the [`Style`]
    /// provided changes.
    pub fn spans(&self, style: Style) -> Arc<[text::Span<'static, Url>]> {
        if Some(style) != self.last_style.get() {
            *self.last_styled_spans.borrow_mut() =
                self.spans.iter().map(|span| span.view(&style)).collect();

            self.last_style.set(Some(style));
        }

        self.last_styled_spans.borrow().clone()
    }
}

#[derive(Debug, Clone)]
enum Span {
    Standard {
        text: String,
        strikethrough: bool,
        link: Option<Url>,
        strong: bool,
        emphasis: bool,
        code: bool,
    },
    #[cfg(feature = "highlighter")]
    Highlight {
        text: String,
        color: Option<Color>,
        font: Option<Font>,
    },
}

impl Span {
    fn view(&self, style: &Style) -> text::Span<'static, Url> {
        match self {
            Span::Standard {
                text,
                strikethrough,
                link,
                strong,
                emphasis,
                code,
            } => {
                let span = span(text.clone()).strikethrough(*strikethrough);

                let span = if *code {
                    span.font(Font::MONOSPACE)
                        .color(style.inline_code_color)
                        .background(style.inline_code_highlight.background)
                        .border(style.inline_code_highlight.border)
                        .padding(style.inline_code_padding)
                } else if *strong || *emphasis {
                    span.font(Font {
                        weight: if *strong {
                            font::Weight::Bold
                        } else {
                            font::Weight::Normal
                        },
                        style: if *emphasis {
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
                    span.color(style.link_color).link(link.clone())
                } else {
                    span
                };

                span
            }
            #[cfg(feature = "highlighter")]
            Span::Highlight { text, color, font } => {
                span(text.clone()).color_maybe(*color).font_maybe(*font)
            }
        }
    }
}

/// Parse the given Markdown content.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// #
/// use iced::widget::markdown;
/// use iced::Theme;
///
/// struct State {
///    markdown: Vec<markdown::Item>,
/// }
///
/// enum Message {
///     LinkClicked(markdown::Url),
/// }
///
/// impl State {
///     pub fn new() -> Self {
///         Self {
///             markdown: markdown::parse("This is some **Markdown**!").collect(),
///         }
///     }
///
///     fn view(&self) -> Element<'_, Message> {
///         markdown::view(
///             &self.markdown,
///             markdown::Settings::default(),
///             markdown::Style::from_palette(Theme::TokyoNightStorm.palette()),
///         )
///         .map(Message::LinkClicked)
///         .into()
///     }
///
///     fn update(state: &mut State, message: Message) {
///         match message {
///             Message::LinkClicked(url) => {
///                 println!("The following url was clicked: {url}");
///             }
///         }
///     }
/// }
/// ```
pub fn parse(markdown: &str) -> impl Iterator<Item = Item> + '_ {
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
                    use iced_highlighter::Highlighter;
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
                    Item::Heading(level, Text::new(spans.drain(..).collect())),
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
                produce(
                    &mut lists,
                    Item::Paragraph(Text::new(spans.drain(..).collect())),
                )
            }
            pulldown_cmark::TagEnd::Item if !metadata && !table => {
                if spans.is_empty() {
                    None
                } else {
                    produce(
                        &mut lists,
                        Item::Paragraph(Text::new(spans.drain(..).collect())),
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

                produce(
                    &mut lists,
                    Item::CodeBlock(Text::new(spans.drain(..).collect())),
                )
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
                    let span = Span::Highlight {
                        text: text[range].to_owned(),
                        color: highlight.color(),
                        font: highlight.font(),
                    };

                    spans.push(span);
                }

                return None;
            }

            let span = Span::Standard {
                text: text.into_string(),
                strong,
                emphasis,
                strikethrough,
                link: link.clone(),
                code: false,
            };

            spans.push(span);

            None
        }
        pulldown_cmark::Event::Code(code) if !metadata && !table => {
            let span = Span::Standard {
                text: code.into_string(),
                strong,
                emphasis,
                strikethrough,
                link: link.clone(),
                code: true,
            };

            spans.push(span);
            None
        }
        pulldown_cmark::Event::SoftBreak if !metadata && !table => {
            spans.push(Span::Standard {
                text: String::from(" "),
                strikethrough,
                strong,
                emphasis,
                link: link.clone(),
                code: false,
            });
            None
        }
        pulldown_cmark::Event::HardBreak if !metadata && !table => {
            spans.push(Span::Standard {
                text: String::from("\n"),
                strikethrough,
                strong,
                emphasis,
                link: link.clone(),
                code: false,
            });
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

/// The text styling of some Markdown rendering in [`view`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The [`Highlight`] to be applied to the background of inline code.
    pub inline_code_highlight: Highlight,
    /// The [`Padding`] to be applied to the background of inline code.
    pub inline_code_padding: Padding,
    /// The [`Color`] to be applied to inline code.
    pub inline_code_color: Color,
    /// The [`Color`] to be applied to links.
    pub link_color: Color,
}

impl Style {
    /// Creates a new [`Style`] from the given [`theme::Palette`].
    pub fn from_palette(palette: theme::Palette) -> Self {
        Self {
            inline_code_padding: padding::left(1).right(1),
            inline_code_highlight: Highlight {
                background: color!(0x111).into(),
                border: border::rounded(2),
            },
            inline_code_color: Color::WHITE,
            link_color: palette.primary,
        }
    }
}

/// Display a bunch of Markdown items.
///
/// You can obtain the items with [`parse`].
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// #
/// use iced::widget::markdown;
/// use iced::Theme;
///
/// struct State {
///    markdown: Vec<markdown::Item>,
/// }
///
/// enum Message {
///     LinkClicked(markdown::Url),
/// }
///
/// impl State {
///     pub fn new() -> Self {
///         Self {
///             markdown: markdown::parse("This is some **Markdown**!").collect(),
///         }
///     }
///
///     fn view(&self) -> Element<'_, Message> {
///         markdown::view(
///             &self.markdown,
///             markdown::Settings::default(),
///             markdown::Style::from_palette(Theme::TokyoNightStorm.palette()),
///         )
///         .map(Message::LinkClicked)
///         .into()
///     }
///
///     fn update(state: &mut State, message: Message) {
///         match message {
///             Message::LinkClicked(url) => {
///                 println!("The following url was clicked: {url}");
///             }
///         }
///     }
/// }
/// ```
pub fn view<'a, 'b, Theme, Renderer>(
    items: impl IntoIterator<Item = &'b Item>,
    settings: Settings,
    style: Style,
) -> Element<'a, Url, Theme, Renderer>
where
    Theme: Catalog + 'a,
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
            container(rich_text(heading.spans(style)).size(match level {
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
            rich_text(paragraph.spans(style)).size(text_size).into()
        }
        Item::List { start: None, items } => {
            column(items.iter().map(|items| {
                row![text("•").size(text_size), view(items, settings, style)]
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
                view(items, settings, style)
            ]
            .spacing(spacing)
            .into()
        }))
        .spacing(spacing)
        .into(),
        Item::CodeBlock(code) => container(
            scrollable(
                container(
                    rich_text(code.spans(style))
                        .font(Font::MONOSPACE)
                        .size(code_size),
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
        .class(Theme::code_block())
        .into(),
    });

    Element::new(column(blocks).width(Length::Fill).spacing(text_size))
}

/// The theme catalog of Markdown items.
pub trait Catalog:
    container::Catalog + scrollable::Catalog + text::Catalog
{
    /// The styling class of a Markdown code block.
    fn code_block<'a>() -> <Self as container::Catalog>::Class<'a>;
}

impl Catalog for Theme {
    fn code_block<'a>() -> <Self as container::Catalog>::Class<'a> {
        Box::new(container::dark)
    }
}
