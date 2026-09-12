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
//!     LinkClicked(markdown::Uri),
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
//!             Theme::TokyoNight,
//!         )
//!             .map(Message::LinkClicked)
//!             .into()
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
use crate::core;
use crate::core::alignment;
use crate::core::border;
use crate::core::font::{self, Font};
use crate::core::padding;
use crate::core::text::LineHeight;
use crate::core::theme;
use crate::core::{Code, Color, Element, Length, Padding, Pixels, Theme};
use crate::{checkbox, column, container, rich_text, row, rule, scrollable, span, text};

use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::mem;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

pub use core::text::{Highlight, Highlighter};
pub use pulldown_cmark::HeadingLevel;

/// A [`String`] representing a [URI] in a Markdown document
///
/// [URI]: https://en.wikipedia.org/wiki/Uniform_Resource_Identifier
pub type Uri = String;

/// A bunch of Markdown that has been parsed.
#[derive(Debug, Default)]
pub struct Content {
    items: Vec<Item>,
    incomplete: HashMap<usize, Section>,
    state: State,
}

#[derive(Debug)]
struct Section {
    content: String,
    broken_links: HashSet<String>,
}

impl Content {
    /// Creates a new empty [`Content`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates some new [`Content`] by parsing the given Markdown.
    pub fn parse(markdown: &str) -> Self {
        let mut content = Self::new();
        content.push_str(markdown);
        content
    }

    /// Pushes more Markdown into the [`Content`]; parsing incrementally!
    ///
    /// This is specially useful when you have long streams of Markdown; like
    /// big files or potentially long replies.
    pub fn push_str(&mut self, markdown: &str) {
        if markdown.is_empty() {
            return;
        }

        // Append to last leftover text
        let mut leftover = std::mem::take(&mut self.state.leftover);
        leftover.push_str(markdown);

        let input = if leftover.trim_end().ends_with('|') {
            leftover.trim_end().trim_end_matches('|')
        } else {
            leftover.as_str()
        };

        // Pop the last item
        let _ = self.items.pop();

        // Re-parse last item and new text
        for (item, source, broken_links) in parse_with(&mut self.state, input) {
            if !broken_links.is_empty() {
                let _ = self.incomplete.insert(
                    self.items.len(),
                    Section {
                        content: source.to_owned(),
                        broken_links,
                    },
                );
            }

            self.items.push(item);
        }

        self.state.leftover.push_str(&leftover[input.len()..]);

        // Re-parse incomplete sections if new references are available
        if !self.incomplete.is_empty() {
            self.incomplete.retain(|index, section| {
                if self.items.len() <= *index {
                    return false;
                }

                let broken_links_before = section.broken_links.len();

                section
                    .broken_links
                    .retain(|link| !self.state.references.contains_key(link));

                if broken_links_before != section.broken_links.len() {
                    let mut state = State {
                        leftover: String::new(),
                        references: self.state.references.clone(),
                        images: HashSet::new(),
                        #[cfg(feature = "highlighter")]
                        parser: None,
                    };

                    if let Some((item, _source, _broken_links)) =
                        parse_with(&mut state, &section.content).next()
                    {
                        self.items[*index] = item;
                    }

                    self.state.images.extend(state.images.drain());
                    drop(state);
                }

                !section.broken_links.is_empty()
            });
        }
    }

    /// Returns the Markdown items, ready to be rendered.
    ///
    /// You can use [`view`] to turn them into an [`Element`].
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    /// Returns the URLs of the Markdown images present in the [`Content`].
    pub fn images(&self) -> &HashSet<Uri> {
        &self.state.images
    }
}

/// Groups the given Markdown [`Item`]s by [`Item::Heading`].
///
/// The returned iterator yields a `(Option<&Item>, &[Item])` pair for each
/// group, without cloning any [`Item`]:
///
/// * The first element is the heading that starts the group, if any. It is
///   [`None`] for the group of items that appears before the first heading,
///   if there is any;
/// * The second element is the slice of items that follow the heading, up to
///   (but not including) the next one.
///
/// Every item in the given slice is yielded exactly once: a heading is
/// returned as the first element of the group it starts, and every other
/// item is part of the slice that follows the last heading before it.
///
/// # Example
/// ```
/// use iced_widget::markdown;
///
/// let items: Vec<_> = markdown::parse("# Title\n\nHello!\n\n# Subtitle\n\nMore!").collect();
///
/// let mut groups = markdown::sections(&items);
///
/// let (heading, contents) = groups.next().unwrap();
/// assert!(heading.is_some());
/// assert_eq!(contents.len(), 1);
///
/// let (heading, contents) = groups.next().unwrap();
/// assert!(heading.is_some());
/// assert_eq!(contents.len(), 1);
///
/// assert!(groups.next().is_none());
/// ```
pub fn sections<'a>(
    items: &'a [Item],
) -> impl Iterator<Item = (Option<&'a Item>, &'a [Item])> + 'a {
    struct Sections<'a> {
        /// The items being grouped.
        items: &'a [Item],
        /// The index of the first item of the next group.
        ///
        /// This is always the index of a heading, except for the very first
        /// group, where it may point at any item (the content before the
        /// first heading).
        start: usize,
    }

    impl<'a> Iterator for Sections<'a> {
        type Item = (Option<&'a Item>, &'a [Item]);

        fn next(&mut self) -> Option<Self::Item> {
            let Self { items, start } = self;

            if *start >= items.len() {
                return None;
            }

            // The heading of the current group, if any
            let heading = if matches!(items[*start], Item::Heading(..)) {
                Some(*start)
            } else {
                None
            };

            // The body of the group: the items after the heading, if any, up to
            // the next heading
            let body_start = heading.map_or(*start, |heading| heading + 1);
            let next_heading = (body_start..items.len())
                .find(|&index| matches!(items[index], Item::Heading(..)))
                .unwrap_or(items.len());

            // The next group starts at the next heading, if any
            *start = next_heading;

            Some((
                heading.map(|index| &items[index]),
                &items[body_start..next_heading],
            ))
        }
    }

    Sections { items, start: 0 }
}

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
    CodeBlock {
        /// The language of the code block, if any.
        language: Option<String>,
        /// The raw code of the code block.
        code: String,
        /// The styled lines of text in the code block.
        lines: Vec<Text>,
    },
    /// A list.
    List {
        /// The first number of the list, if it is ordered.
        start: Option<u64>,
        /// The items of the list.
        bullets: Vec<Bullet>,
    },
    /// An image.
    Image {
        /// The destination URL of the image.
        url: Uri,
        /// The title of the image.
        title: String,
        /// The alternative text of the image.
        alt: Text,
    },
    /// A quote.
    Quote(Vec<Item>),
    /// A horizontal separator.
    Rule,
    /// A table.
    Table {
        /// The columns of the table.
        columns: Vec<Column>,
        /// The rows of the table.
        rows: Vec<Row>,
    },
}

/// The column of a table.
#[derive(Debug, Clone)]
pub struct Column {
    /// The header of the column.
    pub header: Vec<Item>,
    /// The alignment of the column.
    pub alignment: pulldown_cmark::Alignment,
}

/// The row of a table.
#[derive(Debug, Clone)]
pub struct Row {
    /// The cells of the row.
    cells: Vec<Vec<Item>>,
}

/// A bunch of parsed Markdown text.
#[derive(Debug, Clone)]
pub struct Text {
    spans: Vec<Span>,
    last_style: RefCell<Option<(Settings, String, String)>>,
    last_styled_spans: RefCell<Arc<[text::Span<'static, Uri>]>>,
}

impl Text {
    fn new(spans: Vec<Span>) -> Self {
        Self {
            spans,
            last_style: RefCell::default(),
            last_styled_spans: RefCell::default(),
        }
    }

    /// Returns the [`rich_text()`] spans ready to be used for the given style.
    ///
    /// This method performs caching for you. It will only reallocate if the [`Settings`]
    /// or the [`Catalog`] provided changes.
    pub fn spans<Theme: Catalog>(
        &self,
        settings: Settings,
        theme: &Theme,
        highlighter: &dyn text::Highlighter<Code, Theme>,
    ) -> Arc<[text::Span<'static, Uri>]> {
        let is_dirty = self.last_style.borrow().as_ref().is_none_or(
            |(last_settings, last_theme, last_highlighter)| {
                &settings != last_settings
                    || theme.id() != last_theme
                    || highlighter.id() != last_highlighter
            },
        );

        if is_dirty {
            *self.last_styled_spans.borrow_mut() = self
                .spans
                .iter()
                .map(|span| span.view(&settings, theme, highlighter))
                .collect();

            *self.last_style.borrow_mut() =
                Some((settings, theme.id().to_owned(), highlighter.id().to_owned()));
        }

        self.last_styled_spans.borrow().clone()
    }
}

#[derive(Debug, Clone)]
enum Span {
    Standard {
        text: String,
        strikethrough: bool,
        link: Option<Uri>,
        strong: bool,
        emphasis: bool,
        inline_code: bool,
    },
    Code {
        text: String,
        code: Code,
    },
}

impl Span {
    fn view<Theme: Catalog>(
        &self,
        settings: &Settings,
        theme: &Theme,
        highlighter: &dyn text::Highlighter<Code, Theme>,
    ) -> text::Span<'static, Uri> {
        match self {
            Span::Standard {
                text,
                strikethrough,
                link,
                strong,
                emphasis,
                inline_code,
            } => {
                let span = span(text.clone()).strikethrough(*strikethrough);

                let weight = if *strong {
                    font::Weight::Bold
                } else {
                    settings.font.weight
                };

                let style = if *emphasis {
                    font::Style::Italic
                } else {
                    settings.font.style
                };

                let span = if *inline_code {
                    let code = theme.code();

                    span.font(Font {
                        weight,
                        style,
                        ..settings.inline_code_font
                    })
                    .size(settings.inline_code_size)
                    .color(code.color)
                    .background(code.highlight.background)
                    .border(code.highlight.border)
                    .padding(code.padding)
                } else {
                    span.font(Font {
                        weight,
                        style,
                        ..settings.font
                    })
                };

                if let Some(link) = link.as_ref() {
                    span.color(theme.link_color()).link(link.clone())
                } else {
                    span
                }
            }
            Span::Code { text, code } => {
                let format = highlighter.highlight(*code, theme);

                span(text.clone())
                    .color_maybe(format.color)
                    .font_maybe(format.style.map(|style| Font {
                        style,
                        ..settings.code_block_font
                    }))
            }
        }
    }
}

/// The item of a list.
#[derive(Debug, Clone)]
pub enum Bullet {
    /// A simple bullet point.
    Point {
        /// The contents of the bullet point.
        items: Vec<Item>,
    },
    /// A task.
    Task {
        /// The contents of the task.
        items: Vec<Item>,
        /// Whether the task is done or not.
        done: bool,
    },
}

impl Bullet {
    fn items(&self) -> &[Item] {
        match self {
            Bullet::Point { items } | Bullet::Task { items, .. } => items,
        }
    }

    fn push(&mut self, item: Item) {
        let (Bullet::Point { items } | Bullet::Task { items, .. }) = self;

        items.push(item);
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
///     LinkClicked(markdown::Uri),
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
///             Theme::TokyoNight,
///         )
///             .map(Message::LinkClicked)
///             .into()
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
    parse_with(State::default(), markdown).map(|(item, _source, _broken_links)| item)
}

#[derive(Debug, Default)]
struct State {
    leftover: String,
    references: HashMap<String, String>,
    images: HashSet<Uri>,
    #[cfg(feature = "highlighter")]
    parser: Option<code::Parser>,
}

fn parse_with<'a>(
    mut state: impl BorrowMut<State> + 'a,
    markdown: &'a str,
) -> impl Iterator<Item = (Item, &'a str, HashSet<String>)> + 'a {
    enum Scope {
        List(List),
        Quote(Vec<Item>),
        Table {
            alignment: Vec<pulldown_cmark::Alignment>,
            columns: Vec<Column>,
            rows: Vec<Row>,
            current: Vec<Item>,
        },
    }

    struct List {
        start: Option<u64>,
        bullets: Vec<Bullet>,
    }

    let broken_links = Rc::new(RefCell::new(HashSet::new()));

    let mut spans = Vec::new();
    let mut code = String::new();
    let mut code_language = None;
    let mut code_lines = Vec::new();
    let mut strong = false;
    let mut emphasis = false;
    let mut strikethrough = false;
    let mut metadata = false;
    let mut code_block = false;
    let mut link = None;
    let mut image = None;
    let mut stack = Vec::new();

    #[cfg(feature = "highlighter")]
    let mut code_parser = None;

    let parser = pulldown_cmark::Parser::new_with_broken_link_callback(
        markdown,
        pulldown_cmark::Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
            | pulldown_cmark::Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS
            | pulldown_cmark::Options::ENABLE_TABLES
            | pulldown_cmark::Options::ENABLE_STRIKETHROUGH
            | pulldown_cmark::Options::ENABLE_TASKLISTS,
        {
            let references = state.borrow().references.clone();
            let broken_links = broken_links.clone();

            Some(move |broken_link: pulldown_cmark::BrokenLink<'_>| {
                if let Some(reference) = references.get(broken_link.reference.as_ref()) {
                    Some((
                        pulldown_cmark::CowStr::from(reference.to_owned()),
                        broken_link.reference.into_static(),
                    ))
                } else {
                    let _ = RefCell::borrow_mut(&broken_links)
                        .insert(broken_link.reference.into_string());

                    None
                }
            })
        },
    );

    let references = &mut state.borrow_mut().references;

    for reference in parser.reference_definitions().iter() {
        let _ = references.insert(reference.0.to_owned(), reference.1.dest.to_string());
    }

    let produce = move |state: &mut State, stack: &mut Vec<Scope>, item, source: Range<usize>| {
        if let Some(scope) = stack.last_mut() {
            match scope {
                Scope::List(list) => {
                    list.bullets.last_mut().expect("item context").push(item);
                }
                Scope::Quote(items) => {
                    items.push(item);
                }
                Scope::Table { current, .. } => {
                    current.push(item);
                }
            }

            None
        } else {
            state.leftover = markdown[source.start..].to_owned();

            Some((
                item,
                &markdown[source.start..source.end],
                broken_links.take(),
            ))
        }
    };

    let parser = parser.into_offset_iter();

    // We want to keep the `spans` capacity
    #[allow(clippy::drain_collect)]
    parser.filter_map(move |(event, source)| match event {
        pulldown_cmark::Event::Start(tag) => match tag {
            pulldown_cmark::Tag::Strong if !metadata => {
                strong = true;
                None
            }
            pulldown_cmark::Tag::Emphasis if !metadata => {
                emphasis = true;
                None
            }
            pulldown_cmark::Tag::Strikethrough if !metadata => {
                strikethrough = true;
                None
            }
            pulldown_cmark::Tag::Link { dest_url, .. } if !metadata => {
                link = Some(dest_url.into_string());
                None
            }
            pulldown_cmark::Tag::Image {
                dest_url, title, ..
            } if !metadata => {
                image = Some((dest_url.into_string(), title.into_string()));
                None
            }
            pulldown_cmark::Tag::List(first_item) if !metadata => {
                let prev = if spans.is_empty() {
                    None
                } else {
                    produce(
                        state.borrow_mut(),
                        &mut stack,
                        Item::Paragraph(Text::new(spans.drain(..).collect())),
                        source,
                    )
                };

                stack.push(Scope::List(List {
                    start: first_item,
                    bullets: Vec::new(),
                }));

                prev
            }
            pulldown_cmark::Tag::Item => {
                if let Some(Scope::List(list)) = stack.last_mut() {
                    list.bullets.push(Bullet::Point { items: Vec::new() });
                }

                None
            }
            pulldown_cmark::Tag::BlockQuote(_kind) if !metadata => {
                let prev = if spans.is_empty() {
                    None
                } else {
                    produce(
                        state.borrow_mut(),
                        &mut stack,
                        Item::Paragraph(Text::new(spans.drain(..).collect())),
                        source,
                    )
                };

                stack.push(Scope::Quote(Vec::new()));

                prev
            }
            pulldown_cmark::Tag::CodeBlock(pulldown_cmark::CodeBlockKind::Fenced(language))
                if !metadata =>
            {
                #[cfg(feature = "highlighter")]
                {
                    code_parser = Some({
                        let mut code_parser = state
                            .borrow_mut()
                            .parser
                            .take()
                            .filter(|parser| parser.language() == language.as_ref())
                            .unwrap_or_else(|| {
                                code::Parser::new(language.split(',').next().unwrap_or_default())
                            });

                        code_parser.prepare();

                        code_parser
                    });
                }

                code_block = true;
                code_language = (!language.is_empty()).then(|| language.into_string());

                if spans.is_empty() {
                    None
                } else {
                    produce(
                        state.borrow_mut(),
                        &mut stack,
                        Item::Paragraph(Text::new(spans.drain(..).collect())),
                        source,
                    )
                }
            }
            pulldown_cmark::Tag::MetadataBlock(_) => {
                metadata = true;
                None
            }
            pulldown_cmark::Tag::Table(alignment) => {
                stack.push(Scope::Table {
                    columns: Vec::with_capacity(alignment.len()),
                    alignment,
                    current: Vec::new(),
                    rows: Vec::new(),
                });

                None
            }
            pulldown_cmark::Tag::TableHead => {
                strong = true;
                None
            }
            pulldown_cmark::Tag::TableRow => {
                let Scope::Table { rows, .. } = stack.last_mut()? else {
                    return None;
                };

                rows.push(Row { cells: Vec::new() });
                None
            }
            _ => None,
        },
        pulldown_cmark::Event::End(tag) => match tag {
            pulldown_cmark::TagEnd::Heading(level) if !metadata => produce(
                state.borrow_mut(),
                &mut stack,
                Item::Heading(level, Text::new(spans.drain(..).collect())),
                source,
            ),
            pulldown_cmark::TagEnd::Strong if !metadata => {
                strong = false;
                None
            }
            pulldown_cmark::TagEnd::Emphasis if !metadata => {
                emphasis = false;
                None
            }
            pulldown_cmark::TagEnd::Strikethrough if !metadata => {
                strikethrough = false;
                None
            }
            pulldown_cmark::TagEnd::Link if !metadata => {
                link = None;
                None
            }
            pulldown_cmark::TagEnd::Paragraph if !metadata => {
                if spans.is_empty() {
                    None
                } else {
                    produce(
                        state.borrow_mut(),
                        &mut stack,
                        Item::Paragraph(Text::new(spans.drain(..).collect())),
                        source,
                    )
                }
            }
            pulldown_cmark::TagEnd::Item if !metadata => {
                if spans.is_empty() {
                    None
                } else {
                    produce(
                        state.borrow_mut(),
                        &mut stack,
                        Item::Paragraph(Text::new(spans.drain(..).collect())),
                        source,
                    )
                }
            }
            pulldown_cmark::TagEnd::List(_) if !metadata => {
                let scope = stack.pop()?;

                let Scope::List(list) = scope else {
                    return None;
                };

                produce(
                    state.borrow_mut(),
                    &mut stack,
                    Item::List {
                        start: list.start,
                        bullets: list.bullets,
                    },
                    source,
                )
            }
            pulldown_cmark::TagEnd::BlockQuote(_kind) if !metadata => {
                let scope = stack.pop()?;

                let Scope::Quote(quote) = scope else {
                    return None;
                };

                produce(state.borrow_mut(), &mut stack, Item::Quote(quote), source)
            }
            pulldown_cmark::TagEnd::Image if !metadata => {
                let (url, title) = image.take()?;
                let alt = Text::new(spans.drain(..).collect());

                let state = state.borrow_mut();
                let _ = state.images.insert(url.clone());

                produce(state, &mut stack, Item::Image { url, title, alt }, source)
            }
            pulldown_cmark::TagEnd::CodeBlock if !metadata => {
                code_block = false;

                #[cfg(feature = "highlighter")]
                {
                    state.borrow_mut().parser = code_parser.take();
                }

                produce(
                    state.borrow_mut(),
                    &mut stack,
                    Item::CodeBlock {
                        language: code_language.take(),
                        code: mem::take(&mut code),
                        lines: code_lines.drain(..).collect(),
                    },
                    source,
                )
            }
            pulldown_cmark::TagEnd::MetadataBlock(_) => {
                metadata = false;
                None
            }
            pulldown_cmark::TagEnd::Table => {
                let scope = stack.pop()?;

                let Scope::Table { columns, rows, .. } = scope else {
                    return None;
                };

                produce(
                    state.borrow_mut(),
                    &mut stack,
                    Item::Table { columns, rows },
                    source,
                )
            }
            pulldown_cmark::TagEnd::TableHead => {
                strong = false;
                None
            }
            pulldown_cmark::TagEnd::TableCell => {
                if !spans.is_empty() {
                    let _ = produce(
                        state.borrow_mut(),
                        &mut stack,
                        Item::Paragraph(Text::new(spans.drain(..).collect())),
                        source,
                    );
                }

                let Scope::Table {
                    alignment,
                    columns,
                    rows,
                    current,
                } = stack.last_mut()?
                else {
                    return None;
                };

                if columns.len() < alignment.len() {
                    columns.push(Column {
                        header: std::mem::take(current),
                        alignment: alignment[columns.len()],
                    });
                } else {
                    rows.last_mut()
                        .expect("table row")
                        .cells
                        .push(std::mem::take(current));
                }

                None
            }
            _ => None,
        },
        pulldown_cmark::Event::Text(text) if !metadata => {
            if code_block {
                code.push_str(&text);

                #[cfg(feature = "highlighter")]
                if let Some(highlighter) = &mut code_parser {
                    for line in text.lines() {
                        code_lines.push(Text::new(highlighter.parse_line(line).to_vec()));
                    }
                }

                #[cfg(not(feature = "highlighter"))]
                for line in text.lines() {
                    code_lines.push(Text::new(vec![Span::Code {
                        text: line.to_owned(),
                        code: Code::Other,
                    }]));
                }

                return None;
            }

            let span = Span::Standard {
                text: text.into_string(),
                strong,
                emphasis,
                strikethrough,
                link: link.clone(),
                inline_code: false,
            };

            spans.push(span);

            None
        }
        pulldown_cmark::Event::Code(code) if !metadata => {
            let span = Span::Standard {
                text: code.into_string(),
                strong,
                emphasis,
                strikethrough,
                link: link.clone(),
                inline_code: true,
            };

            spans.push(span);
            None
        }
        pulldown_cmark::Event::SoftBreak if !metadata => {
            spans.push(Span::Standard {
                text: String::from(" "),
                strikethrough,
                strong,
                emphasis,
                link: link.clone(),
                inline_code: false,
            });
            None
        }
        pulldown_cmark::Event::HardBreak if !metadata => {
            spans.push(Span::Standard {
                text: String::from("\n"),
                strikethrough,
                strong,
                emphasis,
                link: link.clone(),
                inline_code: false,
            });
            None
        }
        pulldown_cmark::Event::Rule => produce(state.borrow_mut(), &mut stack, Item::Rule, source),
        pulldown_cmark::Event::TaskListMarker(done) => {
            if let Some(Scope::List(list)) = stack.last_mut()
                && let Some(item) = list.bullets.last_mut()
                && let Bullet::Point { items } = item
            {
                *item = Bullet::Task {
                    items: std::mem::take(items),
                    done,
                };
            }

            None
        }
        _ => None,
    })
}

/// Configuration controlling Markdown rendering in [`view`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    /// The [`Font`] to be applied to basic text.
    pub font: Font,
    /// The [`Font`] to be applied to inline code.
    pub inline_code_font: Font,
    /// The [`Font`] to be applied to code blocks.
    pub code_block_font: Font,
    /// The base line height.
    pub line_height: LineHeight,
    /// The base text size.
    pub text_size: Pixels,
    /// The text size used in code blocks.
    pub code_block_size: Pixels,
    /// The text size used in inline code.
    pub inline_code_size: Pixels,
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
    /// The spacing to be used between elements.
    pub spacing: Pixels,
}

impl Settings {
    /// Creates new [`Settings`] with the given base text size in [`Pixels`].
    ///
    /// Heading levels will be adjusted automatically. Specifically,
    /// the first level will be 1.5 times the base size, the second
    /// 1.25 times, the third 1.125 times, and the remaining levels
    /// will use the base size.
    pub fn with_text_size(text_size: impl Into<Pixels>) -> Self {
        let text_size = text_size.into();
        let line_height = LineHeight::default();

        Self {
            font: Font::DEFAULT,
            inline_code_font: Font::MONOSPACE,
            code_block_font: Font::MONOSPACE,
            line_height,
            text_size,
            inline_code_size: text_size * 0.85,
            code_block_size: text_size * 0.85,
            h1_size: text_size * 1.5,
            h2_size: text_size * 1.25,
            h3_size: text_size * 1.125,
            h4_size: text_size,
            h5_size: text_size,
            h6_size: text_size,
            spacing: line_height.to_absolute(text_size) / 1.5,
        }
    }

    /// Sets the [`LineHeight`] of the [`Settings`].
    pub fn line_height(self, line_height: impl Into<LineHeight>) -> Self {
        let line_height = line_height.into();

        Self {
            line_height,
            spacing: line_height.to_absolute(self.text_size) / 1.5,
            ..self
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
///     LinkClicked(markdown::Uri),
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
///             Theme::TokyoNight,
///         )
///             .map(Message::LinkClicked)
///             .into()
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
pub fn view<'a, Theme, Renderer>(
    items: &'a [Item],
    settings: impl Into<Settings>,
    theme: Theme,
) -> Element<'a, Uri, Theme, Renderer>
where
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer + 'a,
{
    view_with(
        items,
        settings,
        &DefaultViewer {
            theme,
            highlighter: None,
        },
    )
}

/// Runs [`view`] but with a custom [`Viewer`] to turn an [`Item`] into
/// an [`Element`].
///
/// This is useful if you want to customize the look of certain Markdown
/// elements.
pub fn view_with<'a, Message, Theme, Renderer>(
    items: &'a [Item],
    settings: impl Into<Settings>,
    viewer: &impl Viewer<'a, Message, Theme, Renderer>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer + 'a,
{
    self::items(viewer, settings.into(), items)
}

/// Displays an [`Item`] using the given [`Viewer`].
pub fn item<'a, Message, Theme, Renderer>(
    viewer: &impl Viewer<'a, Message, Theme, Renderer>,
    settings: Settings,
    item: &'a Item,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer + 'a,
{
    match item {
        Item::Image { url, title, alt } => viewer.image(settings, url, title, alt),
        Item::Heading(level, text) => viewer.heading(settings, level, text),
        Item::Paragraph(text) => viewer.paragraph(settings, text),
        Item::CodeBlock {
            language,
            code,
            lines,
        } => viewer.code_block(settings, language.as_deref(), code, lines),
        Item::List {
            start: None,
            bullets,
        } => viewer.unordered_list(settings, bullets),
        Item::List {
            start: Some(start),
            bullets,
        } => viewer.ordered_list(settings, *start, bullets),
        Item::Quote(quote) => viewer.quote(settings, quote),
        Item::Rule => viewer.rule(),
        Item::Table { columns, rows } => viewer.table(settings, columns, rows),
    }
}

/// Displays a heading using the default look.
pub fn heading<'a, Message, Theme, Renderer>(
    viewer: &impl Viewer<'a, Message, Theme, Renderer>,
    settings: Settings,
    level: &'a HeadingLevel,
    text: &'a Text,
    on_link_click: impl Fn(Uri) -> Message + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer + 'a,
{
    let Settings {
        h1_size,
        h2_size,
        h3_size,
        h4_size,
        h5_size,
        h6_size,
        ..
    } = settings;

    let size = match level {
        pulldown_cmark::HeadingLevel::H1 => h1_size,
        pulldown_cmark::HeadingLevel::H2 => h2_size,
        pulldown_cmark::HeadingLevel::H3 => h3_size,
        pulldown_cmark::HeadingLevel::H4 => h4_size,
        pulldown_cmark::HeadingLevel::H5 => h5_size,
        pulldown_cmark::HeadingLevel::H6 => h6_size,
    };

    container(
        rich_text(text.spans(
            Settings {
                font: Font {
                    weight: font::Weight::Bold,
                    ..settings.font
                },
                inline_code_font: Font {
                    weight: font::Weight::Bold,
                    ..settings.inline_code_font
                },
                inline_code_size: size * (settings.inline_code_size / settings.text_size),
                ..settings
            },
            viewer.theme(),
            viewer.highlighter(),
        ))
        .on_link_click(on_link_click)
        .size(size)
        .line_height(settings.line_height),
    )
    .into()
}

/// Displays a paragraph using the default look.
pub fn paragraph<'a, Message, Theme, Renderer>(
    viewer: &impl Viewer<'a, Message, Theme, Renderer>,
    settings: Settings,
    text: &Text,
    on_link_click: impl Fn(Uri) -> Message + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer + 'a,
{
    rich_text(text.spans(settings, viewer.theme(), viewer.highlighter()))
        .size(settings.text_size)
        .line_height(settings.line_height)
        .on_link_click(on_link_click)
        .into()
}

/// Displays an unordered list using the default look and
/// calling the [`Viewer`] for each bullet point item.
pub fn unordered_list<'a, Message, Theme, Renderer>(
    viewer: &impl Viewer<'a, Message, Theme, Renderer>,
    settings: Settings,
    bullets: &'a [Bullet],
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer + 'a,
{
    column(bullets.iter().map(|bullet| {
        row![
            match bullet {
                Bullet::Point { .. } => {
                    text("•").size(settings.text_size).into()
                }
                Bullet::Task { done, .. } => {
                    Element::from(
                        container(checkbox(*done).size(settings.text_size))
                            .center_y(text::LineHeight::default().to_absolute(settings.text_size)),
                    )
                }
            },
            items(
                viewer,
                Settings {
                    spacing: settings.spacing / 2.0,
                    ..settings
                },
                bullet.items(),
            )
        ]
        .spacing(settings.text_size / 2.0)
        .into()
    }))
    .spacing(settings.spacing / 2.0)
    .padding(padding::left(settings.text_size.0))
    .into()
}

/// Displays an ordered list using the default look and
/// calling the [`Viewer`] for each numbered item.
pub fn ordered_list<'a, Message, Theme, Renderer>(
    viewer: &impl Viewer<'a, Message, Theme, Renderer>,
    settings: Settings,
    start: u64,
    bullets: &'a [Bullet],
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer + 'a,
{
    let digits = (start + bullets.len() as u64).max(1).ilog10() + 1;

    column(bullets.iter().enumerate().map(|(i, bullet)| {
        row![
            text!("{}.", i as u64 + start)
                .size(settings.text_size)
                .align_x(alignment::Horizontal::Right)
                .width(settings.text_size * ((digits as f32 / 2.0).ceil() + 1.0)),
            items(
                viewer,
                Settings {
                    spacing: settings.spacing / 2.0,
                    ..settings
                },
                bullet.items(),
            )
        ]
        .spacing(settings.text_size / 2.0)
        .into()
    }))
    .spacing(settings.spacing / 2.0)
    .into()
}

/// Displays a code block using the default look.
pub fn code_block<'a, Message, Theme, Renderer>(
    viewer: &impl Viewer<'a, Message, Theme, Renderer>,
    settings: Settings,
    lines: &'a [Text],
    on_link_click: impl Fn(Uri) -> Message + Clone + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer + 'a,
{
    container(
        scrollable(column(lines.iter().map(|line| {
            rich_text(line.spans(settings, viewer.theme(), viewer.highlighter()))
                .on_link_click(on_link_click.clone())
                .font(settings.code_block_font)
                .size(settings.code_block_size)
                .line_height(settings.line_height)
                .into()
        })))
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::default()
                .width(settings.code_block_size / 2)
                .scroller_width(settings.code_block_size / 2),
        ))
        .spacing(settings.spacing * 0.75),
    )
    .width(Length::Fill)
    .padding(settings.spacing * 0.75)
    .class(Theme::code_block())
    .into()
}

/// Displays a quote using the default look.
pub fn quote<'a, Message, Theme, Renderer>(
    viewer: &impl Viewer<'a, Message, Theme, Renderer>,
    settings: Settings,
    contents: &'a [Item],
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer + 'a,
{
    container(
        column(
            contents
                .iter()
                .map(|content| item(viewer, settings, content)),
        )
        .spacing(settings.spacing.0),
    )
    .width(Length::Fill)
    .padding(settings.spacing.0)
    .class(Theme::quote())
    .into()
}

/// Displays a rule using the default look.
pub fn rule<'a, Message, Theme, Renderer>() -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer + 'a,
{
    rule::horizontal(2).into()
}

/// Displays a table using the default look.
pub fn table<'a, Message, Theme, Renderer>(
    viewer: &impl Viewer<'a, Message, Theme, Renderer>,
    settings: Settings,
    columns: &'a [Column],
    rows: &'a [Row],
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer + 'a,
{
    use crate::table;

    let table = table(
        columns.iter().enumerate().map(move |(i, column)| {
            table::column(items(viewer, settings, &column.header), move |row: &Row| {
                if let Some(cells) = row.cells.get(i) {
                    items(viewer, settings, cells)
                } else {
                    text("").into()
                }
            })
            .align_x(match column.alignment {
                pulldown_cmark::Alignment::None | pulldown_cmark::Alignment::Left => {
                    alignment::Horizontal::Left
                }
                pulldown_cmark::Alignment::Center => alignment::Horizontal::Center,
                pulldown_cmark::Alignment::Right => alignment::Horizontal::Right,
            })
        }),
        rows,
    )
    .padding_x(settings.spacing.0)
    .padding_y(settings.spacing.0 / 2.0)
    .separator_x(0);

    scrollable(table)
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::default(),
        ))
        .spacing(settings.spacing.0 / 2.0)
        .into()
}

/// Displays a column of items with the default look.
pub fn items<'a, Message, Theme, Renderer>(
    viewer: &impl Viewer<'a, Message, Theme, Renderer>,
    settings: Settings,
    items: &'a [Item],
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer + 'a,
{
    column(sections(items).map(|(heading, contents)| {
        let contents = column(
            contents
                .iter()
                .map(|content| item(viewer, settings, content)),
        )
        .spacing(settings.spacing)
        .into();

        if let Some(heading) = heading {
            column![item(viewer, settings, heading), contents]
                .spacing(settings.spacing / 2.0)
                .into()
        } else {
            contents
        }
    }))
    .spacing(settings.spacing * 1.5)
    .into()
}

/// A view strategy to display a Markdown [`Item`].
///
/// A [`Viewer`] is in charge of turning each [`Item`] into an [`Element`]. It
/// also provides the [`Theme`] and [`text::Highlighter`] used for rendering.
pub trait Viewer<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Self: Sized + 'a,
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer + 'a,
{
    /// The [`Theme`] used for styling the Markdown elements.
    fn theme(&self) -> &Theme;

    /// The [`text::Highlighter`] used for highligthing [`Code`] regions.
    fn highlighter(&self) -> &dyn text::Highlighter<Code, Theme>;

    /// Produces a message when a link is clicked with the given [`Uri`].
    fn on_link_click(url: Uri) -> Message;

    /// Displays an image.
    ///
    /// By default, it will show a container with the image title.
    fn image(
        &self,
        settings: Settings,
        url: &'a Uri,
        title: &'a str,
        alt: &Text,
    ) -> Element<'a, Message, Theme, Renderer> {
        let _url = url;
        let _title = title;

        container(
            rich_text(alt.spans(settings, self.theme(), self.highlighter()))
                .on_link_click(Self::on_link_click),
        )
        .padding(settings.spacing.0)
        .class(Theme::code_block())
        .into()
    }

    /// Displays a heading.
    ///
    /// By default, it calls [`heading`].
    fn heading(
        &self,
        settings: Settings,
        level: &'a HeadingLevel,
        text: &'a Text,
    ) -> Element<'a, Message, Theme, Renderer> {
        heading(self, settings, level, text, Self::on_link_click)
    }

    /// Displays a paragraph.
    ///
    /// By default, it calls [`paragraph`].
    fn paragraph(&self, settings: Settings, text: &Text) -> Element<'a, Message, Theme, Renderer> {
        paragraph(self, settings, text, Self::on_link_click)
    }

    /// Displays a code block.
    ///
    /// By default, it calls [`code_block`].
    fn code_block(
        &self,
        settings: Settings,
        language: Option<&'a str>,
        code: &'a str,
        lines: &'a [Text],
    ) -> Element<'a, Message, Theme, Renderer> {
        let _language = language;
        let _code = code;

        code_block(self, settings, lines, Self::on_link_click)
    }

    /// Displays an unordered list.
    ///
    /// By default, it calls [`unordered_list`].
    fn unordered_list(
        &self,
        settings: Settings,
        bullets: &'a [Bullet],
    ) -> Element<'a, Message, Theme, Renderer> {
        unordered_list(self, settings, bullets)
    }

    /// Displays an ordered list.
    ///
    /// By default, it calls [`ordered_list`].
    fn ordered_list(
        &self,
        settings: Settings,
        start: u64,
        bullets: &'a [Bullet],
    ) -> Element<'a, Message, Theme, Renderer> {
        ordered_list(self, settings, start, bullets)
    }

    /// Displays a quote.
    ///
    /// By default, it calls [`quote`].
    fn quote(
        &self,
        settings: Settings,
        contents: &'a [Item],
    ) -> Element<'a, Message, Theme, Renderer> {
        quote(self, settings, contents)
    }

    /// Displays a rule.
    ///
    /// By default, it calls [`rule`](self::rule()).
    fn rule(&self) -> Element<'a, Message, Theme, Renderer> {
        rule()
    }

    /// Displays a table.
    ///
    /// By default, it calls [`table`].
    fn table(
        &self,
        settings: Settings,
        columns: &'a [Column],
        rows: &'a [Row],
    ) -> Element<'a, Message, Theme, Renderer> {
        table(self, settings, columns, rows)
    }
}

/// The default [`Viewer`].
pub struct DefaultViewer<'a, Theme> {
    theme: Theme,
    highlighter: Option<Box<dyn text::Highlighter<Code, Theme> + 'a>>,
}

impl<'a, Theme> DefaultViewer<'a, Theme> {
    /// Creates a new [`DefaultViewer`] with the given [`Theme`].
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            highlighter: None,
        }
    }

    /// Sets a custom [`text::Highlighter`] for the [`DefaultViewer`].
    pub fn highlighter(mut self, highlighter: impl text::Highlighter<Code, Theme> + 'a) -> Self {
        self.highlighter = Some(Box::new(highlighter));
        self
    }
}

impl<'a, Theme, Renderer> Viewer<'a, Uri, Theme, Renderer> for DefaultViewer<'a, Theme>
where
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer + 'a,
{
    fn theme(&self) -> &Theme {
        &self.theme
    }

    fn highlighter(&self) -> &dyn text::Highlighter<Code, Theme> {
        self.highlighter
            .as_deref()
            .unwrap_or_else(|| self.theme.highlighter())
    }

    fn on_link_click(url: Uri) -> Uri {
        url
    }
}

/// The theme catalog of Markdown items.
pub trait Catalog:
    container::Catalog
    + scrollable::Catalog
    + text::Catalog
    + crate::rule::Catalog
    + checkbox::Catalog
    + crate::table::Catalog
    + Clone
    + PartialEq
{
    /// The unique identifier of the [`Catalog`].
    ///
    /// This will be used to invalidate span styling when a theme changes.
    fn id(&self) -> &str;

    /// The [`Color`] of some link.
    fn link_color(&self) -> Color;

    /// The [`InlineCode`] style of some inline code.
    fn code(&self) -> InlineCode;

    /// The styling class of a code block.
    fn code_block<'a>() -> <Self as container::Catalog>::Class<'a>;

    /// The styling class of a quote.
    fn quote<'a>() -> <Self as container::Catalog>::Class<'a>;

    /// The default [`text::Highlighter`] to use to highlight code.
    fn highlighter(&self) -> &dyn text::Highlighter<Code, Self>;
}

/// The style of some inline code.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InlineCode {
    /// The [`Padding`] to apply around the code.
    pub padding: Padding,
    /// The [`Highlight`] of the code.
    pub highlight: Highlight,
    /// The [`Color`] of the code.
    pub color: Color,
}

impl Catalog for Theme {
    fn id(&self) -> &str {
        theme::Base::name(self)
    }

    fn link_color(&self) -> Color {
        self.seed().primary
    }

    fn code(&self) -> InlineCode {
        let palette = self.palette();

        InlineCode {
            padding: padding::horizontal(4).vertical(1),
            highlight: Highlight {
                background: palette.background.weaker.color.into(),
                border: border::rounded(4),
            },
            color: palette.background.weaker.text,
        }
    }

    fn code_block<'a>() -> <Self as container::Catalog>::Class<'a> {
        Box::new(|theme| container::dark(theme).border(border::rounded(5)))
    }

    fn quote<'a>() -> <Self as container::Catalog>::Class<'a> {
        Box::new(|theme| {
            let palette = theme.palette();

            container::Style {
                text_color: Some(palette.background.weakest.text),
                background: Some(palette.background.weakest.color.into()),
                border: border::rounded(5),
                ..container::Style::default()
            }
        })
    }

    fn highlighter(&self) -> &dyn text::Highlighter<Code, Self> {
        &Code::highlight
    }
}

#[cfg(feature = "highlighter")]
mod code {
    use super::Span;

    #[derive(Debug)]
    pub struct Parser {
        lines: Vec<(String, Vec<Span>)>,
        language: String,
        stream: iced_highlighter::Stream,
        current: usize,
    }

    impl Parser {
        pub fn new(language: &str) -> Self {
            Self {
                lines: Vec::new(),
                stream: iced_highlighter::Stream::new(&iced_highlighter::Settings {
                    token: language.to_owned(),
                }),
                language: language.to_owned(),
                current: 0,
            }
        }

        pub fn language(&self) -> &str {
            &self.language
        }

        pub fn prepare(&mut self) {
            self.current = 0;
        }

        pub fn parse_line(&mut self, text: &str) -> &[Span] {
            match self.lines.get(self.current) {
                Some(line) if line.0 == text => {}
                _ => {
                    if self.current + 1 < self.lines.len() {
                        log::debug!("Resetting highlighter...");
                        self.stream.reset();
                        self.lines.truncate(self.current);

                        for line in &self.lines {
                            log::debug!("Refeeding {n} lines", n = self.lines.len());

                            let _ = self.stream.parse_line(&line.0);
                        }
                    }

                    log::trace!("Parsing: {text}", text = text.trim_end());

                    if self.current + 1 < self.lines.len() {
                        self.stream.commit();
                    }

                    let mut spans = Vec::new();

                    for (range, code) in self.stream.parse_line(text) {
                        spans.push(Span::Code {
                            text: text[range].to_owned(),
                            code,
                        });
                    }

                    if self.current + 1 == self.lines.len() {
                        let _ = self.lines.pop();
                    }

                    self.lines.push((text.to_owned(), spans));
                }
            }

            self.current += 1;

            &self
                .lines
                .get(self.current - 1)
                .expect("Line must be parsed")
                .1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn groups<const N: usize>(items: &[Item]) -> [(Option<&Item>, &[Item]); N] {
        sections(items)
            .collect::<Vec<_>>()
            .try_into()
            .expect("Unexpected number of sections")
    }

    fn assert_same_items<'a>(
        left: impl IntoIterator<Item = &'a Item>,
        right: impl IntoIterator<Item = &'a Item>,
    ) {
        let (mut left, mut right) = (left.into_iter(), right.into_iter());

        loop {
            match (left.next(), right.next()) {
                (Some(left), Some(right)) => assert!(std::ptr::eq(left, right)),
                (None, None) => break,
                (left, right) => panic!("Length mismatch: {left:?} vs {right:?}"),
            }
        }
    }

    #[test]
    fn empty_input_has_no_sections() {
        let items: Vec<_> = parse("").collect();
        assert_eq!(sections(&items).count(), 0);
    }

    #[test]
    fn input_without_headings_is_a_single_section() {
        let items: Vec<_> = parse("hello\n\nworld").collect();
        let [(heading, body)] = groups(&items);

        assert!(heading.is_none());
        assert_eq!(body.len(), items.len());
    }

    #[test]
    fn prefix_before_first_heading() {
        let items: Vec<_> = parse("prefix\n# Heading\n\nbody").collect();
        let [(preamble_heading, preamble), (heading, body)] = groups(&items);

        assert!(preamble_heading.is_none());
        assert_eq!(preamble.len(), 1);
        assert!(std::ptr::eq(&preamble[0], &items[0]));

        assert!(std::ptr::eq(
            heading.expect("Expected a heading"),
            &items[1]
        ));
        assert_eq!(body.len(), 1);
        assert!(std::ptr::eq(&body[0], &items[2]));
    }

    #[test]
    fn consecutive_headings_yield_empty_bodies() {
        let items: Vec<_> = parse("# A\n\n# B\n\n# C\n\nbody").collect();
        let [
            (heading_a, body_a),
            (heading_b, body_b),
            (heading_c, body_c),
        ] = groups(&items);

        assert!(heading_a.is_some());
        assert!(body_a.is_empty());
        assert!(heading_b.is_some());
        assert!(body_b.is_empty());
        assert!(heading_c.is_some());
        assert_eq!(body_c.len(), 1);
    }

    #[test]
    fn trailing_heading_yields_an_empty_body() {
        let items: Vec<_> = parse("body\n# Heading").collect();
        let [(preamble_heading, preamble), (heading, body)] = groups(&items);

        assert!(preamble_heading.is_none());
        assert_eq!(preamble.len(), 1);
        assert!(heading.is_some());
        assert!(body.is_empty());
    }

    #[test]
    fn every_item_is_yielded_exactly_once() {
        let items: Vec<_> =
            parse("intro\n# H1\n\np1\n- item\n> quote\n## H2\n\n```\ncode\n```\np2\n# H3")
                .collect();
        let groups = sections(&items).collect::<Vec<_>>();

        // The headings are yielded as the first element of their groups,
        // in order
        assert_same_items(
            groups.iter().filter_map(|(heading, _)| *heading),
            items
                .iter()
                .filter(|item| matches!(item, Item::Heading(..))),
        );

        // The bodies partition the non-heading items, in order
        assert_same_items(
            groups.iter().flat_map(|(_, body)| body.iter()),
            items
                .iter()
                .filter(|item| !matches!(item, Item::Heading(..))),
        );
    }

    #[test]
    fn content_sections() {
        let content = Content::parse("prefix\n# Heading\n\nbody");
        let [(preamble_heading, _), (heading, _)] = groups(content.items());

        assert!(preamble_heading.is_none());
        assert!(std::ptr::eq(
            heading.expect("Expected a heading"),
            &content.items()[1]
        ));
    }
}
