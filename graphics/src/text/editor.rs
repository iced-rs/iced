//! Draw and edit text.
use crate::core::text::editor::{
    self, Action, Cursor, Direction, Edit, Motion,
};
use crate::core::text::highlighter::{self, Highlighter};
use crate::core::text::{LineHeight, Wrapping};
use crate::core::{Font, Pixels, Point, Rectangle, Size};
use crate::text;

use cosmic_text::Edit as _;

use std::borrow::Cow;
use std::fmt;
use std::sync::{self, Arc, RwLock};

/// A multi-line text editor.
#[derive(Debug, PartialEq)]
pub struct Editor(Option<Arc<Internal>>);

struct Internal {
    editor: cosmic_text::Editor<'static>,
    cursor: RwLock<Option<Cursor>>,
    font: Font,
    bounds: Size,
    topmost_line_changed: Option<usize>,
    version: text::Version,
}

impl Editor {
    /// Creates a new empty [`Editor`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the buffer of the [`Editor`].
    pub fn buffer(&self) -> &cosmic_text::Buffer {
        buffer_from_editor(&self.internal().editor)
    }

    /// Creates a [`Weak`] reference to the [`Editor`].
    ///
    /// This is useful to avoid cloning the [`Editor`] when
    /// referential guarantees are unnecessary. For instance,
    /// when creating a rendering tree.
    pub fn downgrade(&self) -> Weak {
        let editor = self.internal();

        Weak {
            raw: Arc::downgrade(editor),
            bounds: editor.bounds,
        }
    }

    fn internal(&self) -> &Arc<Internal> {
        self.0
            .as_ref()
            .expect("Editor should always be initialized")
    }
}

impl editor::Editor for Editor {
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
            &cosmic_text::Attrs::new(),
            cosmic_text::Shaping::Advanced,
        );

        Editor(Some(Arc::new(Internal {
            editor: cosmic_text::Editor::new(buffer),
            version: font_system.version(),
            ..Default::default()
        })))
    }

    fn is_empty(&self) -> bool {
        let buffer = self.buffer();

        buffer.lines.is_empty()
            || (buffer.lines.len() == 1 && buffer.lines[0].text().is_empty())
    }

    fn line(&self, index: usize) -> Option<editor::Line<'_>> {
        self.buffer().lines.get(index).map(|line| editor::Line {
            text: Cow::Borrowed(line.text()),
            ending: match line.ending() {
                cosmic_text::LineEnding::Lf => editor::LineEnding::Lf,
                cosmic_text::LineEnding::CrLf => editor::LineEnding::CrLf,
                cosmic_text::LineEnding::Cr => editor::LineEnding::Cr,
                cosmic_text::LineEnding::LfCr => editor::LineEnding::LfCr,
                cosmic_text::LineEnding::None => editor::LineEnding::None,
            },
        })
    }

    fn line_count(&self) -> usize {
        self.buffer().lines.len()
    }

    fn selection(&self) -> Option<String> {
        self.internal().editor.copy_selection()
    }

    fn cursor(&self) -> editor::Cursor {
        let internal = self.internal();

        if let Ok(Some(cursor)) = internal.cursor.read().as_deref() {
            return cursor.clone();
        }

        let cursor = internal.editor.cursor();
        let buffer = buffer_from_editor(&internal.editor);

        let cursor = match internal.editor.selection_bounds() {
            Some((start, end)) => {
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
                                    * line_height
                                    - buffer.scroll().vertical,
                                height: line_height,
                            })
                        } else {
                            None
                        }
                    })
                    .collect();

                Cursor::Selection(regions)
            }
            _ => {
                let line_height = buffer.metrics().line_height;

                let visual_lines_offset =
                    visual_lines_offset(cursor.line, buffer);

                let line = buffer
                    .lines
                    .get(cursor.line)
                    .expect("Cursor line should be present");

                let layout =
                    line.layout_opt().expect("Line layout should be cached");

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
                        * line_height
                        - buffer.scroll().vertical,
                ))
            }
        };

        *internal.cursor.write().expect("Write to cursor cache") =
            Some(cursor.clone());

        cursor
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

        // Clear cursor cache
        let _ = internal
            .cursor
            .write()
            .expect("Write to cursor cache")
            .take();

        match action {
            // Motion events
            Action::Move(motion) => {
                if let Some((start, end)) = editor.selection_bounds() {
                    editor.set_selection(cosmic_text::Selection::None);

                    match motion {
                        // These motions are performed as-is even when a selection
                        // is present
                        Motion::Home
                        | Motion::End
                        | Motion::DocumentStart
                        | Motion::DocumentEnd => {
                            editor.action(
                                font_system.raw(),
                                cosmic_text::Action::Motion(to_motion(motion)),
                            );
                        }
                        // Other motions simply move the cursor to one end of the selection
                        _ => editor.set_cursor(match motion.direction() {
                            Direction::Left => start,
                            Direction::Right => end,
                        }),
                    }
                } else {
                    editor.action(
                        font_system.raw(),
                        cosmic_text::Action::Motion(to_motion(motion)),
                    );
                }
            }

            // Selection events
            Action::Select(motion) => {
                let cursor = editor.cursor();

                if editor.selection_bounds().is_none() {
                    editor
                        .set_selection(cosmic_text::Selection::Normal(cursor));
                }

                editor.action(
                    font_system.raw(),
                    cosmic_text::Action::Motion(to_motion(motion)),
                );

                // Deselect if selection matches cursor position
                if let Some((start, end)) = editor.selection_bounds() {
                    if start.line == end.line && start.index == end.index {
                        editor.set_selection(cosmic_text::Selection::None);
                    }
                }
            }
            Action::SelectWord => {
                let cursor = editor.cursor();

                editor.set_selection(cosmic_text::Selection::Word(cursor));
            }
            Action::SelectLine => {
                let cursor = editor.cursor();

                editor.set_selection(cosmic_text::Selection::Line(cursor));
            }
            Action::SelectAll => {
                let buffer = buffer_from_editor(editor);

                if buffer.lines.len() > 1
                    || buffer
                        .lines
                        .first()
                        .is_some_and(|line| !line.text().is_empty())
                {
                    let cursor = editor.cursor();

                    editor.set_selection(cosmic_text::Selection::Normal(
                        cosmic_text::Cursor {
                            line: 0,
                            index: 0,
                            ..cursor
                        },
                    ));

                    editor.action(
                        font_system.raw(),
                        cosmic_text::Action::Motion(
                            cosmic_text::Motion::BufferEnd,
                        ),
                    );
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
                    Edit::Indent => {
                        editor.action(
                            font_system.raw(),
                            cosmic_text::Action::Indent,
                        );
                    }
                    Edit::Unindent => {
                        editor.action(
                            font_system.raw(),
                            cosmic_text::Action::Unindent,
                        );
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
                let selection_start = editor
                    .selection_bounds()
                    .map(|(start, _)| start)
                    .unwrap_or(cursor);

                internal.topmost_line_changed = Some(selection_start.line);
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
                if let Some((start, end)) = editor.selection_bounds() {
                    if start.line == end.line && start.index == end.index {
                        editor.set_selection(cosmic_text::Selection::None);
                    }
                }
            }
            Action::Scroll { lines } => {
                editor.action(
                    font_system.raw(),
                    cosmic_text::Action::Scroll { lines },
                );
            }
        }

        self.0 = Some(Arc::new(internal));
    }

    fn bounds(&self) -> Size {
        self.internal().bounds
    }

    fn min_bounds(&self) -> Size {
        let internal = self.internal();

        let (bounds, _has_rtl) =
            text::measure(buffer_from_editor(&internal.editor));

        bounds
    }

    fn update(
        &mut self,
        new_bounds: Size,
        new_font: Font,
        new_size: Pixels,
        new_line_height: LineHeight,
        new_wrapping: Wrapping,
        new_highlighter: &mut impl Highlighter,
    ) {
        let editor =
            self.0.take().expect("Editor should always be initialized");

        let mut internal = Arc::try_unwrap(editor)
            .expect("Editor cannot have multiple strong references");

        let mut font_system =
            text::font_system().write().expect("Write font system");

        let buffer = buffer_mut_from_editor(&mut internal.editor);

        if font_system.version() != internal.version {
            log::trace!("Updating `FontSystem` of `Editor`...");

            for line in buffer.lines.iter_mut() {
                line.reset();
            }

            internal.version = font_system.version();
            internal.topmost_line_changed = Some(0);
        }

        if new_font != internal.font {
            log::trace!("Updating font of `Editor`...");

            for line in buffer.lines.iter_mut() {
                let _ = line.set_attrs_list(cosmic_text::AttrsList::new(
                    &text::to_attributes(new_font),
                ));
            }

            internal.font = new_font;
            internal.topmost_line_changed = Some(0);
        }

        let metrics = buffer.metrics();
        let new_line_height = new_line_height.to_absolute(new_size);

        if new_size.0 != metrics.font_size
            || new_line_height.0 != metrics.line_height
        {
            log::trace!("Updating `Metrics` of `Editor`...");

            buffer.set_metrics(
                font_system.raw(),
                cosmic_text::Metrics::new(new_size.0, new_line_height.0),
            );
        }

        let new_wrap = text::to_wrap(new_wrapping);

        if new_wrap != buffer.wrap() {
            log::trace!("Updating `Wrap` strategy of `Editor`...");

            buffer.set_wrap(font_system.raw(), new_wrap);
        }

        if new_bounds != internal.bounds {
            log::trace!("Updating size of `Editor`...");

            buffer.set_size(
                font_system.raw(),
                Some(new_bounds.width),
                Some(new_bounds.height),
            );

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

        // Clear cursor cache
        let _ = internal
            .cursor
            .write()
            .expect("Write to cursor cache")
            .take();

        self.0 = Some(Arc::new(internal));
    }

    fn highlight<H: Highlighter>(
        &mut self,
        font: Self::Font,
        highlighter: &mut H,
        format_highlight: impl Fn(&H::Highlight) -> highlighter::Format<Self::Font>,
    ) {
        let internal = self.internal();
        let buffer = buffer_from_editor(&internal.editor);

        let scroll = buffer.scroll();
        let mut window = (internal.bounds.height / buffer.metrics().line_height)
            .ceil() as i32;

        let last_visible_line = buffer.lines[scroll.line..]
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
                    Some(scroll.line + i)
                }
            })
            .unwrap_or(buffer.lines.len().saturating_sub(1));

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

        for line in &mut buffer_mut_from_editor(&mut internal.editor).lines
            [current_line..=last_visible_line]
        {
            let mut list = cosmic_text::AttrsList::new(&attributes);

            for (range, highlight) in highlighter.highlight_line(line.text()) {
                let format = format_highlight(&highlight);

                if format.color.is_some() || format.font.is_some() {
                    list.add_span(
                        range,
                        &cosmic_text::Attrs {
                            color_opt: format.color.map(text::to_color),
                            ..if let Some(font) = format.font {
                                text::to_attributes(font)
                            } else {
                                attributes.clone()
                            }
                        },
                    );
                }
            }

            let _ = line.set_attrs_list(list);
        }

        internal.editor.shape_as_needed(font_system.raw(), false);

        self.0 = Some(Arc::new(internal));
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self(Some(Arc::new(Internal::default())))
    }
}

impl PartialEq for Internal {
    fn eq(&self, other: &Self) -> bool {
        self.font == other.font
            && self.bounds == other.bounds
            && buffer_from_editor(&self.editor).metrics()
                == buffer_from_editor(&other.editor).metrics()
    }
}

impl Default for Internal {
    fn default() -> Self {
        Self {
            editor: cosmic_text::Editor::new(cosmic_text::Buffer::new_empty(
                cosmic_text::Metrics {
                    font_size: 1.0,
                    line_height: 1.0,
                },
            )),
            cursor: RwLock::new(None),
            font: Font::default(),
            bounds: Size::ZERO,
            topmost_line_changed: None,
            version: text::Version::default(),
        }
    }
}

impl fmt::Debug for Internal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Internal")
            .field("font", &self.font)
            .field("bounds", &self.bounds)
            .finish()
    }
}

/// A weak reference to an [`Editor`].
#[derive(Debug, Clone)]
pub struct Weak {
    raw: sync::Weak<Internal>,
    /// The bounds of the [`Editor`].
    pub bounds: Size,
}

impl Weak {
    /// Tries to update the reference into an [`Editor`].
    pub fn upgrade(&self) -> Option<Editor> {
        self.raw.upgrade().map(Some).map(Editor)
    }
}

impl PartialEq for Weak {
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
    let layout = line.layout_opt().map(Vec::as_slice).unwrap_or_default();

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
    let scroll = buffer.scroll();

    let start = scroll.line.min(line);
    let end = scroll.line.max(line);

    let visual_lines_offset: usize = buffer.lines[start..]
        .iter()
        .take(end - start)
        .map(|line| line.layout_opt().map(Vec::len).unwrap_or_default())
        .sum();

    visual_lines_offset as i32 * if scroll.line < line { 1 } else { -1 }
}

fn to_motion(motion: Motion) -> cosmic_text::Motion {
    match motion {
        Motion::Left => cosmic_text::Motion::Left,
        Motion::Right => cosmic_text::Motion::Right,
        Motion::Up => cosmic_text::Motion::Up,
        Motion::Down => cosmic_text::Motion::Down,
        Motion::WordLeft => cosmic_text::Motion::LeftWord,
        Motion::WordRight => cosmic_text::Motion::RightWord,
        Motion::Home => cosmic_text::Motion::Home,
        Motion::End => cosmic_text::Motion::End,
        Motion::PageUp => cosmic_text::Motion::PageUp,
        Motion::PageDown => cosmic_text::Motion::PageDown,
        Motion::DocumentStart => cosmic_text::Motion::BufferStart,
        Motion::DocumentEnd => cosmic_text::Motion::BufferEnd,
    }
}

fn buffer_from_editor<'a, 'b>(
    editor: &'a impl cosmic_text::Edit<'b>,
) -> &'a cosmic_text::Buffer
where
    'b: 'a,
{
    match editor.buffer_ref() {
        cosmic_text::BufferRef::Owned(buffer) => buffer,
        cosmic_text::BufferRef::Borrowed(buffer) => buffer,
        cosmic_text::BufferRef::Arc(buffer) => buffer,
    }
}

fn buffer_mut_from_editor<'a, 'b>(
    editor: &'a mut impl cosmic_text::Edit<'b>,
) -> &'a mut cosmic_text::Buffer
where
    'b: 'a,
{
    match editor.buffer_ref_mut() {
        cosmic_text::BufferRef::Owned(buffer) => buffer,
        cosmic_text::BufferRef::Borrowed(buffer) => buffer,
        cosmic_text::BufferRef::Arc(_buffer) => unreachable!(),
    }
}
