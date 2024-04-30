//! Draw and edit text.
use crate::core::text::editor::{
    self, Action, Cursor, Direction, Edit, Motion,
};
use crate::core::text::highlighter::{self, Highlighter};
use crate::core::text::LineHeight;
use crate::core::{Font, Pixels, Point, Rectangle, Size};
use crate::text;

use cosmic_text::{Edit as _, Selection};

use std::fmt;
use std::sync::{self, Arc};

/// A multi-line text editor.
#[derive(Debug, PartialEq)]
pub struct Editor<'buffer>(Option<Arc<Internal<'buffer>>>);

struct Internal<'buffer> {
    editor: cosmic_text::Editor<'buffer>,
    font: Font,
    bounds: Size,
    topmost_line_changed: Option<usize>,
    version: text::Version,
}

impl Editor<'_> {
    /// Creates a new empty [`Editor`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the internal buffer of the [`Editor`]
    pub fn with_buffer<F: FnOnce(&cosmic_text::Buffer) -> T, T>(&self, f: F) -> T {
        self.internal().editor.with_buffer(f)
    }

    /// Creates a [`Weak`] reference to the [`Editor`].
    ///
    /// This is useful to avoid cloning the [`Editor`] when
    /// referential guarantees are unnecessary. For instance,
    /// when creating a rendering tree.
    pub fn downgrade(&self) -> Weak<'_> {
        let editor = self.internal();

        Weak {
            raw: Arc::downgrade(editor),
            bounds: editor.bounds,
        }
    }

    fn internal(&self) -> &Arc<Internal<'_>> {
        self.0
            .as_ref()
            .expect("Editor should always be initialized")
    }
}

impl editor::Editor for Editor<'_> {
    type Font = Font;

    fn with_text(text: &str) -> Self {
        let mut buffer = cosmic_text::Buffer::new_empty(cosmic_text::Metrics {
            font_size: 1.0,
            line_height: 1.0,
        });

        let mut font_system =
            text::font_system().write().expect("Write font system");

        buffer.set_text(
            font_system.raw(),
            text,
            cosmic_text::Attrs::new(),
            cosmic_text::Shaping::Advanced,
        );

        Editor(Some(Arc::new(Internal {
            editor: cosmic_text::Editor::new(buffer),
            version: font_system.version(),
            ..Default::default()
        })))
    }

    fn line(&self, index: usize) -> Option<&str> {
        self.with_buffer(|buffer| {
            buffer.lines
                .get(index)
                .map(cosmic_text::BufferLine::text)
        })
    }

    fn line_count(&self) -> usize {
        self.with_buffer(|buffer| {
            buffer.lines.len()
        })
    }

    fn selection(&self) -> Option<String> {
        self.internal().editor.copy_selection()
    }

    fn cursor(&self) -> editor::Cursor {
        let internal = self.internal();

        let cursor = internal.editor.cursor();

        internal.editor.with_buffer(|buffer| {
            match internal.editor.selection() {
                Selection::Line(selection) |
                Selection::Normal(selection) |
                Selection::Word(selection) => {
                    let (start, end) = if cursor < selection {
                        (cursor, selection)
                    } else {
                        (selection, cursor)
                    };

                    let line_height = buffer.metrics().line_height;
                    let selected_lines = end.line - start.line + 1;

                    let visual_lines_offset =
                        visual_lines_offset(start.line, buffer);

                    let regions = buffer
                        .lines
                        .iter()
                        .skip(start.line)
                        .take(selected_lines)
                        .enumerate()
                        .flat_map(|(i, line)| {
                            highlight_line(
                                line,
                                if i == 0 { start.index } else { 0 },
                                if i == selected_lines - 1 {
                                    end.index
                                } else {
                                    line.text().len()
                                },
                            )
                        })
                        .enumerate()
                        .filter_map(|(visual_line, (x, width))| {
                            if width > 0.0 {
                                Some(Rectangle {
                                    x,
                                    width,
                                    y: (visual_line as i32 + visual_lines_offset)
                                        as f32
                                        * line_height,
                                    height: line_height,
                                })
                            } else {
                                None
                            }
                        })
                        .collect();

                    Cursor::Selection(regions)
                }
                Selection::None => {
                    let line_height = buffer.metrics().line_height;

                    let visual_lines_offset =
                        visual_lines_offset(cursor.line, buffer);

                    let line = buffer
                        .lines
                        .get(cursor.line)
                        .expect("Cursor line should be present");

                    let layout = line
                        .layout_opt()
                        .as_ref()
                        .expect("Line layout should be cached");

                    let mut lines = layout.iter().enumerate();

                    let (visual_line, offset) = lines
                        .find_map(|(i, line)| {
                            let start = line
                                .glyphs
                                .first()
                                .map(|glyph| glyph.start)
                                .unwrap_or(0);
                            let end = line
                                .glyphs
                                .last()
                                .map(|glyph| glyph.end)
                                .unwrap_or(0);

                            let is_cursor_before_start = start > cursor.index;

                            let is_cursor_before_end = match cursor.affinity {
                                cosmic_text::Affinity::Before => {
                                    cursor.index <= end
                                }
                                cosmic_text::Affinity::After => cursor.index < end,
                            };

                            if is_cursor_before_start {
                                // Sometimes, the glyph we are looking for is right
                                // between lines. This can happen when a line wraps
                                // on a space.
                                // In that case, we can assume the cursor is at the
                                // end of the previous line.
                                // i is guaranteed to be > 0 because `start` is always
                                // 0 for the first line, so there is no way for the
                                // cursor to be before it.
                                Some((i - 1, layout[i - 1].w))
                            } else if is_cursor_before_end {
                                let offset = line
                                    .glyphs
                                    .iter()
                                    .take_while(|glyph| cursor.index > glyph.start)
                                    .map(|glyph| glyph.w)
                                    .sum();

                                Some((i, offset))
                            } else {
                                None
                            }
                        })
                        .unwrap_or((
                            layout.len().saturating_sub(1),
                            layout.last().map(|line| line.w).unwrap_or(0.0),
                        ));

                    Cursor::Caret(Point::new(
                        offset,
                        (visual_lines_offset + visual_line as i32) as f32
                            * line_height,
                    ))
                }
            }
        })
    }

    fn cursor_position(&self) -> (usize, usize) {
        let cursor = self.internal().editor.cursor();

        (cursor.line, cursor.index)
    }

    fn perform(&mut self, action: Action) {
        let mut font_system =
            text::font_system().write().expect("Write font system");

        let editor =
            self.0.take().expect("Editor should always be initialized");

        // TODO: Handle multiple strong references somehow
        let mut internal = Arc::try_unwrap(editor)
            .expect("Editor cannot have multiple strong references");

        let editor = &mut internal.editor;

        match action {
            // Motion events
            Action::Move(motion) => {
                if let Selection::Line(selection) |
                Selection::Normal(selection) |
                Selection::Word(selection) = editor.selection() {
                    let cursor = editor.cursor();

                    let (left, right) = if cursor < selection {
                        (cursor, selection)
                    } else {
                        (selection, cursor)
                    };

                    editor.set_selection(Selection::None);

                    match motion {
                        // These motions are performed as-is even when a selection
                        // is present
                        Motion::Home
                        | Motion::End
                        | Motion::DocumentStart
                        | Motion::DocumentEnd => {
                            editor.action(
                                font_system.raw(),
                                motion_to_action(motion),
                            );
                        }
                        // Other motions simply move the cursor to one end of the selection
                        _ => editor.set_cursor(match motion.direction() {
                            Direction::Left => left,
                            Direction::Right => right,
                        }),
                    }
                } else {
                    editor.action(font_system.raw(), motion_to_action(motion));
                }
            }

            // Selection events
            Action::Select(motion) => {
                let cursor = editor.cursor();

                if editor.selection() == Selection::None {
                    editor.set_selection(Selection::Normal(cursor));
                }

                editor.action(font_system.raw(), motion_to_action(motion));

                // Deselect if selection matches cursor position
                if let Selection::Line(selection) |
                    Selection::Normal(selection) |
                    Selection::Word(selection) = editor.selection() {
                    let cursor = editor.cursor();

                    if cursor.line == selection.line
                        && cursor.index == selection.index
                    {
                        editor.set_selection(Selection::None);
                    }
                }
            }
            Action::SelectWord => {
                use unicode_segmentation::UnicodeSegmentation;

                let cursor = editor.cursor();

                if let Some(line) = editor.with_buffer(|buffer| {
                    buffer.lines.get(cursor.line)
                }) {
                    let (start, end) =
                        UnicodeSegmentation::unicode_word_indices(line.text())
                            // Split words with dots
                            .flat_map(|(i, word)| {
                                word.split('.').scan(i, |current, word| {
                                    let start = *current;
                                    *current += word.len() + 1;

                                    Some((start, word))
                                })
                            })
                            // Turn words into ranges
                            .map(|(i, word)| (i, i + word.len()))
                            // Find the word at cursor
                            .find(|&(start, end)| {
                                start <= cursor.index && cursor.index < end
                            })
                            // Cursor is not in a word. Let's select its punctuation cluster.
                            .unwrap_or_else(|| {
                                let start = line.text()[..cursor.index]
                                    .char_indices()
                                    .rev()
                                    .take_while(|(_, c)| {
                                        c.is_ascii_punctuation()
                                    })
                                    .map(|(i, _)| i)
                                    .last()
                                    .unwrap_or(cursor.index);

                                let end = line.text()[cursor.index..]
                                    .char_indices()
                                    .skip_while(|(_, c)| {
                                        c.is_ascii_punctuation()
                                    })
                                    .map(|(i, _)| i + cursor.index)
                                    .next()
                                    .unwrap_or(cursor.index);

                                (start, end)
                            });
                    if start != end {
                        editor.set_cursor(cosmic_text::Cursor {
                            index: start,
                            ..cursor
                        });

                        editor.set_selection(Selection::Word(cosmic_text::Cursor {
                            index: end,
                            ..cursor
                        }));
                    }
                }
            }
            Action::SelectLine => {
                let cursor = editor.cursor();

                if let Some(line_length) = editor.with_buffer(|buffer| {
                    buffer
                        .lines
                        .get(cursor.line)
                        .map(|line| line.text().len())
                }) {
                    editor
                        .set_cursor(cosmic_text::Cursor { index: 0, ..cursor });

                    editor.set_selection(Selection::Line(cosmic_text::Cursor {
                        index: line_length,
                        ..cursor
                    }));
                }
            }

            // Editing events
            Action::Edit(edit) => {
                match edit {
                    Edit::Insert(c) => {
                        editor.action(
                            font_system.raw(),
                            cosmic_text::Action::Insert(c),
                        );
                    }
                    Edit::Paste(text) => {
                        editor.insert_string(&text, None);
                    }
                    Edit::Enter => {
                        editor.action(
                            font_system.raw(),
                            cosmic_text::Action::Enter,
                        );
                    }
                    Edit::Backspace => {
                        editor.action(
                            font_system.raw(),
                            cosmic_text::Action::Backspace,
                        );
                    }
                    Edit::Delete => {
                        editor.action(
                            font_system.raw(),
                            cosmic_text::Action::Delete,
                        );
                    }
                }

                let cursor = editor.cursor();
                let selection = match editor.selection() {
                    Selection::Line(selection) => selection,
                    Selection::Normal(selection) => selection,
                    Selection::Word(selection) => selection,
                    Selection::None => cursor
                };

                internal.topmost_line_changed =
                    Some(cursor.min(selection).line);
            }

            // Mouse events
            Action::Click(position) => {
                editor.action(
                    font_system.raw(),
                    cosmic_text::Action::Click {
                        x: position.x as i32,
                        y: position.y as i32,
                    },
                );
            }
            Action::Drag(position) => {
                editor.action(
                    font_system.raw(),
                    cosmic_text::Action::Drag {
                        x: position.x as i32,
                        y: position.y as i32,
                    },
                );

                // Deselect if selection matches cursor position
                if let Selection::Line(selection) |
                    Selection::Normal(selection) |
                    Selection::Word(selection) = editor.selection() {
                    let cursor = editor.cursor();

                    if cursor.line == selection.line
                        && cursor.index == selection.index
                    {
                        editor.set_selection(Selection::None);
                    }
                }
            }
            Action::Scroll { lines } => {
                let (_, height) = editor.with_buffer(|buffer| {
                     buffer.size()
                });

                if height < i32::MAX as f32 {
                    editor.action(
                        font_system.raw(),
                        cosmic_text::Action::Scroll { lines },
                    );
                }
            }
        }

        self.0 = Some(Arc::new(internal));
    }

    fn bounds(&self) -> Size {
        self.internal().bounds
    }

    fn min_bounds(&self) -> Size {
        let internal = self.internal();

        internal.editor.with_buffer(|buffer| {
            text::measure(buffer)
        })
    }

    fn update(
        &mut self,
        new_bounds: Size,
        new_font: Font,
        new_size: Pixels,
        new_line_height: LineHeight,
        new_highlighter: &mut impl Highlighter,
    ) {
        let editor =
            self.0.take().expect("Editor should always be initialized");

        let mut internal = Arc::try_unwrap(editor)
            .expect("Editor cannot have multiple strong references");

        let mut font_system =
            text::font_system().write().expect("Write font system");

        if font_system.version() != internal.version {
            log::trace!("Updating `FontSystem` of `Editor`...");

            internal.editor.with_buffer_mut(|buffer| {
                for line in buffer.lines.iter_mut() {
                    line.reset();
                }
            });

            internal.version = font_system.version();
            internal.topmost_line_changed = Some(0);
        }

        if new_font != internal.font {
            log::trace!("Updating font of `Editor`...");

            internal.editor.with_buffer_mut(|buffer| {
                for line in buffer.lines.iter_mut() {
                    let _ = line.set_attrs_list(cosmic_text::AttrsList::new(
                        text::to_attributes(new_font),
                    ));
                }
            });

            internal.font = new_font;
            internal.topmost_line_changed = Some(0);
        }

        let metrics = internal.editor.with_buffer(|buffer| {
            buffer.metrics()
        });
        let new_line_height = new_line_height.to_absolute(new_size);

        if new_size.0 != metrics.font_size
            || new_line_height.0 != metrics.line_height
        {
            log::trace!("Updating `Metrics` of `Editor`...");

            internal.editor.with_buffer_mut(|buffer| {
                buffer.set_metrics(
                    font_system.raw(),
                    cosmic_text::Metrics::new(new_size.0, new_line_height.0),
                );
            });
        }

        if new_bounds != internal.bounds {
            log::trace!("Updating size of `Editor`...");

            internal.editor.with_buffer_mut(|buffer| {
                buffer.set_size(
                    font_system.raw(),
                    new_bounds.width,
                    new_bounds.height,
                );
            });

            internal.bounds = new_bounds;
        }

        if let Some(topmost_line_changed) = internal.topmost_line_changed.take()
        {
            log::trace!(
                "Notifying highlighter of line change: {topmost_line_changed}"
            );

            new_highlighter.change_line(topmost_line_changed);
        }

        internal.editor.shape_as_needed(font_system.raw(), false);

        self.0 = Some(Arc::new(internal));
    }

    fn highlight<H: Highlighter>(
        &mut self,
        font: Self::Font,
        highlighter: &mut H,
        format_highlight: impl Fn(&H::Highlight) -> highlighter::Format<Self::Font>,
    ) {
        let internal = self.internal();

        let mut window = internal.editor.with_buffer(|buffer| {
            buffer.scroll().layout + buffer.visible_lines()
        });

        let last_visible_line = internal.editor.with_buffer(|buffer| {
            buffer
                .lines
                .iter()
                .enumerate()
                .find_map(|(i, line)| {
                    let visible_lines = line
                        .layout_opt()
                        .as_ref()
                        .expect("Line layout should be cached")
                        .len() as i32;

                    if window > visible_lines {
                        window -= visible_lines;
                        None
                    } else {
                        Some(i)
                    }
                })
                .unwrap_or(buffer.lines.len().saturating_sub(1))
        });

        let current_line = highlighter.current_line();

        if current_line > last_visible_line {
            return;
        }

        let editor =
            self.0.take().expect("Editor should always be initialized");

        let mut internal = Arc::try_unwrap(editor)
            .expect("Editor cannot have multiple strong references");

        let mut font_system =
            text::font_system().write().expect("Write font system");

        let attributes = text::to_attributes(font);

        internal.editor.with_buffer_mut(|buffer| {
            for line in &mut buffer.lines
                [current_line..=last_visible_line]
            {
                let mut list = cosmic_text::AttrsList::new(attributes);

                for (range, highlight) in highlighter.highlight_line(line.text()) {
                    let format = format_highlight(&highlight);

                    if format.color.is_some() || format.font.is_some() {
                        list.add_span(
                            range,
                            cosmic_text::Attrs {
                                color_opt: format.color.map(text::to_color),
                                ..if let Some(font) = format.font {
                                    text::to_attributes(font)
                                } else {
                                    attributes
                                }
                            },
                        );
                    }
                }

                let _ = line.set_attrs_list(list);
            }

        });

        internal.editor.shape_as_needed(font_system.raw(), false);

        self.0 = Some(Arc::new(internal));
    }
}

impl Default for Editor<'_> {
    fn default() -> Self {
        Self(Some(Arc::new(Internal::default())))
    }
}

impl PartialEq for Internal<'_> {
    fn eq(&self, other: &Self) -> bool {
        let self_metrics = self.editor.with_buffer(|buffer| {
            buffer.metrics()
        });

        let other_metrics = other.editor.with_buffer(|buffer| {
            buffer.metrics()
        });

        self.font == other.font
            && self.bounds == other.bounds
            && self_metrics == other_metrics
    }
}

impl Default for Internal<'_> {
    fn default() -> Self {
        Self {
            editor: cosmic_text::Editor::new(cosmic_text::Buffer::new_empty(
                cosmic_text::Metrics {
                    font_size: 1.0,
                    line_height: 1.0,
                },
            )),
            font: Font::default(),
            bounds: Size::ZERO,
            topmost_line_changed: None,
            version: text::Version::default(),
        }
    }
}

impl fmt::Debug for Internal<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Internal")
            .field("font", &self.font)
            .field("bounds", &self.bounds)
            .finish()
    }
}

/// A weak reference to an [`Editor`].
#[derive(Debug, Clone)]
pub struct Weak<'buffer> {
    raw: sync::Weak<Internal<'buffer>>,
    /// The bounds of the [`Editor`].
    pub bounds: Size,
}

impl Weak<'_> {
    /// Tries to update the reference into an [`Editor`].
    pub fn upgrade(&self) -> Option<Editor<'_>> {
        self.raw.upgrade().map(Some).map(Editor)
    }
}

impl PartialEq for Weak<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self.raw.upgrade(), other.raw.upgrade()) {
            (Some(p1), Some(p2)) => p1 == p2,
            _ => false,
        }
    }
}

fn highlight_line(
    line: &cosmic_text::BufferLine,
    from: usize,
    to: usize,
) -> impl Iterator<Item = (f32, f32)> + '_ {
    let layout = line
        .layout_opt()
        .as_ref()
        .expect("Line layout should be cached");

    layout.iter().map(move |visual_line| {
        let start = visual_line
            .glyphs
            .first()
            .map(|glyph| glyph.start)
            .unwrap_or(0);
        let end = visual_line
            .glyphs
            .last()
            .map(|glyph| glyph.end)
            .unwrap_or(0);

        let range = start.max(from)..end.min(to);

        if range.is_empty() {
            (0.0, 0.0)
        } else if range.start == start && range.end == end {
            (0.0, visual_line.w)
        } else {
            let first_glyph = visual_line
                .glyphs
                .iter()
                .position(|glyph| range.start <= glyph.start)
                .unwrap_or(0);

            let mut glyphs = visual_line.glyphs.iter();

            let x =
                glyphs.by_ref().take(first_glyph).map(|glyph| glyph.w).sum();

            let width: f32 = glyphs
                .take_while(|glyph| range.end > glyph.start)
                .map(|glyph| glyph.w)
                .sum();

            (x, width)
        }
    })
}

fn visual_lines_offset(line: usize, buffer: &cosmic_text::Buffer) -> i32 {
    let visual_lines_before_start: usize = buffer
        .lines
        .iter()
        .take(line)
        .map(|line| {
            line.layout_opt()
                .as_ref()
                .expect("Line layout should be cached")
                .len()
        })
        .sum();

    visual_lines_before_start as i32 - buffer.scroll().layout
}

fn motion_to_action(motion: Motion) -> cosmic_text::Action {
    match motion {
        Motion::Left => cosmic_text::Action::Motion(cosmic_text::Motion::Left),
        Motion::Right => cosmic_text::Action::Motion(cosmic_text::Motion::Right),
        Motion::Up => cosmic_text::Action::Motion(cosmic_text::Motion::Up),
        Motion::Down => cosmic_text::Action::Motion(cosmic_text::Motion::Down),
        Motion::WordLeft => cosmic_text::Action::Motion(cosmic_text::Motion::LeftWord),
        Motion::WordRight => cosmic_text::Action::Motion(cosmic_text::Motion::RightWord),
        Motion::Home => cosmic_text::Action::Motion(cosmic_text::Motion::Home),
        Motion::End => cosmic_text::Action::Motion(cosmic_text::Motion::End),
        Motion::PageUp => cosmic_text::Action::Motion(cosmic_text::Motion::PageUp),
        Motion::PageDown => cosmic_text::Action::Motion(cosmic_text::Motion::PageDown),
        Motion::DocumentStart => cosmic_text::Action::Motion(cosmic_text::Motion::BufferStart),
        Motion::DocumentEnd => cosmic_text::Action::Motion(cosmic_text::Motion::BufferEnd),
    }
}
