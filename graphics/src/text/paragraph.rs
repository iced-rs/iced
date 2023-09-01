use crate::core;
use crate::core::alignment;
use crate::core::text::{Hit, LineHeight, Shaping, Text};
use crate::core::{Font, Pixels, Point, Size};
use crate::text::{self, FontSystem};

use std::fmt;
use std::sync::{self, Arc};

#[derive(PartialEq, Default)]
pub struct Paragraph(Arc<Internal>);

struct Internal {
    buffer: cosmic_text::Buffer,
    content: String, // TODO: Reuse from `buffer` (?)
    font: Font,
    shaping: Shaping,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    bounds: Size,
    min_bounds: Size,
}

impl Paragraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_text(text: Text<'_, Font>, font_system: &FontSystem) -> Self {
        let mut font_system = font_system.write();

        let mut buffer = cosmic_text::Buffer::new(
            &mut font_system,
            cosmic_text::Metrics::new(
                text.size.into(),
                text.line_height.to_absolute(text.size).into(),
            ),
        );

        buffer.set_size(
            &mut font_system,
            text.bounds.width,
            text.bounds.height,
        );

        buffer.set_text(
            &mut font_system,
            text.content,
            text::to_attributes(text.font),
            text::to_shaping(text.shaping),
        );

        let min_bounds = text::measure(&buffer);

        Self(Arc::new(Internal {
            buffer,
            content: text.content.to_owned(),
            font: text.font,
            horizontal_alignment: text.horizontal_alignment,
            vertical_alignment: text.vertical_alignment,
            shaping: text.shaping,
            bounds: text.bounds,
            min_bounds,
        }))
    }

    pub fn buffer(&self) -> &cosmic_text::Buffer {
        &self.0.buffer
    }

    pub fn downgrade(&self) -> Weak {
        Weak {
            raw: Arc::downgrade(&self.0),
            min_bounds: self.0.min_bounds,
            horizontal_alignment: self.0.horizontal_alignment,
            vertical_alignment: self.0.vertical_alignment,
        }
    }

    pub fn resize(&mut self, new_bounds: Size, font_system: &FontSystem) {
        if let Some(internal) = Arc::get_mut(&mut self.0) {
            // If there is no strong reference holding on to the paragraph, we
            // resize the buffer in-place
            internal.buffer.set_size(
                &mut font_system.write(),
                new_bounds.width,
                new_bounds.height,
            );

            internal.bounds = new_bounds;
            internal.min_bounds = text::measure(&internal.buffer);
        } else {
            let metrics = self.0.buffer.metrics();

            // If there is a strong reference somewhere, we recompute the buffer
            // from scratch
            *self = Self::with_text(
                Text {
                    content: &self.0.content,
                    bounds: self.0.bounds,
                    size: Pixels(metrics.font_size),
                    line_height: LineHeight::Absolute(Pixels(
                        metrics.line_height,
                    )),
                    font: self.0.font,
                    horizontal_alignment: self.0.horizontal_alignment,
                    vertical_alignment: self.0.vertical_alignment,
                    shaping: self.0.shaping,
                },
                font_system,
            );
        }
    }
}

impl core::text::Paragraph for Paragraph {
    type Font = Font;

    fn content(&self) -> &str {
        &self.0.content
    }

    fn text_size(&self) -> Pixels {
        Pixels(self.0.buffer.metrics().font_size)
    }

    fn line_height(&self) -> LineHeight {
        LineHeight::Absolute(Pixels(self.0.buffer.metrics().line_height))
    }

    fn font(&self) -> Font {
        self.0.font
    }

    fn shaping(&self) -> Shaping {
        self.0.shaping
    }

    fn horizontal_alignment(&self) -> alignment::Horizontal {
        self.0.horizontal_alignment
    }

    fn vertical_alignment(&self) -> alignment::Vertical {
        self.0.vertical_alignment
    }

    fn bounds(&self) -> Size {
        self.0.bounds
    }

    fn min_bounds(&self) -> Size {
        self.0.min_bounds
    }

    fn hit_test(&self, point: Point) -> Option<Hit> {
        let cursor = self.0.buffer.hit(point.x, point.y)?;

        Some(Hit::CharOffset(cursor.index))
    }

    fn grapheme_position(&self, line: usize, index: usize) -> Option<Point> {
        let run = self.0.buffer.layout_runs().nth(line)?;

        // TODO: Index represents a grapheme, not a glyph
        let glyph = run.glyphs.get(index).or_else(|| run.glyphs.last())?;

        let advance_last = if index == run.glyphs.len() {
            glyph.w
        } else {
            0.0
        };

        Some(Point::new(
            glyph.x + glyph.x_offset * glyph.font_size + advance_last,
            glyph.y - glyph.y_offset * glyph.font_size,
        ))
    }
}

impl PartialEq for Internal {
    fn eq(&self, other: &Self) -> bool {
        self.content == other.content
            && self.font == other.font
            && self.shaping == other.shaping
            && self.horizontal_alignment == other.horizontal_alignment
            && self.vertical_alignment == other.vertical_alignment
            && self.bounds == other.bounds
            && self.min_bounds == other.min_bounds
            && self.buffer.metrics() == other.buffer.metrics()
    }
}

impl Default for Internal {
    fn default() -> Self {
        Self {
            buffer: cosmic_text::Buffer::new_empty(cosmic_text::Metrics {
                font_size: 1.0,
                line_height: 1.0,
            }),
            content: String::new(),
            font: Font::default(),
            shaping: Shaping::default(),
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
            bounds: Size::ZERO,
            min_bounds: Size::ZERO,
        }
    }
}

impl fmt::Debug for Paragraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Paragraph")
            .field("content", &self.0.content)
            .field("font", &self.0.font)
            .field("shaping", &self.0.shaping)
            .field("horizontal_alignment", &self.0.horizontal_alignment)
            .field("vertical_alignment", &self.0.vertical_alignment)
            .field("bounds", &self.0.bounds)
            .field("min_bounds", &self.0.min_bounds)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct Weak {
    raw: sync::Weak<Internal>,
    pub min_bounds: Size,
    pub horizontal_alignment: alignment::Horizontal,
    pub vertical_alignment: alignment::Vertical,
}

impl Weak {
    pub fn upgrade(&self) -> Option<Paragraph> {
        self.raw.upgrade().map(Paragraph)
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
