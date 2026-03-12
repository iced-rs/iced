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
//!         markdown::view(&self.markdown, Theme::TokyoNight)
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

use crate::core::alignment;
use crate::core::border;
use crate::core::font::{self, Font};
use crate::core::padding;
use crate::core::text::Wrapping;
use crate::core::theme;
use crate::core::{self, Color, Element, Length, Padding, Pixels, Theme, color};
use crate::{checkbox, column, container, rich_text, row, rule, scrollable, span, text};

use std::borrow::BorrowMut;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::mem;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

pub use core::text::Highlight;
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
                        highlighter: None,
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

impl Row {
    /// Returns the cells in this row.
    pub fn cells(&self) -> &[Vec<Item>] {
        &self.cells
    }
}

/// A bunch of parsed Markdown text.
#[derive(Debug, Clone)]
pub struct Text {
    spans: Vec<Span>,
    last_style: Cell<Option<Style>>,
    last_styled_spans: RefCell<Arc<[text::Span<'static, Uri>]>>,
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
    pub fn spans(&self, style: Style) -> Arc<[text::Span<'static, Uri>]> {
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
        link: Option<Uri>,
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
    fn view(&self, style: &Style) -> text::Span<'static, Uri> {
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
                    span.font(style.inline_code_font)
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
                        ..style.font
                    })
                } else {
                    span.font(style.font)
                };

                if let Some(link) = link.as_ref() {
                    span.color(style.link_color).link(link.clone())
                } else {
                    span
                }
            }
            #[cfg(feature = "highlighter")]
            Span::Highlight { text, color, font } => {
                span(text.clone()).color_maybe(*color).font_maybe(*font)
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
    /// Returns the items contained in this bullet.
    pub fn items(&self) -> &[Item] {
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
///         markdown::view(&self.markdown, Theme::TokyoNight)
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
    highlighter: Option<Highlighter>,
}

#[cfg(feature = "highlighter")]
#[derive(Debug)]
struct Highlighter {
    lines: Vec<(String, Vec<Span>)>,
    language: String,
    parser: iced_highlighter::Stream,
    current: usize,
}

#[cfg(feature = "highlighter")]
impl Highlighter {
    pub fn new(language: &str) -> Self {
        Self {
            lines: Vec::new(),
            parser: iced_highlighter::Stream::new(&iced_highlighter::Settings {
                theme: iced_highlighter::Theme::Base16Ocean,
                token: language.to_owned(),
            }),
            language: language.to_owned(),
            current: 0,
        }
    }

    pub fn prepare(&mut self) {
        self.current = 0;
    }

    pub fn highlight_line(&mut self, text: &str) -> &[Span] {
        match self.lines.get(self.current) {
            Some(line) if line.0 == text => {}
            _ => {
                if self.current + 1 < self.lines.len() {
                    log::debug!("Resetting highlighter...");
                    self.parser.reset();
                    self.lines.truncate(self.current);

                    for line in &self.lines {
                        log::debug!("Refeeding {n} lines", n = self.lines.len());

                        let _ = self.parser.highlight_line(&line.0);
                    }
                }

                log::trace!("Parsing: {text}", text = text.trim_end());

                if self.current + 1 < self.lines.len() {
                    self.parser.commit();
                }

                let mut spans = Vec::new();

                for (range, highlight) in self.parser.highlight_line(text) {
                    spans.push(Span::Highlight {
                        text: text[range].to_owned(),
                        color: highlight.color(),
                        font: highlight.font(),
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
    let mut highlighter = None;

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
                    highlighter = Some({
                        let mut highlighter = state
                            .borrow_mut()
                            .highlighter
                            .take()
                            .filter(|highlighter| highlighter.language == language.as_ref())
                            .unwrap_or_else(|| {
                                Highlighter::new(language.split(',').next().unwrap_or_default())
                            });

                        highlighter.prepare();

                        highlighter
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
                    state.borrow_mut().highlighter = highlighter.take();
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
                if let Some(highlighter) = &mut highlighter {
                    for line in text.lines() {
                        code_lines.push(Text::new(highlighter.highlight_line(line).to_vec()));
                    }
                }

                #[cfg(not(feature = "highlighter"))]
                for line in text.lines() {
                    code_lines.push(Text::new(vec![Span::Standard {
                        text: line.to_owned(),
                        strong,
                        emphasis,
                        strikethrough,
                        link: link.clone(),
                        code: true, // Use monospace font for code blocks
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
                code: false,
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
                code: true,
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
                code: false,
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
                code: false,
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
    /// The spacing to be used between elements.
    pub spacing: Pixels,
    /// The styling of the Markdown.
    pub style: Style,
    /// The width of text elements (paragraphs, headings, code blocks).
    ///
    /// Defaults to [`Length::Fill`]. Set to [`Length::Shrink`] for
    /// content that should wrap to the natural text width
    pub width: Length,
    /// The [`Wrapping`] strategy for text.
    ///
    /// Defaults to [`Wrapping::WordOrGlyph`].
    pub wrapping: Wrapping,
}

impl Settings {
    /// Creates new [`Settings`] with default text size and the given [`Style`].
    pub fn with_style(style: impl Into<Style>) -> Self {
        Self::with_text_size(16, style)
    }

    /// Creates new [`Settings`] with the given base text size in [`Pixels`].
    ///
    /// Heading levels will be adjusted automatically. Specifically,
    /// the first level will be twice the base size, and then every level
    /// after that will be 25% smaller.
    pub fn with_text_size(text_size: impl Into<Pixels>, style: impl Into<Style>) -> Self {
        let text_size = text_size.into();

        Self {
            text_size,
            h1_size: text_size * 2.0,
            h2_size: text_size * 1.75,
            h3_size: text_size * 1.5,
            h4_size: text_size * 1.25,
            h5_size: text_size,
            h6_size: text_size,
            code_size: text_size,
            spacing: text_size * 0.875,
            style: style.into(),
            width: Length::Fill,
            wrapping: Wrapping::WordOrGlyph,
        }
    }
}

impl From<&Theme> for Settings {
    fn from(theme: &Theme) -> Self {
        Self::with_style(Style::from(theme))
    }
}

impl From<Theme> for Settings {
    fn from(theme: Theme) -> Self {
        Self::with_style(Style::from(theme))
    }
}

/// The text styling of some Markdown rendering in [`view`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The [`Font`] to be applied to basic text.
    pub font: Font,
    /// The [`Highlight`] to be applied to the background of inline code.
    pub inline_code_highlight: Highlight,
    /// The [`Padding`] to be applied to the background of inline code.
    pub inline_code_padding: Padding,
    /// The [`Color`] to be applied to inline code.
    pub inline_code_color: Color,
    /// The [`Font`] to be applied to inline code.
    pub inline_code_font: Font,
    /// The [`Font`] to be applied to code blocks.
    pub code_block_font: Font,
    /// The [`Color`] to be applied to links.
    pub link_color: Color,
    /// The [`Color`] to be applied to text selection.
    pub selection_color: Color,
}

impl Style {
    /// Creates a new [`Style`] from the given [`theme::Palette`].
    pub fn from_palette(palette: theme::Palette) -> Self {
        Self {
            font: Font::default(),
            inline_code_padding: padding::left(1).right(1),
            inline_code_highlight: Highlight {
                background: color!(0x111111).into(),
                border: border::rounded(4),
            },
            inline_code_color: Color::WHITE,
            inline_code_font: Font::MONOSPACE,
            code_block_font: Font::MONOSPACE,
            link_color: palette.primary,
            selection_color: Color::from_rgba(
                palette.primary.r,
                palette.primary.g,
                palette.primary.b,
                0.3,
            ),
        }
    }
}

impl From<theme::Palette> for Style {
    fn from(palette: theme::Palette) -> Self {
        Self::from_palette(palette)
    }
}

impl From<&Theme> for Style {
    fn from(theme: &Theme) -> Self {
        Self::from_palette(theme.palette())
    }
}

impl From<Theme> for Style {
    fn from(theme: Theme) -> Self {
        Self::from_palette(theme.palette())
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
///         markdown::view(&self.markdown, Theme::TokyoNight)
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
    items: impl IntoIterator<Item = &'a Item>,
    settings: impl Into<Settings>,
) -> Element<'a, Uri, Theme, Renderer>
where
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer<Font = Font> + 'a,
{
    view_with(items, settings, &DefaultViewer)
}

/// Runs [`view`] but with a custom [`Viewer`] to turn an [`Item`] into
/// an [`Element`].
///
/// This is useful if you want to customize the look of certain Markdown
/// elements without selection. For selection support, use [`view_selectable_with`].
pub fn view_with<'a, Message, Theme, Renderer>(
    items: impl IntoIterator<Item = &'a Item>,
    settings: impl Into<Settings>,
    viewer: &impl Viewer<'a, Message, Theme, Renderer>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer<Font = Font> + 'a,
{
    let settings = settings.into();

    let blocks = items
        .into_iter()
        .enumerate()
        .map(|(i, item_)| item(viewer, settings, item_, i));

    Element::new(column(blocks).spacing(settings.spacing))
}

/// Selection state for markdown text selection.
///
/// Manages anchor/focus positions and computes per-element selection ranges.
#[derive(Debug, Clone, Default)]
pub struct Selection {
    /// Anchor point (element_idx, char_offset) - where selection started
    anchor: Option<(usize, usize)>,
    /// Focus point (element_idx, char_offset) - where selection currently ends
    focus: Option<(usize, usize)>,
    /// Whether a drag is in progress
    is_selecting: bool,
    /// Per-element selection ranges
    ranges: Vec<Option<(usize, usize)>>,
}

impl Selection {
    /// Create a new selection state for the given number of elements.
    pub fn new(element_count: usize) -> Self {
        Self {
            anchor: None,
            focus: None,
            is_selecting: false,
            ranges: vec![None; element_count],
        }
    }

    /// Reset selection state for a new element count (e.g., when content changes).
    pub fn reset(&mut self, element_count: usize) {
        self.anchor = None;
        self.focus = None;
        self.is_selecting = false;
        self.ranges = vec![None; element_count];
    }

    /// Start a selection at the given element and offset.
    pub fn start(&mut self, element_idx: usize, char_offset: usize) {
        self.anchor = Some((element_idx, char_offset));
        self.focus = Some((element_idx, char_offset));
        self.is_selecting = true;
        self.update_ranges();
    }

    /// Update the focus point during drag.
    pub fn update(&mut self, element_idx: usize, char_offset: usize) {
        self.focus = Some((element_idx, char_offset));
        self.update_ranges();
    }

    /// End the selection drag.
    pub fn end(&mut self) {
        self.is_selecting = false;
    }

    /// Whether a drag is in progress.
    pub fn is_selecting(&self) -> bool {
        self.is_selecting
    }

    /// Get the selection range for a specific element.
    pub fn get(&self, element_idx: usize) -> Option<(usize, usize)> {
        self.ranges.get(element_idx).copied().flatten()
    }

    /// Recompute per-element ranges from anchor/focus.
    fn update_ranges(&mut self) {
        // Clear all
        for range in &mut self.ranges {
            *range = None;
        }

        let Some((anchor_elem, anchor_off)) = self.anchor else {
            return;
        };
        let Some((focus_elem, focus_off)) = self.focus else {
            return;
        };

        // Normalize to start/end
        let (start_elem, start_off, end_elem, end_off) =
            if anchor_elem < focus_elem || (anchor_elem == focus_elem && anchor_off <= focus_off) {
                (anchor_elem, anchor_off, focus_elem, focus_off)
            } else {
                (focus_elem, focus_off, anchor_elem, anchor_off)
            };

        // Apply selection to each element in range
        for elem_idx in start_elem..=end_elem {
            if elem_idx >= self.ranges.len() {
                continue;
            }

            let elem_start = if elem_idx == start_elem { start_off } else { 0 };
            let elem_end = if elem_idx == end_elem {
                end_off
            } else {
                usize::MAX // Will be clamped by rich_text
            };

            if elem_start != elem_end {
                self.ranges[elem_idx] = Some((elem_start, elem_end));
            }
        }
    }
}

/// Count the total number of selectable elements in markdown items.
///
/// Useful for pre-allocating selection state with [`Selection::new`].
pub fn count_selectable_elements(items: &[Item]) -> usize {
    items
        .iter()
        .map(|item| {
            match item {
                Item::Paragraph(_) | Item::Heading(_, _) => 1,
                Item::CodeBlock { lines, .. } => lines.len(),
                Item::List { bullets, .. } => count_selectable_in_bullets(bullets),
                Item::Quote(nested) => count_selectable_elements(nested),
                Item::Table { columns, rows } => count_selectable_in_table(columns, rows),
                _ => 0, // Rule, Image
            }
        })
        .sum()
}

/// Count selectable elements in a table (headers + all cells).
fn count_selectable_in_table(columns: &[Column], rows: &[Row]) -> usize {
    // Count header cells (one per column header item)
    let header_count: usize = columns
        .iter()
        .flat_map(|col| &col.header)
        .map(count_selectable_in_item)
        .sum();

    // Count row cells
    let row_count: usize = rows
        .iter()
        .flat_map(|row| row.cells())
        .flat_map(|cell| cell.iter())
        .map(count_selectable_in_item)
        .sum();

    header_count + row_count
}

/// Count selectable elements in a single item.
fn count_selectable_in_item(item: &Item) -> usize {
    match item {
        Item::Paragraph(_) | Item::Heading(_, _) => 1,
        Item::CodeBlock { lines, .. } => lines.len(),
        Item::List { bullets, .. } => count_selectable_in_bullets(bullets),
        Item::Quote(nested) => count_selectable_elements(nested),
        Item::Table { columns, rows } => count_selectable_in_table(columns, rows),
        _ => 0,
    }
}

/// Count selectable elements in list bullets (recursively).
fn count_selectable_in_bullets(bullets: &[Bullet]) -> usize {
    bullets
        .iter()
        .map(|bullet| {
            bullet
                .items()
                .iter()
                .map(|item| match item {
                    Item::Paragraph(_) | Item::Heading(_, _) => 1,
                    Item::CodeBlock { lines, .. } => lines.len(),
                    Item::List { bullets, .. } => count_selectable_in_bullets(bullets),
                    Item::Quote(nested) => count_selectable_elements(nested),
                    _ => 0,
                })
                .sum::<usize>()
        })
        .sum()
}

/// View markdown with text selection support.
///
/// Each selectable element (paragraph, heading, code line, list item, table cell)
/// gets an index. Use [`count_selectable_elements`] to determine the total count
/// for pre-allocating selection state.
///
/// # Arguments
/// - `is_selecting`: Whether a drag selection is in progress
/// - `get_selection`: Returns the selection range for a given element index
/// - `on_start`: Called when selection starts (element_idx, char_offset)
/// - `on_drag`: Called during selection drag (element_idx, char_offset)
/// - `on_end`: Called when selection ends
/// - `images`: Optional pre-loaded images by URL
pub fn view_selectable<'a, Message, Theme, Renderer>(
    items: &'a [Item],
    settings: impl Into<Settings>,
    is_selecting: bool,
    get_selection: impl Fn(usize) -> Option<(usize, usize)> + Clone + 'a,
    on_start: impl Fn(usize, usize) -> Message + Clone + 'a,
    on_drag: impl Fn(usize, usize) -> Message + Clone + 'a,
    on_end: impl Fn() -> Message + Clone + 'a,
    images: Option<&'a std::collections::HashMap<String, crate::image::Handle>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer<Font = Font>
        + core::image::Renderer<Handle = crate::image::Handle>
        + 'a,
{
    let settings = settings.into();

    // Count total selectable elements (paragraphs, headings, code block lines, list items)
    let total_elements = count_selectable_elements(items);

    let mut element_idx = 0usize;

    let blocks = items.iter().map(|item_| {
        match item_ {
            Item::Paragraph(text) => {
                let idx = element_idx;
                element_idx += 1;

                let selection = get_selection.clone()(idx);
                let on_s = on_start.clone();
                let on_d = on_drag.clone();
                let on_e = on_end.clone();

                rich_text(text.spans(settings.style))
                    .font(settings.style.font)
                    .size(settings.text_size)
                    .wrapping(settings.wrapping)
                    .width(settings.width)
                    .selection(selection)
                    .selection_color(settings.style.selection_color)
                    .global_selecting(is_selecting)
                    .paragraph_info(idx, total_elements)
                    .on_selection_start(move |offset| on_s(idx, offset))
                    .on_selection_drag(move |offset| on_d(idx, offset))
                    .on_selection_end(on_e)
                    .into()
            }
            Item::Heading(level, text) => {
                let idx = element_idx;
                element_idx += 1;

                let selection = get_selection.clone()(idx);
                let on_s = on_start.clone();
                let on_d = on_drag.clone();
                let on_e = on_end.clone();

                let size = match level {
                    HeadingLevel::H1 => settings.h1_size,
                    HeadingLevel::H2 => settings.h2_size,
                    HeadingLevel::H3 => settings.h3_size,
                    HeadingLevel::H4 | HeadingLevel::H5 | HeadingLevel::H6 => settings.text_size,
                };

                rich_text(text.spans(settings.style))
                    .font(settings.style.font)
                    .size(size)
                    .wrapping(settings.wrapping)
                    .width(settings.width)
                    .selection(selection)
                    .selection_color(settings.style.selection_color)
                    .global_selecting(is_selecting)
                    .paragraph_info(idx, total_elements)
                    .on_selection_start(move |offset| on_s(idx, offset))
                    .on_selection_drag(move |offset| on_d(idx, offset))
                    .on_selection_end(on_e)
                    .into()
            }
            Item::CodeBlock { lines, .. } => {
                let start_idx = element_idx;
                element_idx += lines.len();

                code_block_selectable(
                    settings,
                    lines,
                    start_idx,
                    total_elements,
                    is_selecting,
                    get_selection.clone(),
                    on_start.clone(),
                    on_drag.clone(),
                    on_end.clone(),
                )
            }
            Item::List { start, bullets } => {
                let start_idx = element_idx;
                element_idx += count_selectable_in_bullets(bullets);

                list_selectable(
                    settings,
                    *start,
                    bullets,
                    start_idx,
                    total_elements,
                    is_selecting,
                    get_selection.clone(),
                    on_start.clone(),
                    on_drag.clone(),
                    on_end.clone(),
                )
            }
            Item::Rule => rule::horizontal(settings.spacing).into(),
            Item::Image { url, alt, .. } => {
                // Try to show cached image, fallback to alt text
                if let Some(handle) = images.and_then(|imgs| imgs.get(url)) {
                    crate::image(handle.clone()).width(Length::Shrink).into()
                } else {
                    container(
                        rich_text(alt.spans(settings.style))
                            .font(settings.style.font)
                            .size(settings.text_size)
                            .wrapping(settings.wrapping),
                    )
                    .padding(settings.spacing)
                    .class(Theme::code_block())
                    .into()
                }
            }
            Item::Quote(nested) => {
                let start_idx = element_idx;
                element_idx += count_selectable_elements(nested);

                quote_selectable(
                    settings,
                    nested,
                    start_idx,
                    total_elements,
                    is_selecting,
                    get_selection.clone(),
                    on_start.clone(),
                    on_drag.clone(),
                    on_end.clone(),
                )
            }
            Item::Table { columns, rows } => {
                let start_idx = element_idx;
                element_idx += count_selectable_in_table(columns, rows);

                table_selectable(
                    settings,
                    columns,
                    rows,
                    start_idx,
                    total_elements,
                    is_selecting,
                    get_selection.clone(),
                    on_start.clone(),
                    on_drag.clone(),
                    on_end.clone(),
                )
            }
        }
    });

    Element::new(column(blocks).spacing(settings.spacing))
}

/// View markdown with text selection and a custom [`Viewer`].
///
/// Like [`view_selectable`], but uses a [`Viewer`] for customizing
/// image display, link handling, and other rendering.
pub fn view_selectable_with<'a, Message, Theme, Renderer, V>(
    items: &'a [Item],
    settings: impl Into<Settings>,
    viewer: &V,
    is_selecting: bool,
    get_selection: impl Fn(usize) -> Option<(usize, usize)> + Clone + 'a,
    on_start: impl Fn(usize, usize) -> Message + Clone + 'a,
    on_drag: impl Fn(usize, usize) -> Message + Clone + 'a,
    on_end: impl Fn() -> Message + Clone + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer<Font = Font>
        + core::image::Renderer<Handle = crate::image::Handle>
        + 'a,
    V: Viewer<'a, Message, Theme, Renderer>,
{
    let settings = settings.into();
    let total_elements = count_selectable_elements(items);
    let mut element_idx = 0usize;

    let blocks = items.iter().map(|item_| {
        match item_ {
            Item::Paragraph(text) => {
                let idx = element_idx;
                element_idx += 1;

                let selection = get_selection.clone()(idx);
                let on_s = on_start.clone();
                let on_d = on_drag.clone();
                let on_e = on_end.clone();

                rich_text(text.spans(settings.style))
                    .on_link_click(V::on_link_click)
                    .font(settings.style.font)
                    .size(settings.text_size)
                    .wrapping(settings.wrapping)
                    .width(settings.width)
                    .selection(selection)
                    .selection_color(settings.style.selection_color)
                    .global_selecting(is_selecting)
                    .paragraph_info(idx, total_elements)
                    .on_selection_start(move |offset| on_s(idx, offset))
                    .on_selection_drag(move |offset| on_d(idx, offset))
                    .on_selection_end(on_e)
                    .into()
            }
            Item::Heading(level, text) => {
                let idx = element_idx;
                element_idx += 1;

                let selection = get_selection.clone()(idx);
                let on_s = on_start.clone();
                let on_d = on_drag.clone();
                let on_e = on_end.clone();

                let size = match level {
                    HeadingLevel::H1 => settings.h1_size,
                    HeadingLevel::H2 => settings.h2_size,
                    HeadingLevel::H3 => settings.h3_size,
                    HeadingLevel::H4 | HeadingLevel::H5 | HeadingLevel::H6 => settings.text_size,
                };

                rich_text(text.spans(settings.style))
                    .on_link_click(V::on_link_click)
                    .font(settings.style.font)
                    .size(size)
                    .wrapping(settings.wrapping)
                    .width(settings.width)
                    .selection(selection)
                    .selection_color(settings.style.selection_color)
                    .global_selecting(is_selecting)
                    .paragraph_info(idx, total_elements)
                    .on_selection_start(move |offset| on_s(idx, offset))
                    .on_selection_drag(move |offset| on_d(idx, offset))
                    .on_selection_end(on_e)
                    .into()
            }
            Item::CodeBlock { lines, .. } => {
                let start_idx = element_idx;
                element_idx += lines.len();

                code_block_selectable(
                    settings,
                    lines,
                    start_idx,
                    total_elements,
                    is_selecting,
                    get_selection.clone(),
                    on_start.clone(),
                    on_drag.clone(),
                    on_end.clone(),
                )
            }
            Item::List { start, bullets } => {
                let start_idx = element_idx;
                element_idx += count_selectable_in_bullets(bullets);

                list_selectable(
                    settings,
                    *start,
                    bullets,
                    start_idx,
                    total_elements,
                    is_selecting,
                    get_selection.clone(),
                    on_start.clone(),
                    on_drag.clone(),
                    on_end.clone(),
                )
            }
            Item::Quote(nested) => {
                let start_idx = element_idx;
                element_idx += count_selectable_elements(nested);

                quote_selectable(
                    settings,
                    nested,
                    start_idx,
                    total_elements,
                    is_selecting,
                    get_selection.clone(),
                    on_start.clone(),
                    on_drag.clone(),
                    on_end.clone(),
                )
            }
            Item::Rule => rule::horizontal(settings.spacing).into(),
            Item::Image { url, title, alt } => {
                // Delegate to viewer for custom image handling (lazy loading, etc.)
                viewer.image(settings, url, title, alt)
            }
            Item::Table { columns, rows } => {
                let start_idx = element_idx;
                element_idx += count_selectable_in_table(columns, rows);

                table_selectable(
                    settings,
                    columns,
                    rows,
                    start_idx,
                    total_elements,
                    is_selecting,
                    get_selection.clone(),
                    on_start.clone(),
                    on_drag.clone(),
                    on_end.clone(),
                )
            }
        }
    });

    Element::new(column(blocks).spacing(settings.spacing))
}

/// Render a code block with per-line selection support
fn code_block_selectable<'a, Message, Theme, Renderer>(
    settings: Settings,
    lines: &'a [Text],
    start_idx: usize,
    total_elements: usize,
    is_selecting: bool,
    get_selection: impl Fn(usize) -> Option<(usize, usize)> + Clone + 'a,
    on_start: impl Fn(usize, usize) -> Message + Clone + 'a,
    on_drag: impl Fn(usize, usize) -> Message + Clone + 'a,
    on_end: impl Fn() -> Message + Clone + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer<Font = Font> + 'a,
{
    let line_elements: Vec<Element<'a, Message, Theme, Renderer>> = lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let idx = start_idx + i;
            let selection = get_selection.clone()(idx);
            let on_s = on_start.clone();
            let on_d = on_drag.clone();
            let on_e = on_end.clone();

            rich_text(line.spans(settings.style))
                .font(Font::MONOSPACE)
                .size(settings.code_size)
                .wrapping(settings.wrapping)
                .width(settings.width)
                .selection(selection)
                .selection_color(settings.style.selection_color)
                .global_selecting(is_selecting)
                .paragraph_info(idx, total_elements)
                .on_selection_start(move |offset| on_s(idx, offset))
                .on_selection_drag(move |offset| on_d(idx, offset))
                .on_selection_end(on_e)
                .into()
        })
        .collect();

    container(column(line_elements).width(settings.width))
        .width(settings.width)
        .padding(settings.code_size)
        .class(Theme::code_block())
        .into()
}

/// Render a list with per-item selection support  
fn list_selectable<'a, Message, Theme, Renderer>(
    settings: Settings,
    start: Option<u64>,
    bullets: &'a [Bullet],
    start_idx: usize,
    total_elements: usize,
    is_selecting: bool,
    get_selection: impl Fn(usize) -> Option<(usize, usize)> + Clone + 'a,
    on_start: impl Fn(usize, usize) -> Message + Clone + 'a,
    on_drag: impl Fn(usize, usize) -> Message + Clone + 'a,
    on_end: impl Fn() -> Message + Clone + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer<Font = Font> + 'a,
{
    let digits = start
        .map(|s| ((s + bullets.len() as u64).max(1) as f32).log10().ceil())
        .unwrap_or(0.0);

    let mut current_idx = start_idx;

    let bullet_elements: Vec<Element<'a, Message, Theme, Renderer>> = bullets
        .iter()
        .enumerate()
        .map(|(i, bullet)| {
            let marker: Element<'_, Message, Theme, Renderer> = match (start, bullet) {
                (None, Bullet::Point { .. }) => crate::text("•").size(settings.text_size).into(),
                (None, Bullet::Task { done, .. }) => {
                    checkbox(*done).size(settings.text_size).into()
                }
                (Some(start), _) => {
                    let number = start + i as u64;
                    crate::text(format!("{number:>width$}.", width = digits as usize))
                        .size(settings.text_size)
                        .into()
                }
            };

            // Render bullet content
            let content_elements: Vec<Element<'a, Message, Theme, Renderer>> = bullet
                .items()
                .iter()
                .map(|item| {
                    let elem = match item {
                        Item::Paragraph(text) => {
                            let idx = current_idx;
                            current_idx += 1;
                            let selection = get_selection.clone()(idx);
                            let on_s = on_start.clone();
                            let on_d = on_drag.clone();
                            let on_e = on_end.clone();

                            rich_text(text.spans(settings.style))
                                .font(settings.style.font)
                                .size(settings.text_size)
                                .wrapping(settings.wrapping)
                                .selection(selection)
                                .selection_color(settings.style.selection_color)
                                .global_selecting(is_selecting)
                                .paragraph_info(idx, total_elements)
                                .on_selection_start(move |offset| on_s(idx, offset))
                                .on_selection_drag(move |offset| on_d(idx, offset))
                                .on_selection_end(on_e)
                                .into()
                        }
                        Item::List {
                            start: nested_start,
                            bullets: nested_bullets,
                        } => {
                            let nested_count = count_selectable_in_bullets(nested_bullets);
                            let nested_start_idx = current_idx;
                            current_idx += nested_count;
                            list_selectable(
                                settings,
                                *nested_start,
                                nested_bullets,
                                nested_start_idx,
                                total_elements,
                                is_selecting,
                                get_selection.clone(),
                                on_start.clone(),
                                on_drag.clone(),
                                on_end.clone(),
                            )
                        }
                        _ => crate::text("").into(),
                    };
                    elem
                })
                .collect();

            row![
                marker,
                column(content_elements).spacing(settings.spacing / 2)
            ]
            .spacing(settings.spacing / 2)
            .into()
        })
        .collect();

    column(bullet_elements).spacing(settings.spacing / 2).into()
}

/// Render a quote with selection support for nested items.
fn quote_selectable<'a, Message, Theme, Renderer>(
    settings: Settings,
    items: &'a [Item],
    start_idx: usize,
    total_elements: usize,
    is_selecting: bool,
    get_selection: impl Fn(usize) -> Option<(usize, usize)> + Clone + 'a,
    on_start: impl Fn(usize, usize) -> Message + Clone + 'a,
    on_drag: impl Fn(usize, usize) -> Message + Clone + 'a,
    on_end: impl Fn() -> Message + Clone + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer<Font = Font> + 'a,
{
    let mut current_idx = start_idx;

    let content_elements: Vec<Element<'a, Message, Theme, Renderer>> = items
        .iter()
        .map(|item_| match item_ {
            Item::Paragraph(text) => {
                let idx = current_idx;
                current_idx += 1;
                let selection = get_selection.clone()(idx);
                let on_s = on_start.clone();
                let on_d = on_drag.clone();
                let on_e = on_end.clone();

                rich_text(text.spans(settings.style))
                    .font(settings.style.font)
                    .size(settings.text_size)
                    .wrapping(settings.wrapping)
                    .selection(selection)
                    .selection_color(settings.style.selection_color)
                    .global_selecting(is_selecting)
                    .paragraph_info(idx, total_elements)
                    .on_selection_start(move |offset| on_s(idx, offset))
                    .on_selection_drag(move |offset| on_d(idx, offset))
                    .on_selection_end(on_e)
                    .into()
            }
            Item::Heading(level, text) => {
                let idx = current_idx;
                current_idx += 1;
                let selection = get_selection.clone()(idx);
                let on_s = on_start.clone();
                let on_d = on_drag.clone();
                let on_e = on_end.clone();

                let size = match level {
                    HeadingLevel::H1 => settings.h1_size,
                    HeadingLevel::H2 => settings.h2_size,
                    HeadingLevel::H3 => settings.h3_size,
                    HeadingLevel::H4 | HeadingLevel::H5 | HeadingLevel::H6 => settings.text_size,
                };

                rich_text(text.spans(settings.style))
                    .font(settings.style.font)
                    .size(size)
                    .wrapping(settings.wrapping)
                    .selection(selection)
                    .selection_color(settings.style.selection_color)
                    .global_selecting(is_selecting)
                    .paragraph_info(idx, total_elements)
                    .on_selection_start(move |offset| on_s(idx, offset))
                    .on_selection_drag(move |offset| on_d(idx, offset))
                    .on_selection_end(on_e)
                    .into()
            }
            Item::CodeBlock { lines, .. } => {
                let block_start = current_idx;
                current_idx += lines.len();
                code_block_selectable(
                    settings,
                    lines,
                    block_start,
                    total_elements,
                    is_selecting,
                    get_selection.clone(),
                    on_start.clone(),
                    on_drag.clone(),
                    on_end.clone(),
                )
            }
            Item::List { start, bullets } => {
                let list_start = current_idx;
                current_idx += count_selectable_in_bullets(bullets);
                list_selectable(
                    settings,
                    *start,
                    bullets,
                    list_start,
                    total_elements,
                    is_selecting,
                    get_selection.clone(),
                    on_start.clone(),
                    on_drag.clone(),
                    on_end.clone(),
                )
            }
            _ => crate::text("").into(),
        })
        .collect();

    row![
        rule::vertical(4),
        column(content_elements).spacing(settings.spacing.0),
    ]
    .height(Length::Shrink)
    .spacing(settings.spacing.0)
    .into()
}

/// Render a table with selection support using the table widget.
fn table_selectable<'a, Message, Theme, Renderer>(
    settings: Settings,
    columns: &'a [Column],
    rows: &'a [Row],
    start_idx: usize,
    total_elements: usize,
    is_selecting: bool,
    get_selection: impl Fn(usize) -> Option<(usize, usize)> + Clone + 'a,
    on_start: impl Fn(usize, usize) -> Message + Clone + 'a,
    on_drag: impl Fn(usize, usize) -> Message + Clone + 'a,
    on_end: impl Fn() -> Message + Clone + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer<Font = Font> + 'a,
{
    use crate::table;

    // Pre-compute header element count per column
    let header_counts: Vec<usize> = columns
        .iter()
        .map(|col| col.header.iter().map(|i| count_selectable_in_item(i)).sum())
        .collect();

    // Pre-compute cumulative header indices (starting index for each column's header)
    let mut header_start_indices = Vec::with_capacity(columns.len());
    let mut cumulative = start_idx;
    for &count in &header_counts {
        header_start_indices.push(cumulative);
        cumulative += count;
    }
    let header_end_idx = cumulative;

    // Pre-compute row cell counts: for each row, for each column, how many selectable items
    let row_cell_counts: Vec<Vec<usize>> = rows
        .iter()
        .map(|row| {
            row.cells()
                .iter()
                .map(|cell| cell.iter().map(|i| count_selectable_in_item(i)).sum())
                .collect()
        })
        .collect();

    // Pre-compute starting indices for each (row, column) cell
    let mut cell_start_indices: Vec<Vec<usize>> = Vec::with_capacity(rows.len());
    let mut current_idx = header_end_idx;
    for row_counts in &row_cell_counts {
        let mut row_starts = Vec::with_capacity(row_counts.len());
        for &count in row_counts {
            row_starts.push(current_idx);
            current_idx += count;
        }
        cell_start_indices.push(row_starts);
    }

    // Clone what we need for closures
    let cell_start_indices = std::sync::Arc::new(cell_start_indices);
    let rows_arc = std::sync::Arc::new(rows.iter().collect::<Vec<_>>());

    let table_widget = table(
        columns.iter().enumerate().map(|(col_idx, col)| {
            let header_start = header_start_indices[col_idx];
            let get_sel = get_selection.clone();
            let on_s = on_start.clone();
            let on_d = on_drag.clone();
            let on_e = on_end.clone();

            // Build header with selection
            let header_elements: Vec<Element<'a, Message, Theme, Renderer>> = col
                .header
                .iter()
                .enumerate()
                .map(|(item_idx, item)| {
                    let idx = header_start + item_idx;
                    render_item_with_index(
                        settings,
                        item,
                        idx,
                        total_elements,
                        is_selecting,
                        get_sel.clone(),
                        on_s.clone(),
                        on_d.clone(),
                        on_e.clone(),
                    )
                })
                .collect();

            let header = column(header_elements).spacing(settings.spacing.0 / 2.0);

            // Clone for the cell closure
            let cell_indices = cell_start_indices.clone();
            let rows_ref = rows_arc.clone();
            let get_sel2 = get_selection.clone();
            let on_s2 = on_start.clone();
            let on_d2 = on_drag.clone();
            let on_e2 = on_end.clone();

            table::column(
                header,
                move |row: &Row| -> Element<'a, Message, Theme, Renderer> {
                    // Find which row index this is
                    let row_idx = rows_ref
                        .iter()
                        .position(|r| std::ptr::eq(*r, row))
                        .unwrap_or(0);

                    if let Some(cells) = row.cells.get(col_idx) {
                        let cell_start = cell_indices
                            .get(row_idx)
                            .and_then(|r| r.get(col_idx))
                            .copied()
                            .unwrap_or(0);

                        let cell_elements: Vec<Element<'a, Message, Theme, Renderer>> = cells
                            .iter()
                            .enumerate()
                            .map(|(item_idx, item)| {
                                let idx = cell_start + item_idx;
                                render_item_with_index(
                                    settings,
                                    item,
                                    idx,
                                    total_elements,
                                    is_selecting,
                                    get_sel2.clone(),
                                    on_s2.clone(),
                                    on_d2.clone(),
                                    on_e2.clone(),
                                )
                            })
                            .collect();

                        column(cell_elements)
                            .spacing(settings.spacing.0 / 2.0)
                            .into()
                    } else {
                        crate::text("").into()
                    }
                },
            )
            .align_x(match col.alignment {
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

    scrollable(table_widget)
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::default(),
        ))
        .spacing(settings.spacing.0 / 2.0)
        .into()
}

/// Render an item with a specific pre-computed index.
fn render_item_with_index<'a, Message, Theme, Renderer>(
    settings: Settings,
    item: &'a Item,
    idx: usize,
    total_elements: usize,
    is_selecting: bool,
    get_selection: impl Fn(usize) -> Option<(usize, usize)> + Clone + 'a,
    on_start: impl Fn(usize, usize) -> Message + Clone + 'a,
    on_drag: impl Fn(usize, usize) -> Message + Clone + 'a,
    on_end: impl Fn() -> Message + Clone + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer<Font = Font> + 'a,
{
    match item {
        Item::Paragraph(text) => {
            let selection = get_selection(idx);
            rich_text(text.spans(settings.style))
                .font(settings.style.font)
                .size(settings.text_size)
                .wrapping(settings.wrapping)
                .selection(selection)
                .selection_color(settings.style.selection_color)
                .global_selecting(is_selecting)
                .paragraph_info(idx, total_elements)
                .on_selection_start(move |offset| on_start(idx, offset))
                .on_selection_drag(move |offset| on_drag(idx, offset))
                .on_selection_end(move || on_end())
                .into()
        }
        Item::Heading(level, text) => {
            let selection = get_selection(idx);
            let size = match level {
                HeadingLevel::H1 => settings.h1_size,
                HeadingLevel::H2 => settings.h2_size,
                HeadingLevel::H3 => settings.h3_size,
                HeadingLevel::H4 | HeadingLevel::H5 | HeadingLevel::H6 => settings.text_size,
            };
            rich_text(text.spans(settings.style))
                .font(settings.style.font)
                .size(size)
                .wrapping(settings.wrapping)
                .selection(selection)
                .selection_color(settings.style.selection_color)
                .global_selecting(is_selecting)
                .paragraph_info(idx, total_elements)
                .on_selection_start(move |offset| on_start(idx, offset))
                .on_selection_drag(move |offset| on_drag(idx, offset))
                .on_selection_end(move || on_end())
                .into()
        }
        _ => crate::text("").into(),
    }
}

/// Displays an [`Item`] using the given [`Viewer`].
pub fn item<'a, Message, Theme, Renderer>(
    viewer: &impl Viewer<'a, Message, Theme, Renderer>,
    settings: Settings,
    item: &'a Item,
    index: usize,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer<Font = Font> + 'a,
{
    match item {
        Item::Image { url, title, alt } => viewer.image(settings, url, title, alt),
        Item::Heading(level, text) => viewer.heading(settings, level, text, index),
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
        Item::Rule => viewer.rule(settings),
        Item::Table { columns, rows } => viewer.table(settings, columns, rows),
    }
}

/// Displays a heading using the default look.
pub fn heading<'a, Message, Theme, Renderer>(
    settings: Settings,
    level: &'a HeadingLevel,
    text: &'a Text,
    index: usize,
    on_link_click: impl Fn(Uri) -> Message + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer<Font = Font> + 'a,
{
    let Settings {
        h1_size,
        h2_size,
        h3_size,
        h4_size,
        h5_size,
        h6_size,
        text_size,
        ..
    } = settings;

    container(
        rich_text(text.spans(settings.style))
            .on_link_click(on_link_click)
            .size(match level {
                pulldown_cmark::HeadingLevel::H1 => h1_size,
                pulldown_cmark::HeadingLevel::H2 => h2_size,
                pulldown_cmark::HeadingLevel::H3 => h3_size,
                pulldown_cmark::HeadingLevel::H4 => h4_size,
                pulldown_cmark::HeadingLevel::H5 => h5_size,
                pulldown_cmark::HeadingLevel::H6 => h6_size,
            })
            .wrapping(settings.wrapping),
    )
    .padding(padding::top(if index > 0 {
        text_size / 2.0
    } else {
        Pixels::ZERO
    }))
    .into()
}

/// Displays a paragraph using the default look.
pub fn paragraph<'a, Message, Theme, Renderer>(
    settings: Settings,
    text: &Text,
    on_link_click: impl Fn(Uri) -> Message + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer<Font = Font> + 'a,
{
    rich_text(text.spans(settings.style))
        .size(settings.text_size)
        .wrapping(settings.wrapping)
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
    Renderer: core::text::Renderer<Font = Font> + 'a,
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
            view_with(
                bullet.items(),
                Settings {
                    spacing: settings.spacing * 0.6,
                    ..settings
                },
                viewer,
            )
        ]
        .spacing(settings.spacing)
        .into()
    }))
    .spacing(settings.spacing * 0.75)
    .padding([0.0, settings.spacing.0])
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
    Renderer: core::text::Renderer<Font = Font> + 'a,
{
    let digits = ((start + bullets.len() as u64).max(1) as f32)
        .log10()
        .ceil();

    column(bullets.iter().enumerate().map(|(i, bullet)| {
        row![
            text!("{}.", i as u64 + start)
                .size(settings.text_size)
                .align_x(alignment::Horizontal::Right)
                .width(settings.text_size * ((digits / 2.0).ceil() + 1.0)),
            view_with(
                bullet.items(),
                Settings {
                    spacing: settings.spacing * 0.6,
                    ..settings
                },
                viewer,
            )
        ]
        .spacing(settings.spacing)
        .into()
    }))
    .spacing(settings.spacing * 0.75)
    .into()
}

/// Displays a code block using the default look.
pub fn code_block<'a, Message, Theme, Renderer>(
    settings: Settings,
    lines: &'a [Text],
    on_link_click: impl Fn(Uri) -> Message + Clone + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer<Font = Font> + 'a,
{
    container(
        scrollable(
            container(column(lines.iter().map(|line| {
                rich_text(line.spans(settings.style))
                    .on_link_click(on_link_click.clone())
                    .font(settings.style.code_block_font)
                    .size(settings.code_size)
                    .wrapping(settings.wrapping)
                    .into()
            })))
            .padding(settings.code_size),
        )
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::default()
                .width(settings.code_size / 2)
                .scroller_width(settings.code_size / 2),
        )),
    )
    .width(settings.width)
    .padding(settings.code_size / 4)
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
    Renderer: core::text::Renderer<Font = Font> + 'a,
{
    row![
        rule::vertical(4),
        column(
            contents
                .iter()
                .enumerate()
                .map(|(i, content)| item(viewer, settings, content, i)),
        )
        .spacing(settings.spacing.0),
    ]
    .height(Length::Shrink)
    .spacing(settings.spacing.0)
    .into()
}

/// Displays a rule using the default look.
pub fn rule<'a, Message, Theme, Renderer>() -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer<Font = Font> + 'a,
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
    Renderer: core::text::Renderer<Font = Font> + 'a,
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
    Renderer: core::text::Renderer<Font = Font> + 'a,
{
    column(
        items
            .iter()
            .enumerate()
            .map(|(i, content)| item(viewer, settings, content, i)),
    )
    .spacing(settings.spacing.0)
    .into()
}

/// A view strategy to display a Markdown [`Item`].
pub trait Viewer<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Self: Sized + 'a,
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer<Font = Font> + 'a,
{
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
            rich_text(alt.spans(settings.style))
                .on_link_click(Self::on_link_click)
                .wrapping(settings.wrapping),
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
        index: usize,
    ) -> Element<'a, Message, Theme, Renderer> {
        heading(settings, level, text, index, Self::on_link_click)
    }

    /// Displays a paragraph.
    ///
    /// By default, it calls [`paragraph`].
    fn paragraph(&self, settings: Settings, text: &Text) -> Element<'a, Message, Theme, Renderer> {
        paragraph(settings, text, Self::on_link_click)
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

        code_block(settings, lines, Self::on_link_click)
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
    fn rule(&self, _settings: Settings) -> Element<'a, Message, Theme, Renderer> {
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

#[derive(Debug, Clone, Copy)]
struct DefaultViewer;

impl<'a, Theme, Renderer> Viewer<'a, Uri, Theme, Renderer> for DefaultViewer
where
    Theme: Catalog + 'a,
    Renderer: core::text::Renderer<Font = Font> + 'a,
{
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
{
    /// The styling class of a Markdown code block.
    fn code_block<'a>() -> <Self as container::Catalog>::Class<'a>;
}

impl Catalog for Theme {
    fn code_block<'a>() -> <Self as container::Catalog>::Class<'a> {
        Box::new(container::dark)
    }
}
