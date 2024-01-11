//! Draw paragraphs.
use crate::core;
use crate::core::alignment;
use crate::core::text::{Hit, LineHeight, Shaping, Text};
use crate::core::{Font, Pixels, Point, Size};
use crate::text;

use std::fmt;
use std::sync::{self, Arc};

/// A bunch of text.
#[derive(Clone, PartialEq)]
pub struct Paragraph(Option<Arc<Internal>>);

struct Internal {
    buffer: cosmic_text::Buffer,
    content: String, // TODO: Reuse from `buffer` (?)
    font: Font,
    shaping: Shaping,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    bounds: Size,
    min_bounds: Size,
    version: text::Version,
}

impl Paragraph {
    /// Creates a new empty [`Paragraph`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the buffer of the [`Paragraph`].
    pub fn buffer(&self) -> &cosmic_text::Buffer {
        &self.internal().buffer
    }

    /// Creates a [`Weak`] reference to the [`Paragraph`].
    ///
    /// This is useful to avoid cloning the [`Paragraph`] when
    /// referential guarantees are unnecessary. For instance,
    /// when creating a rendering tree.
    pub fn downgrade(&self) -> Weak {
        let paragraph = self.internal();

        Weak {
            raw: Arc::downgrade(paragraph),
            min_bounds: paragraph.min_bounds,
            horizontal_alignment: paragraph.horizontal_alignment,
            vertical_alignment: paragraph.vertical_alignment,
        }
    }

    fn internal(&self) -> &Arc<Internal> {
        self.0
            .as_ref()
            .expect("paragraph should always be initialized")
    }
}

impl core::text::Paragraph for Paragraph {
    type Font = Font;

    fn with_text(text: Text<'_, Font>) -> Self {
        log::trace!("Allocating paragraph: {}", text.content);

        let mut font_system =
            text::font_system().write().expect("Write font system");

        let mut buffer = cosmic_text::Buffer::new(
            font_system.raw(),
            cosmic_text::Metrics::new(
                text.size.into(),
                text.line_height.to_absolute(text.size).into(),
            ),
        );

        buffer.set_size(
            font_system.raw(),
            text.bounds.width,
            text.bounds.height,
        );

        buffer.set_text(
            font_system.raw(),
            text.content,
            text::to_attributes(text.font),
            text::to_shaping(text.shaping),
        );

        let min_bounds = text::measure(&buffer);

        Self(Some(Arc::new(Internal {
            buffer,
            content: text.content.to_owned(),
            font: text.font,
            horizontal_alignment: text.horizontal_alignment,
            vertical_alignment: text.vertical_alignment,
            shaping: text.shaping,
            bounds: text.bounds,
            min_bounds,
            version: font_system.version(),
        })))
    }

    fn resize(&mut self, new_bounds: Size) {
        let paragraph = self
            .0
            .take()
            .expect("paragraph should always be initialized");

        match Arc::try_unwrap(paragraph) {
            Ok(mut internal) => {
                let mut font_system =
                    text::font_system().write().expect("Write font system");

                internal.buffer.set_size(
                    font_system.raw(),
                    new_bounds.width,
                    new_bounds.height,
                );

                internal.bounds = new_bounds;
                internal.min_bounds = text::measure(&internal.buffer);

                self.0 = Some(Arc::new(internal));
            }
            Err(internal) => {
                let metrics = internal.buffer.metrics();

                // If there is a strong reference somewhere, we recompute the
                // buffer from scratch
                *self = Self::with_text(Text {
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
                });
            }
        }
    }

    fn compare(&self, text: Text<'_, Font>) -> core::text::Difference {
        let font_system = text::font_system().read().expect("Read font system");
        let paragraph = self.internal();
        let metrics = paragraph.buffer.metrics();

        if paragraph.version != font_system.version
            || paragraph.content != text.content
            || metrics.font_size != text.size.0
            || metrics.line_height != text.line_height.to_absolute(text.size).0
            || paragraph.font != text.font
            || paragraph.shaping != text.shaping
            || paragraph.horizontal_alignment != text.horizontal_alignment
            || paragraph.vertical_alignment != text.vertical_alignment
        {
            core::text::Difference::Shape
        } else if paragraph.bounds != text.bounds {
            core::text::Difference::Bounds
        } else {
            core::text::Difference::None
        }
    }

    fn horizontal_alignment(&self) -> alignment::Horizontal {
        self.internal().horizontal_alignment
    }

    fn vertical_alignment(&self) -> alignment::Vertical {
        self.internal().vertical_alignment
    }

    fn min_bounds(&self) -> Size {
        self.internal().min_bounds
    }

    fn hit_test(&self, point: Point) -> Option<Hit> {
        let cursor = self.internal().buffer.hit(point.x, point.y)?;

        Some(Hit::CharOffset(cursor.index))
    }

    fn grapheme_position(&self, line: usize, index: usize) -> Option<Point> {
        use unicode_segmentation::UnicodeSegmentation;

        let run = self.internal().buffer.layout_runs().nth(line)?;

        // index represents a grapheme, not a glyph
        // Let's find the first glyph for the given grapheme cluster
        let mut last_start = None;
        let mut last_grapheme_count = 0;
        let mut graphemes_seen = 0;

        let glyph = run
            .glyphs
            .iter()
            .find(|glyph| {
                if Some(glyph.start) != last_start {
                    last_grapheme_count = run.text[glyph.start..glyph.end]
                        .graphemes(false)
                        .count();
                    last_start = Some(glyph.start);
                    graphemes_seen += last_grapheme_count;
                }

                graphemes_seen >= index
            })
            .or_else(|| run.glyphs.last())?;

        let advance = if index == 0 {
            0.0
        } else {
            glyph.w
                * (1.0
                    - graphemes_seen.saturating_sub(index) as f32
                        / last_grapheme_count.max(1) as f32)
        };

        Some(Point::new(
            glyph.x + glyph.x_offset * glyph.font_size + advance,
            glyph.y - glyph.y_offset * glyph.font_size,
        ))
    }
}

impl Default for Paragraph {
    fn default() -> Self {
        Self(Some(Arc::new(Internal::default())))
    }
}

impl fmt::Debug for Paragraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let paragraph = self.internal();

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
            version: text::Version::default(),
        }
    }
}

/// A weak reference to a [`Paragraph`].
#[derive(Debug, Clone)]
pub struct Weak {
    raw: sync::Weak<Internal>,
    /// The minimum bounds of the [`Paragraph`].
    pub min_bounds: Size,
    /// The horizontal alignment of the [`Paragraph`].
    pub horizontal_alignment: alignment::Horizontal,
    /// The vertical alignment of the [`Paragraph`].
    pub vertical_alignment: alignment::Vertical,
}

impl Weak {
    /// Tries to update the reference into a [`Paragraph`].
    pub fn upgrade(&self) -> Option<Paragraph> {
        self.raw.upgrade().map(Some).map(Paragraph)
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
