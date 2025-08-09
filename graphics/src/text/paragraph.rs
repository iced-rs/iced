//! Draw paragraphs.
use crate::core;
use crate::core::alignment;
use crate::core::text::{
    Alignment, Hit, LineHeight, Shaping, Span, Text, Wrapping,
};
use crate::core::{Font, Pixels, Point, Rectangle, Size};
use crate::text;

use std::fmt;
use std::sync::{self, Arc};

/// A bunch of text.
#[derive(Clone, PartialEq)]
pub struct Paragraph(Arc<Internal>);

#[derive(Clone)]
struct Internal {
    buffer: cosmic_text::Buffer,
    font: Font,
    shaping: Shaping,
    wrapping: Wrapping,
    align_x: Alignment,
    align_y: alignment::Vertical,
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
            align_x: paragraph.align_x,
            align_y: paragraph.align_y,
        }
    }

    fn internal(&self) -> &Arc<Internal> {
        &self.0
    }
}

impl core::text::Paragraph for Paragraph {
    type Font = Font;

    fn with_text(text: Text<&str>) -> Self {
        log::trace!("Allocating plain paragraph: {}", text.content);

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
            Some(text.bounds.width),
            Some(text.bounds.height),
        );

        buffer.set_wrap(font_system.raw(), text::to_wrap(text.wrapping));

        buffer.set_text(
            font_system.raw(),
            text.content,
            &text::to_attributes(text.font),
            text::to_shaping(text.shaping),
        );

        let min_bounds =
            text::align(&mut buffer, font_system.raw(), text.align_x);

        Self(Arc::new(Internal {
            buffer,
            font: text.font,
            align_x: text.align_x,
            align_y: text.align_y,
            shaping: text.shaping,
            wrapping: text.wrapping,
            bounds: text.bounds,
            min_bounds,
            version: font_system.version(),
        }))
    }

    fn with_spans<Link>(text: Text<&[Span<'_, Link>]>) -> Self {
        log::trace!("Allocating rich paragraph: {} spans", text.content.len());

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
            Some(text.bounds.width),
            Some(text.bounds.height),
        );

        buffer.set_wrap(font_system.raw(), text::to_wrap(text.wrapping));

        buffer.set_rich_text(
            font_system.raw(),
            text.content.iter().enumerate().map(|(i, span)| {
                let attrs = text::to_attributes(span.font.unwrap_or(text.font));

                let attrs = match (span.size, span.line_height) {
                    (None, None) => attrs,
                    _ => {
                        let size = span.size.unwrap_or(text.size);

                        attrs.metrics(cosmic_text::Metrics::new(
                            size.into(),
                            span.line_height
                                .unwrap_or(text.line_height)
                                .to_absolute(size)
                                .into(),
                        ))
                    }
                };

                let attrs = if let Some(color) = span.color {
                    attrs.color(text::to_color(color))
                } else {
                    attrs
                };

                (span.text.as_ref(), attrs.metadata(i))
            }),
            &text::to_attributes(text.font),
            text::to_shaping(text.shaping),
            None,
        );

        let min_bounds =
            text::align(&mut buffer, font_system.raw(), text.align_x);

        Self(Arc::new(Internal {
            buffer,
            font: text.font,
            align_x: text.align_x,
            align_y: text.align_y,
            shaping: text.shaping,
            wrapping: text.wrapping,
            bounds: text.bounds,
            min_bounds,
            version: font_system.version(),
        }))
    }

    fn resize(&mut self, new_bounds: Size) {
        let paragraph = Arc::make_mut(&mut self.0);

        let mut font_system =
            text::font_system().write().expect("Write font system");

        paragraph.buffer.set_size(
            font_system.raw(),
            Some(new_bounds.width),
            Some(new_bounds.height),
        );

        let min_bounds = text::align(
            &mut paragraph.buffer,
            font_system.raw(),
            paragraph.align_x,
        );

        paragraph.bounds = new_bounds;
        paragraph.min_bounds = min_bounds;
    }

    fn compare(&self, text: Text<()>) -> core::text::Difference {
        let font_system = text::font_system().read().expect("Read font system");
        let paragraph = self.internal();
        let metrics = paragraph.buffer.metrics();

        if paragraph.version != font_system.version
            || metrics.font_size != text.size.0
            || metrics.line_height != text.line_height.to_absolute(text.size).0
            || paragraph.font != text.font
            || paragraph.shaping != text.shaping
            || paragraph.wrapping != text.wrapping
            || paragraph.align_x != text.align_x
            || paragraph.align_y != text.align_y
        {
            core::text::Difference::Shape
        } else if paragraph.bounds != text.bounds {
            core::text::Difference::Bounds
        } else {
            core::text::Difference::None
        }
    }

    fn size(&self) -> Pixels {
        Pixels(self.0.buffer.metrics().font_size)
    }

    fn font(&self) -> Font {
        self.0.font
    }

    fn line_height(&self) -> LineHeight {
        LineHeight::Absolute(Pixels(self.0.buffer.metrics().line_height))
    }

    fn align_x(&self) -> Alignment {
        self.internal().align_x
    }

    fn align_y(&self) -> alignment::Vertical {
        self.internal().align_y
    }

    fn wrapping(&self) -> Wrapping {
        self.0.wrapping
    }

    fn shaping(&self) -> Shaping {
        self.0.shaping
    }

    fn bounds(&self) -> Size {
        self.0.bounds
    }

    fn min_bounds(&self) -> Size {
        self.internal().min_bounds
    }

    fn hit_test(&self, point: Point) -> Option<Hit> {
        let cursor = self.internal().buffer.hit(point.x, point.y)?;

        Some(Hit::CharOffset(cursor.index))
    }

    fn hit_span(&self, point: Point) -> Option<usize> {
        let internal = self.internal();

        let cursor = internal.buffer.hit(point.x, point.y)?;
        let line = internal.buffer.lines.get(cursor.line)?;

        let mut last_glyph = None;
        let mut glyphs = line
            .layout_opt()
            .as_ref()?
            .iter()
            .flat_map(|line| line.glyphs.iter())
            .peekable();

        while let Some(glyph) = glyphs.peek() {
            if glyph.start <= cursor.index && cursor.index < glyph.end {
                break;
            }

            last_glyph = glyphs.next();
        }

        let glyph = match cursor.affinity {
            cosmic_text::Affinity::Before => last_glyph,
            cosmic_text::Affinity::After => glyphs.next(),
        }?;

        Some(glyph.metadata)
    }

    fn span_bounds(&self, index: usize) -> Vec<Rectangle> {
        let internal = self.internal();

        let mut bounds = Vec::new();
        let mut current_bounds = None;

        let glyphs = internal
            .buffer
            .layout_runs()
            .flat_map(|run| {
                let line_top = run.line_top;
                let line_height = run.line_height;

                run.glyphs
                    .iter()
                    .map(move |glyph| (line_top, line_height, glyph))
            })
            .skip_while(|(_, _, glyph)| glyph.metadata != index)
            .take_while(|(_, _, glyph)| glyph.metadata == index);

        for (line_top, line_height, glyph) in glyphs {
            let y = line_top + glyph.y;

            let new_bounds = || {
                Rectangle::new(
                    Point::new(glyph.x, y),
                    Size::new(
                        glyph.w,
                        glyph.line_height_opt.unwrap_or(line_height),
                    ),
                )
            };

            match current_bounds.as_mut() {
                None => {
                    current_bounds = Some(new_bounds());
                }
                Some(current_bounds) if y != current_bounds.y => {
                    bounds.push(*current_bounds);
                    *current_bounds = new_bounds();
                }
                Some(current_bounds) => {
                    current_bounds.width += glyph.w;
                }
            }
        }

        bounds.extend(current_bounds);
        bounds
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
        Self(Arc::new(Internal::default()))
    }
}

impl fmt::Debug for Paragraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let paragraph = self.internal();

        f.debug_struct("Paragraph")
            .field("font", &paragraph.font)
            .field("shaping", &paragraph.shaping)
            .field("horizontal_alignment", &paragraph.align_x)
            .field("vertical_alignment", &paragraph.align_y)
            .field("bounds", &paragraph.bounds)
            .field("min_bounds", &paragraph.min_bounds)
            .finish()
    }
}

impl PartialEq for Internal {
    fn eq(&self, other: &Self) -> bool {
        self.font == other.font
            && self.shaping == other.shaping
            && self.align_x == other.align_x
            && self.align_y == other.align_y
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
            font: Font::default(),
            shaping: Shaping::default(),
            wrapping: Wrapping::default(),
            align_x: Alignment::Default,
            align_y: alignment::Vertical::Top,
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
    pub align_x: Alignment,
    /// The vertical alignment of the [`Paragraph`].
    pub align_y: alignment::Vertical,
}

impl Weak {
    /// Tries to update the reference into a [`Paragraph`].
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
