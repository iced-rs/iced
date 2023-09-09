use crate::core;
use crate::core::alignment;
use crate::core::text::{Hit, LineHeight, Shaping, Text};
use crate::core::{Font, Pixels, Point, Size};
use crate::text::{self, FontSystem};

use std::fmt;
use std::sync::{self, Arc};

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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_text(text: Text<'_, Font>, font_system: &FontSystem) -> Self {
        log::trace!("Allocating paragraph: {}", text.content);

        let (mut font_system, version) = font_system.write();

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

        Self(Some(Arc::new(Internal {
            buffer,
            content: text.content.to_owned(),
            font: text.font,
            horizontal_alignment: text.horizontal_alignment,
            vertical_alignment: text.vertical_alignment,
            shaping: text.shaping,
            bounds: text.bounds,
            min_bounds,
            version,
        })))
    }

    pub fn buffer(&self) -> &cosmic_text::Buffer {
        &self.internal().buffer
    }

    pub fn version(&self) -> text::Version {
        self.internal().version
    }

    pub fn downgrade(&self) -> Weak {
        let paragraph = self.internal();

        Weak {
            raw: Arc::downgrade(paragraph),
            min_bounds: paragraph.min_bounds,
            horizontal_alignment: paragraph.horizontal_alignment,
            vertical_alignment: paragraph.vertical_alignment,
        }
    }

    pub fn resize(&mut self, new_bounds: Size, font_system: &FontSystem) {
        let paragraph = self
            .0
            .take()
            .expect("paragraph should always be initialized");

        match Arc::try_unwrap(paragraph) {
            Ok(mut internal) => {
                let (mut font_system, _) = font_system.write();

                internal.buffer.set_size(
                    &mut font_system,
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
                *self = Self::with_text(
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
            }
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

    fn content(&self) -> &str {
        &self.internal().content
    }

    fn text_size(&self) -> Pixels {
        Pixels(self.internal().buffer.metrics().font_size)
    }

    fn line_height(&self) -> LineHeight {
        LineHeight::Absolute(Pixels(
            self.internal().buffer.metrics().line_height,
        ))
    }

    fn font(&self) -> Font {
        self.internal().font
    }

    fn shaping(&self) -> Shaping {
        self.internal().shaping
    }

    fn horizontal_alignment(&self) -> alignment::Horizontal {
        self.internal().horizontal_alignment
    }

    fn vertical_alignment(&self) -> alignment::Vertical {
        self.internal().vertical_alignment
    }

    fn bounds(&self) -> Size {
        self.internal().bounds
    }

    fn min_bounds(&self) -> Size {
        self.internal().min_bounds
    }

    fn hit_test(&self, point: Point) -> Option<Hit> {
        let cursor = self.internal().buffer.hit(point.x, point.y)?;

        Some(Hit::CharOffset(cursor.index))
    }

    fn grapheme_position(&self, line: usize, index: usize) -> Option<Point> {
        let run = self.internal().buffer.layout_runs().nth(line)?;

        // index represents a grapheme, not a glyph
        // Let's find the first glyph for the given grapheme cluster
        let mut last_start = None;
        let mut graphemes_seen = 0;

        let glyph = run
            .glyphs
            .iter()
            .find(|glyph| {
                if graphemes_seen == index {
                    return true;
                }

                if Some(glyph.start) != last_start {
                    last_start = Some(glyph.start);
                    graphemes_seen += 1;
                }

                false
            })
            .or_else(|| run.glyphs.last())?;

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

#[derive(Debug, Clone)]
pub struct Weak {
    raw: sync::Weak<Internal>,
    pub min_bounds: Size,
    pub horizontal_alignment: alignment::Horizontal,
    pub vertical_alignment: alignment::Vertical,
}

impl Weak {
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
