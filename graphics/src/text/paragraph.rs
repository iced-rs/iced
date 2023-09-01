use crate::core;
use crate::core::alignment;
use crate::core::text::{Hit, LineHeight, Shaping, Text};
use crate::core::{Font, Pixels, Point, Size};
use crate::text::{self, FontSystem};

use std::fmt;
use std::mem::MaybeUninit;
use std::sync::{self, Arc};

pub struct Paragraph(MaybeUninit<Arc<Internal>>);

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

impl Default for Paragraph {
    fn default() -> Self {
        Self(MaybeUninit::new(Arc::new(Internal::default())))
    }
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

        Self(MaybeUninit::new(Arc::new(Internal {
            buffer,
            content: text.content.to_owned(),
            font: text.font,
            horizontal_alignment: text.horizontal_alignment,
            vertical_alignment: text.vertical_alignment,
            shaping: text.shaping,
            bounds: text.bounds,
            min_bounds,
        })))
    }

    pub fn buffer(&self) -> &cosmic_text::Buffer {
        #[allow(unsafe_code)]
        &unsafe { self.0.assume_init_ref() }.buffer
    }

    pub fn downgrade(&self) -> Weak {
        #[allow(unsafe_code)]
        let paragraph = unsafe { self.0.assume_init_ref() };

        Weak {
            raw: Arc::downgrade(paragraph),
            min_bounds: paragraph.min_bounds,
            horizontal_alignment: paragraph.horizontal_alignment,
            vertical_alignment: paragraph.vertical_alignment,
        }
    }

    pub fn resize(&mut self, new_bounds: Size, font_system: &FontSystem) {
        // Place uninit for now, we always write to `self.0` in the end
        let paragraph = std::mem::replace(&mut self.0, MaybeUninit::uninit());

        // Mutable self guarantees unique access and `uninit` call only happens
        // in this method.
        #[allow(unsafe_code)]
        let paragraph = unsafe { paragraph.assume_init() };

        match Arc::try_unwrap(paragraph) {
            Ok(mut internal) => {
                internal.buffer.set_size(
                    &mut font_system.write(),
                    new_bounds.width,
                    new_bounds.height,
                );

                internal.bounds = new_bounds;
                internal.min_bounds = text::measure(&internal.buffer);

                let _ = self.0.write(Arc::new(internal));
            }
            Err(internal) => {
                let metrics = internal.buffer.metrics();

                // If there is a strong reference somewhere, we recompute the
                // buffer from scratch
                let new_paragraph = Self::with_text(
                    Text {
                        content: &internal.content,
                        bounds: internal.bounds,
                        size: Pixels(metrics.font_size),
                        line_height: LineHeight::Absolute(Pixels(
                            metrics.line_height,
                        )),
                        font: internal.font,
                        horizontal_alignment: internal.horizontal_alignment,
                        vertical_alignment: internal.vertical_alignment,
                        shaping: internal.shaping,
                    },
                    font_system,
                );

                // New paragraph should always be initialized
                #[allow(unsafe_code)]
                let _ = self.0.write(unsafe { new_paragraph.0.assume_init() });
            }
        }
    }

    fn internal_ref(&self) -> &Internal {
        #[allow(unsafe_code)]
        unsafe {
            self.0.assume_init_ref()
        }
    }
}

impl core::text::Paragraph for Paragraph {
    type Font = Font;

    fn content(&self) -> &str {
        &self.internal_ref().content
    }

    fn text_size(&self) -> Pixels {
        Pixels(self.internal_ref().buffer.metrics().font_size)
    }

    fn line_height(&self) -> LineHeight {
        LineHeight::Absolute(Pixels(
            self.internal_ref().buffer.metrics().line_height,
        ))
    }

    fn font(&self) -> Font {
        self.internal_ref().font
    }

    fn shaping(&self) -> Shaping {
        self.internal_ref().shaping
    }

    fn horizontal_alignment(&self) -> alignment::Horizontal {
        self.internal_ref().horizontal_alignment
    }

    fn vertical_alignment(&self) -> alignment::Vertical {
        self.internal_ref().vertical_alignment
    }

    fn bounds(&self) -> Size {
        self.internal_ref().bounds
    }

    fn min_bounds(&self) -> Size {
        self.internal_ref().min_bounds
    }

    fn hit_test(&self, point: Point) -> Option<Hit> {
        let cursor = self.internal_ref().buffer.hit(point.x, point.y)?;

        Some(Hit::CharOffset(cursor.index))
    }

    fn grapheme_position(&self, line: usize, index: usize) -> Option<Point> {
        let run = self.internal_ref().buffer.layout_runs().nth(line)?;

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
        let paragraph = self.internal_ref();

        f.debug_struct("Paragraph")
            .field("content", &paragraph.content)
            .field("font", &paragraph.font)
            .field("shaping", &paragraph.shaping)
            .field("horizontal_alignment", &paragraph.horizontal_alignment)
            .field("vertical_alignment", &paragraph.vertical_alignment)
            .field("bounds", &paragraph.bounds)
            .field("min_bounds", &paragraph.min_bounds)
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
        self.raw.upgrade().map(MaybeUninit::new).map(Paragraph)
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
