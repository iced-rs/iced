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
use std::collections::hash_map::Entry;
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
    /// The raw Markdown accumulated so far, shared between all the
    /// items and sections.
    raw: String,
    /// The parsed output.
    items: Vec<Item>,
    /// The start of the source that is re-parsed when the item is the
    /// last one: the start of the item, or the start of its last
    /// bullet, when it is a list.
    starts: Vec<usize>,
    /// The true start of the item's source; for a list, the start of
    /// the list, unlike `starts`, which is the start of its last
    /// bullet.
    base: Vec<usize>,
    /// The start of the source that will be re-parsed on the next
    /// push.
    window: usize,
    /// Whether a not-yet-settled metadata block was live on the last
    /// push; when it was, the re-parse starts at the start of the
    /// block, so that the block is swallowed by the parser once it is
    /// closed (or re-parsed as the rule it turned out to be).
    pending_block: bool,
    /// The start of the not-yet-settled metadata block, if any; the
    /// re-parse starts there, so that the block (a tentative rule and
    /// its content, for instance) is re-parsed as a whole.
    pending_block_start: Option<usize>,
    incomplete: HashMap<usize, Section>,
    state: State,
}

#[derive(Debug)]
struct Section {
    /// The start of the source to re-parse when a reference becomes
    /// available or changes.
    start: usize,
    /// The end of the source to re-parse; `None` if the item is still
    /// the last one, so that the source can still grow.
    end: Option<usize>,
    broken_links: HashSet<String>,
    /// The references that were resolved when the item was last
    /// re-parsed, along with their destination at that time.
    references: HashMap<String, String>,
    /// The index of the item in the re-parse of the source.
    ///
    /// A source region can produce more than one item (an image and
    /// the paragraph it belongs to, for instance); this is the index
    /// of the item that this section refers to.
    item: usize,
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
    ///
    /// Only the last item is re-parsed on every call; and, when the last
    /// item is a list, only its last bullet is re-parsed, so that pushing
    /// new items to a long list stays cheap.
    ///
    /// The result converges to the one obtained by parsing the whole
    /// stream at once, as the stream grows.
    pub fn push_str(&mut self, markdown: &str) {
        if markdown.is_empty() {
            return;
        }

        self.raw.push_str(markdown);

        // The text to re-parse: from the start of the source of the
        // last item (or its last bullet, when it is a list) to the
        // end. Unless a not-yet-settled metadata block is live (or
        // was live on the last push), in which case the re-parse
        // starts at the start of the block: it is swallowed by the
        // parser once it is closed, and its first line (a tentative
        // rule) must be re-parsed with it.
        let block_live = self
            .pending_block_start
            .map(|start| Self::metadata_block_live(&self.raw[start..]))
            .unwrap_or(false);
        // A not-yet-settled metadata block that was live on the last
        // push is now settled: the references registered while it was
        // open are swallowed by it, so they must be dropped.
        let block_closed = self.pending_block && !block_live;
        let mut input_start = if self.pending_block || block_live {
            self.pending_block_start.unwrap_or(self.window)
        } else {
            self.window
        };

        // When the last item is a list and the previous item is a list
        // or a quote, the last list may be a lazy continuation of the
        // previous item's last bullet once more of it is streamed: an
        // empty `2. two\n-` bullet list, for instance, becomes the
        // `--` continuation of the numbered item as the second dash
        // arrives, and a `-` outside a quoted list likewise becomes a
        // continuation of the quoted bullet. Re-parse from the
        // previous item's start so that this collapse is re-evaluated
        // (and the re-parse yields the merged item, not a trailing
        // paragraph or list).
        if !self.pending_block
            && !block_live
            && let [.., prev, last] = self.items.as_slice()
            && matches!(last, Item::List { .. })
            && matches!(prev, Item::List { .. } | Item::Quote(_))
        {
            input_start = input_start.min(self.base[self.base.len() - 2]);
        }
        let tail = &self.raw[input_start..];
        let trimmed = tail.trim_end();
        let mut input = if trimmed.ends_with('|') {
            trimmed.trim_end_matches('|')
        } else {
            tail
        };

        // Pop the last item and the items whose source falls within
        // the text that will be re-parsed (an image and the paragraph
        // it belongs to, for instance); they will be re-parsed as
        // well.
        let last = self.items.pop();
        let _ = self.starts.pop();
        // The true start of the source of the last item, if any; it
        // is the start of the merged list, when the last item is a
        // list.
        let old_last_base = self.base.pop();
        while self
            .starts
            .last()
            .is_some_and(|start| *start >= input_start)
        {
            let _ = self.items.pop();
            let _ = self.starts.pop();
            let _ = self.base.pop();
        }

        // Re-parse the last item and the new text
        let mut items: Vec<(Item, usize, HashSet<String>)> =
            parse_with(&mut self.state, input).collect();

        // We only re-parse the last bullet of a list, so merge the
        // re-parsed list into the old one, keeping the bullets that
        // were already parsed. This only applies when the re-parse
        // actually started after the start of the list (at its last
        // bullet); when it re-parsed the whole list, the re-parsed
        // list already contains all the bullets, and merging would
        // duplicate them.
        let last_is_list = matches!(last.as_ref(), Some(Item::List { .. }));
        let mut merged = false;
        if let Some(Item::List { start, bullets, .. }) = last
            && let Some((first_item, _, _)) = items.first_mut()
            && let Item::List { bullets: new, .. } = first_item
            && old_last_base.is_some_and(|base| input_start > base)
        {
            // The last bullet of the old list was re-parsed
            let mut bullets = bullets;
            let _ = bullets.pop();
            bullets.extend(mem::take(new));
            *first_item = Item::List { start, bullets };
            merged = true;
        } else if last_is_list {
            // The re-parse of the last bullet no longer produces a
            // list (the bullet grew into a rule or a heading, for
            // instance), so it cannot be merged into the old list:
            // re-parse from the start of the whole list, so its
            // source is re-parsed in full.
            //
            // The re-parse must not start after a not-yet-settled
            // metadata block opener (kept in `input_start`), or the
            // block would be dropped; use the earliest of the two.
            let base = old_last_base.expect("a list has a base").min(input_start);
            let tail = &self.raw[base..];
            let trimmed = tail.trim_end();
            input = if trimmed.ends_with('|') {
                trimmed.trim_end_matches('|')
            } else {
                tail
            };
            input_start = base;
            items = parse_with(&mut self.state, input).collect();
        } else if let Some((Item::List { .. }, 0, _)) = items.first()
            && let Some(Item::List { .. }) = self.items.last()
            && let Some(base) = self.base.last().copied()
        {
            // The re-parse produced a list that starts where the old
            // last item (a lone paragraph, for instance) used to be,
            // right after a previous list: the paragraph grew into a
            // list item that continues the previous list. Re-parse
            // from the start of the previous list, so that the list
            // is not split in two; the parser decides whether the
            // two regions are one list.
            let tail = &self.raw[base..];
            let trimmed = tail.trim_end();
            input = if trimmed.ends_with('|') {
                trimmed.trim_end_matches('|')
            } else {
                tail
            };
            let reparsed: Vec<(Item, usize, HashSet<String>)> =
                parse_with(&mut self.state, input).collect();
            if let Some((Item::List { .. }, _, _)) = reparsed.first() {
                input_start = base;
                items = reparsed;
                // The previous list is covered by the re-parse
                let _ = self.items.pop();
                let _ = self.starts.pop();
                let _ = self.base.pop();
            }
        }

        // The start of the most recent `---` or `+++` rule produced
        // by the re-parse, if any; it is a tentative metadata block
        // opener.
        let mut newest_rule_start: Option<usize> = None;

        if items.is_empty() {
            // The new text did not produce any item (it completed a
            // reference definition or a metadata block, for
            // instance), so the last item was replaced by it; the
            // next push re-parses from the start of the new last
            // item.
            self.window = self.starts.last().copied().unwrap_or(input_start);
        } else {
            // Remember the start of the source of each re-parsed
            // item.
            let starts: Vec<usize> = items
                .iter()
                .map(|(_, start, _)| input_start + *start)
                .collect();

            for (i, (item, _start, broken_links)) in items.into_iter().enumerate() {
                let start = starts[i];
                // The merged list is anchored at the start of the
                // whole list, unlike `start`, which is the start of
                // its last bullet.
                let base = if i == 0 && merged {
                    old_last_base.expect("a merged list has a base")
                } else {
                    start
                };

                if !broken_links.is_empty() {
                    // The index of the item once it is pushed
                    let index = self.items.len();

                    // The next item can cover this one (a paragraph
                    // and the image it contains, for instance); in
                    // that case, the source to re-parse spans both.
                    let covers = starts.get(i + 1).is_some_and(|next| *next <= start);

                    // The source to re-parse starts at the start of
                    // the covered group, if any, and ends where the
                    // item after the group starts; it grows with the
                    // source while the group is the last one.
                    let (section_start, end) = if covers {
                        (starts[i + 1], starts.get(i + 2).copied())
                    } else {
                        (base, starts.get(i + 1).copied())
                    };

                    // A bullet that was not re-parsed can have broken
                    // links of its own, so they need to be kept
                    match self.incomplete.entry(index) {
                        Entry::Occupied(mut entry) => {
                            let section = entry.get_mut();
                            section.broken_links.extend(broken_links);
                            // The geometry can change (the item was
                            // not covered when the section was
                            // created, and its paragraph is now
                            // re-parsed as well)
                            section.start = section_start;
                            section.end = end;
                        }
                        Entry::Vacant(entry) => {
                            // The re-parse of the section's source
                            // produces the items that fall within
                            // the section's range (an image and the
                            // paragraph it belongs to, for instance);
                            // remember the index of the one this
                            // section refers to.
                            let item = starts
                                .iter()
                                .take(i)
                                .copied()
                                .filter(|start| *start >= section_start)
                                .count();
                            let _ = entry.insert(Section {
                                start: section_start,
                                end,
                                broken_links,
                                references: HashMap::new(),
                                item,
                            });
                        }
                    }
                }

                if matches!(item, Item::Rule) && Self::metadata_delimiter(&self.raw, start) {
                    // A `---` or `+++` rule is a tentative metadata
                    // block opener; remember where it starts so that
                    // the block is re-parsed as a whole when it is
                    // closed.
                    newest_rule_start = Some(start);
                }

                self.items.push(item);
                self.starts.push(start);
                self.base.push(base);
            }

            self.window = input_start + self.state.window.unwrap_or(input.len());
        }

        // Remember the tentative metadata block opener, if any, so
        // that the block is re-parsed as a whole as it grows, and
        // swallowed by the parser once it is closed.
        self.pending_block_start = newest_rule_start;
        self.pending_block = newest_rule_start
            .map(|start| Self::metadata_block_live(&self.raw[start..]))
            .unwrap_or(false);

        // A metadata block that was open on the last push is now
        // settled: the references registered while it was open are
        // swallowed by it, so recompute the references from the whole
        // source, as the one-shot parse does.
        if block_closed {
            self.recompute_references();
        }

        // The sections whose item is not the last one anymore have
        // a fixed source range
        self.fix_section_ends();

        // The sections whose broken links became resolvable, or
        // whose references changed, are re-parsed
        self.resolve_sections();

        // The images are those present in the items; recompute them,
        // as an image parsed while a metadata block was still open
        // can be swallowed by it once the block is closed.
        self.state.images = self
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Image { url, .. } => Some(url.clone()),
                _ => None,
            })
            .collect();
    }

    /// Returns `true` if the source starts with a metadata block that
    /// is not settled yet: the first line is a complete `---` or
    /// `+++` delimiter, the second line is not a blank one
    /// (otherwise the first line is a rule), and the block has not
    /// been closed.
    ///
    /// While such a block is live, its first line is a tentative
    /// rule that the parser swallows once the block is closed, so
    /// the re-parse has to cover the block as a whole.
    fn metadata_block_live(source: &str) -> bool {
        // The first line, which must be complete
        let (first, rest) = match source.find('\n') {
            Some(end) => (&source[..end], &source[end + 1..]),
            None => return false,
        };

        // The delimiter line
        let first = first.trim_end();
        if first != "---" && first != "+++" {
            return false;
        }

        // The second line: a blank one makes the first line a rule,
        // not a metadata block; an incomplete one could still be
        // the start of a block
        let Some(second_end) = rest.find('\n') else {
            return true;
        };
        let second = &rest[..second_end];
        if second.trim().is_empty() {
            return false;
        }

        // The block is closed by a `---` or `...` line, or a `+++`
        // line, when it is delimited by `+++`
        let closed = rest.lines().any(|line| {
            let line = line.trim_end();
            if first == "+++" {
                line == "+++"
            } else {
                line == "---" || line == "..."
            }
        });

        !closed
    }

    /// Returns `true` if the line starting at `start` is a complete
    /// `---` or `+++` metadata block delimiter.
    fn metadata_delimiter(source: &str, start: usize) -> bool {
        let line = &source[start..];
        let end = line.find('\n').unwrap_or(line.len());
        let line = line[..end].trim_end();
        line == "---" || line == "+++"
    }

    /// Re-parses the whole source and replaces the reference
    /// definitions with those of the one-shot parse.
    ///
    /// A reference registered while a metadata block was still open
    /// is swallowed by it once the block is settled, so it must not
    /// resolve links any more; re-parsing the whole source drops it,
    /// like the one-shot parse does.
    fn recompute_references(&mut self) {
        let parser = pulldown_cmark::Parser::new_ext(&self.raw, options());
        let definitions = parser.reference_definitions();

        self.state.references.clear();
        self.state.references_staged.clear();

        absorb_references(
            &self.raw,
            definitions,
            &mut self.state.references,
            &mut self.state.references_staged,
        );
    }

    /// Ends the sections whose item is not the last one anymore:
    /// their source range is now fixed, and it ends where the next
    /// item starts.
    fn fix_section_ends(&mut self) {
        if self.incomplete.is_empty() {
            return;
        }

        for (index, section) in self.incomplete.iter_mut() {
            if section.end.is_none() && *index + 1 < self.items.len() {
                // The next item can cover the section's item (a
                // paragraph and the image it contains), so the end
                // is the first start that is strictly after the
                // section's start
                section.end = self.starts[*index + 1..]
                    .iter()
                    .copied()
                    .find(|end| *end > section.start);
            }
        }
    }

    /// Re-parses the sections whose broken links became resolvable,
    /// or whose references changed destination; the sections that
    /// are left with nothing to watch are dropped.
    fn resolve_sections(&mut self) {
        if self.incomplete.is_empty() {
            return;
        }

        self.incomplete.retain(|index, section| {
            if self.items.len() <= *index {
                // The section's item is gone
                return false;
            }

            // A link becomes resolvable...
            let mut newly_resolved = Vec::new();
            section.broken_links.retain(|link| {
                if self.state.references.contains_key(link) {
                    newly_resolved.push(link.clone());
                    false
                } else {
                    true
                }
            });

            // ...or the destination of a resolved reference changes,
            // or the reference is dropped (its definition swallowed
            // by a metadata block, for instance)
            let needs_reparse = !newly_resolved.is_empty()
                || section.references.iter().any(|(link, dest)| {
                    match self.state.references.get(link) {
                        Some(new_dest) => new_dest != dest,
                        None => true,
                    }
                });

            if needs_reparse {
                let mut state = State {
                    window: None,
                    references: self.state.references.clone(),
                    references_staged: HashSet::new(),
                    images: HashSet::new(),
                    #[cfg(feature = "highlighter")]
                    parser: None,
                };

                let end = section.end.unwrap_or(self.raw.len());
                let source = &self.raw[section.start..end];

                if let Some((item, _start, broken_links)) =
                    parse_with(&mut state, source).nth(section.item)
                {
                    self.items[*index] = item;

                    // Track the references that were resolved by the
                    // re-parse, so that a later change of their
                    // destination triggers a new re-parse
                    for link in newly_resolved {
                        if let Some(dest) = self.state.references.get(&link)
                            && !broken_links.contains(&link)
                        {
                            let _ = section.references.insert(link, dest.to_owned());
                        }
                    }

                    section.broken_links = broken_links;
                    section
                        .references
                        .retain(|link, _| !section.broken_links.contains(link));

                    for (link, dest) in &mut section.references {
                        if let Some(new_dest) = self.state.references.get(link) {
                            *dest = new_dest.clone();
                        }
                    }
                }

                self.state.images.extend(state.images);
            }

            // The section is kept while something is left to watch
            !section.broken_links.is_empty() || !section.references.is_empty()
        });
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

    /// Returns the raw Markdown.
    pub fn raw(&self) -> &str {
        &self.raw
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
    parse_with(State::default(), markdown).map(|(item, _start, _broken_links)| item)
}

#[derive(Debug, Default)]
struct State {
    /// The start of the source that will be re-parsed next, after the
    /// current parse.
    window: Option<usize>,
    /// The reference definitions, mapping a label to its destination.
    ///
    /// The first definition of a label wins, like in CommonMark.
    references: HashMap<String, String>,
    /// The labels whose destination in `references` comes from the
    /// definition of the last line, which is not terminated yet and
    /// can still grow; their destination is updated on each push,
    /// until the line is terminated.
    references_staged: HashSet<String>,
    images: HashSet<Uri>,
    #[cfg(feature = "highlighter")]
    parser: Option<code::Parser>,
}

/// The options used by the parser.
fn options() -> pulldown_cmark::Options {
    pulldown_cmark::Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
        | pulldown_cmark::Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS
        | pulldown_cmark::Options::ENABLE_TABLES
        | pulldown_cmark::Options::ENABLE_STRIKETHROUGH
        | pulldown_cmark::Options::ENABLE_TASKLISTS
}

/// Absorbs the reference definitions of a parse into `references`,
/// keeping the first definition of a label, like in CommonMark.
///
/// The definition of the last line, when that line is not terminated
/// yet, can still grow, so its destination is updated on each push,
/// until the line is terminated; the growing labels are tracked in
/// `growing_refs`.
///
/// `markdown` is the source of the parse, and `definitions` are its
/// reference definitions.
fn absorb_references(
    markdown: &str,
    definitions: &pulldown_cmark::RefDefs<'_>,
    references: &mut HashMap<String, String>,
    growing_refs: &mut HashSet<String>,
) {
    for reference in definitions.iter() {
        let name = reference.0.to_string();
        let dest = reference.1.dest.to_string();

        if markdown[reference.1.span.end..].contains('\n') {
            if !references.contains_key(&name) || growing_refs.remove(&name) {
                let _ = references.insert(name, dest);
            }
        } else if growing_refs.contains(&name) {
            // The map's value is the growing one: update it
            let _ = references.insert(name, dest);
        } else if !references.contains_key(&name) {
            // No terminated definition wins: the growing value is
            // provisional
            let _ = growing_refs.insert(name.clone());
            let _ = references.insert(name, dest);
        }
    }
}

fn parse_with<'a>(
    mut state: impl BorrowMut<State> + 'a,
    markdown: &'a str,
) -> impl Iterator<Item = (Item, usize, HashSet<String>)> + 'a {
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
        /// The start of the last item of the list, if any.
        last_item_start: Option<usize>,
    }

    // The broken links reported by the parser, along with their span
    // in the input.
    //
    // The broken links are reported before the items that contain them
    // are produced, so the links are attributed to an item by their
    // span.
    let broken_links = Rc::new(RefCell::new(Vec::new()));

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
    let mut paragraph_start = None;
    let mut stack = Vec::new();

    #[cfg(feature = "highlighter")]
    let mut code_parser = None;

    let parser = pulldown_cmark::Parser::new_with_broken_link_callback(markdown, options(), {
        let references = state.borrow().references.clone();
        let broken_links = broken_links.clone();

        Some(move |broken_link: pulldown_cmark::BrokenLink<'_>| {
            if let Some(reference) = references.get(broken_link.reference.as_ref()) {
                Some((
                    pulldown_cmark::CowStr::from(reference.to_owned()),
                    broken_link.reference.into_static(),
                ))
            } else {
                RefCell::borrow_mut(&broken_links)
                    .push((broken_link.span, broken_link.reference.into_string()));

                None
            }
        })
    });

    {
        let state = state.borrow_mut();
        absorb_references(
            markdown,
            parser.reference_definitions(),
            &mut state.references,
            &mut state.references_staged,
        );
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
            state.window = Some(source.start);

            // Attribute the broken links whose span falls within the
            // source of the item
            let mut links = HashSet::new();
            for (span, reference) in RefCell::borrow(&broken_links).iter() {
                if source.contains(&span.start) {
                    let _ = links.insert(reference.clone());
                }
            }

            Some((item, source.start, links))
        }
    };

    // A reference link or image resolves with the first definition
    // of its label in the whole document, like in the one-shot
    // parse. A later definition that falls within the input wins
    // within the input, so, when known, prefer the global
    // definition.
    let resolve_reference = |state: &mut State,
                             link_type: pulldown_cmark::LinkType,
                             id: &str,
                             dest_url: &pulldown_cmark::CowStr<'a>|
     -> String {
        match link_type {
            pulldown_cmark::LinkType::Reference
            | pulldown_cmark::LinkType::ReferenceUnknown
            | pulldown_cmark::LinkType::Collapsed
            | pulldown_cmark::LinkType::CollapsedUnknown
            | pulldown_cmark::LinkType::Shortcut
            | pulldown_cmark::LinkType::ShortcutUnknown => state
                .references
                .get(id)
                .cloned()
                .unwrap_or_else(|| dest_url.to_string()),
            _ => dest_url.to_string(),
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
            pulldown_cmark::Tag::Link {
                link_type,
                dest_url,
                id,
                ..
            } if !metadata => {
                link = Some(resolve_reference(
                    state.borrow_mut(),
                    link_type,
                    &id,
                    &dest_url,
                ));
                None
            }
            pulldown_cmark::Tag::Paragraph if !metadata => {
                paragraph_start = Some(source.start);
                None
            }
            pulldown_cmark::Tag::Image {
                link_type,
                dest_url,
                title,
                id,
            } if !metadata => {
                image = Some((
                    resolve_reference(state.borrow_mut(), link_type, &id, &dest_url),
                    title.into_string(),
                    spans.len(),
                ));
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
                    last_item_start: None,
                }));

                prev
            }
            pulldown_cmark::Tag::Item => {
                if let Some(Scope::List(list)) = stack.last_mut() {
                    list.last_item_start = Some(source.start);
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
                paragraph_start = None;

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

                let last_item_start = list.last_item_start;
                let produced = produce(
                    state.borrow_mut(),
                    &mut stack,
                    Item::List {
                        start: list.start,
                        bullets: list.bullets,
                    },
                    source,
                );

                // A list is re-parsed only from the start of its last
                // item, so that adding new items to a long list does not
                // require re-parsing the whole list.
                if produced.is_some()
                    && let Some(start) = last_item_start
                {
                    state.borrow_mut().window = Some(start);
                }

                produced
            }
            pulldown_cmark::TagEnd::BlockQuote(_kind) if !metadata => {
                let scope = stack.pop()?;

                let Scope::Quote(quote) = scope else {
                    return None;
                };

                produce(state.borrow_mut(), &mut stack, Item::Quote(quote), source)
            }
            pulldown_cmark::TagEnd::Image if !metadata => {
                let (url, title, start) = image.take()?;
                let alt = Text::new(spans.drain(start..).collect());

                let state = state.borrow_mut();
                let _ = state.images.insert(url.clone());

                let produced = produce(state, &mut stack, Item::Image { url, title, alt }, source);

                // A top-level image is re-parsed from the start of the
                // line that contains it, as the rest of the line can
                // change how the image is parsed.
                if let Some(start) = paragraph_start.filter(|_| produced.is_some()) {
                    state.borrow_mut().window = Some(start);
                }

                produced
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
    let padding = settings.code_block_size / 0.85 * 0.75;

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
                .width(padding / 2.0)
                .scroller_width(padding / 2.0),
        ))
        .spacing(padding),
    )
    .width(Length::Fill)
    .padding(padding)
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
