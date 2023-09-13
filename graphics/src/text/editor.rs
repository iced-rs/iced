use crate::core::text::editor::{self, Action, Cursor, Motion};
use crate::core::text::LineHeight;
use crate::core::{Font, Pixels, Point, Rectangle, Size, Vector};
use crate::text;

use cosmic_text::Edit;

use std::fmt;
use std::sync::{self, Arc};

#[derive(Debug, PartialEq)]
pub struct Editor(Option<Arc<Internal>>);

struct Internal {
    editor: cosmic_text::Editor,
    font: Font,
    bounds: Size,
    min_bounds: Size,
    version: text::Version,
}

impl Editor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn buffer(&self) -> &cosmic_text::Buffer {
        &self.internal().editor.buffer()
    }

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
            .expect("editor should always be initialized")
    }
}

impl editor::Editor for Editor {
    type Font = Font;

    fn with_text(text: &str) -> Self {
        let mut buffer = cosmic_text::Buffer::new_empty(cosmic_text::Metrics {
            font_size: 1.0,
            line_height: 1.0,
        });

        buffer.set_text(
            text::font_system()
                .write()
                .expect("Write font system")
                .raw(),
            text,
            cosmic_text::Attrs::new(),
            cosmic_text::Shaping::Advanced,
        );

        Editor(Some(Arc::new(Internal {
            editor: cosmic_text::Editor::new(buffer),
            ..Default::default()
        })))
    }

    fn cursor(&self) -> editor::Cursor {
        let internal = self.internal();

        let cursor = internal.editor.cursor();
        let buffer = internal.editor.buffer();

        match internal.editor.select_opt() {
            Some(selection) => {
                let line_height = buffer.metrics().line_height;
                let scroll_offset = buffer.scroll() as f32 * line_height;

                let (start, end) = if cursor < selection {
                    (cursor, selection)
                } else {
                    (selection, cursor)
                };

                let visual_lines_before_start: usize = buffer
                    .lines
                    .iter()
                    .take(start.line)
                    .map(|line| {
                        line.layout_opt()
                            .as_ref()
                            .expect("Line layout should be cached")
                            .len()
                    })
                    .sum();

                let selected_lines = end.line - start.line + 1;

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
                                y: visual_line as f32 * line_height,
                                height: line_height,
                            })
                        } else {
                            None
                        }
                    })
                    .map(|region| {
                        region
                            + Vector::new(
                                0.0,
                                visual_lines_before_start as f32 * line_height
                                    + scroll_offset,
                            )
                    })
                    .collect();

                Cursor::Selection(regions)
            }
            _ => {
                let lines_before_cursor: usize = buffer
                    .lines
                    .iter()
                    .take(cursor.line)
                    .map(|line| {
                        line.layout_opt()
                            .as_ref()
                            .expect("Line layout should be cached")
                            .len()
                    })
                    .sum();

                let line = buffer
                    .lines
                    .get(cursor.line)
                    .expect("Cursor line should be present");

                let layout = line
                    .layout_opt()
                    .as_ref()
                    .expect("Line layout should be cached");

                let mut lines = layout.iter().enumerate();

                let (subline, offset) = lines
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

                        let is_cursor_after_start = start <= cursor.index;

                        let is_cursor_before_end = match cursor.affinity {
                            cosmic_text::Affinity::Before => {
                                cursor.index <= end
                            }
                            cosmic_text::Affinity::After => cursor.index < end,
                        };

                        if is_cursor_after_start && is_cursor_before_end {
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

                let line_height = buffer.metrics().line_height;

                let scroll_offset = buffer.scroll() as f32 * line_height;

                Cursor::Caret(Point::new(
                    offset,
                    (lines_before_cursor + subline) as f32 * line_height
                        - scroll_offset,
                ))
            }
        }
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
                if let Some(_selection) = editor.select_opt() {
                    editor.set_select_opt(None);
                } else {
                    editor.action(
                        font_system.raw(),
                        match motion {
                            Motion::Left => cosmic_text::Action::Left,
                            Motion::Right => cosmic_text::Action::Right,
                            Motion::Up => cosmic_text::Action::Up,
                            Motion::Down => cosmic_text::Action::Down,
                            Motion::WordLeft => cosmic_text::Action::LeftWord,
                            Motion::WordRight => cosmic_text::Action::RightWord,
                            Motion::Home => cosmic_text::Action::Home,
                            Motion::End => cosmic_text::Action::End,
                            Motion::PageUp => cosmic_text::Action::PageUp,
                            Motion::PageDown => cosmic_text::Action::PageDown,
                            Motion::DocumentStart => {
                                cosmic_text::Action::BufferStart
                            }
                            Motion::DocumentEnd => {
                                cosmic_text::Action::BufferEnd
                            }
                        },
                    );
                }
            }

            // Selection events
            Action::Select(_motion) => todo!(),
            Action::SelectWord => todo!(),
            Action::SelectLine => todo!(),

            // Editing events
            Action::Insert(c) => {
                editor
                    .action(font_system.raw(), cosmic_text::Action::Insert(c));
            }
            Action::Enter => {
                editor.action(font_system.raw(), cosmic_text::Action::Enter);
            }
            Action::Backspace => {
                editor
                    .action(font_system.raw(), cosmic_text::Action::Backspace);
            }
            Action::Delete => {
                editor.action(font_system.raw(), cosmic_text::Action::Delete);
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
                if let Some(selection) = editor.select_opt() {
                    let cursor = editor.cursor();

                    if cursor.line == selection.line
                        && cursor.index == selection.index
                    {
                        editor.set_select_opt(None);
                    }
                }
            }
        }

        editor.shape_as_needed(font_system.raw());

        self.0 = Some(Arc::new(internal));
    }

    fn bounds(&self) -> Size {
        self.internal().bounds
    }

    fn min_bounds(&self) -> Size {
        self.internal().min_bounds
    }

    fn update(
        &mut self,
        new_bounds: Size,
        new_font: Font,
        new_size: Pixels,
        new_line_height: LineHeight,
    ) {
        let editor =
            self.0.take().expect("editor should always be initialized");

        let mut internal = Arc::try_unwrap(editor)
            .expect("Editor cannot have multiple strong references");

        let mut font_system =
            text::font_system().write().expect("Write font system");

        let mut changed = false;

        if new_font != internal.font {
            for line in internal.editor.buffer_mut().lines.iter_mut() {
                let _ = line.set_attrs_list(cosmic_text::AttrsList::new(
                    text::to_attributes(new_font),
                ));
            }

            changed = true;
        }

        let metrics = internal.editor.buffer().metrics();
        let new_line_height = new_line_height.to_absolute(new_size);

        if new_size.0 != metrics.font_size
            || new_line_height.0 != metrics.line_height
        {
            internal.editor.buffer_mut().set_metrics(
                font_system.raw(),
                cosmic_text::Metrics::new(new_size.0, new_line_height.0),
            );

            changed = true;
        }

        if new_bounds != internal.bounds {
            internal.editor.buffer_mut().set_size(
                font_system.raw(),
                new_bounds.width,
                new_bounds.height,
            );

            internal.bounds = new_bounds;
            changed = true;
        }

        if changed {
            internal.min_bounds = text::measure(&internal.editor.buffer());
        }

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
            && self.min_bounds == other.min_bounds
            && self.editor.buffer().metrics() == other.editor.buffer().metrics()
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
            font: Font::default(),
            bounds: Size::ZERO,
            min_bounds: Size::ZERO,
            version: text::Version::default(),
        }
    }
}

impl fmt::Debug for Internal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Internal")
            .field("font", &self.font)
            .field("bounds", &self.bounds)
            .field("min_bounds", &self.min_bounds)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct Weak {
    raw: sync::Weak<Internal>,
    pub bounds: Size,
}

impl Weak {
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

fn highlight_line<'a>(
    line: &'a cosmic_text::BufferLine,
    from: usize,
    to: usize,
) -> impl Iterator<Item = (f32, f32)> + 'a {
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
