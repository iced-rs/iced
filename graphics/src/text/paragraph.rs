//! Draw paragraphs.
use crate::core;
use crate::core::alignment;
use crate::core::text::{Alignment, Ellipsis, Hit, LineHeight, Shaping, Span, Text, Wrapping};
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
    ellipsis: Ellipsis,
    align_x: Alignment,
    align_y: alignment::Vertical,
    bounds: Size,
    min_bounds: Size,
    version: text::Version,
    hint: bool,
    hint_factor: f32,
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
    fn with_text(text: Text<&str>) -> Self {
        log::trace!("Allocating plain paragraph: {}", text.content);

        let mut font_system = text::font_system().write().expect("Write font system");

        let (hint, hint_factor) = match text::hint_factor(text.size, text.hint_factor) {
            Some(hint_factor) => (true, hint_factor),
            _ => (false, 1.0),
        };

        let mut buffer = cosmic_text::Buffer::new(
            font_system.raw(),
            cosmic_text::Metrics::new(
                f32::from(text.size) * hint_factor,
                f32::from(text.line_height.to_absolute(text.size)) * hint_factor,
            ),
        );

        if hint {
            buffer.set_hinting(cosmic_text::Hinting::Enabled);
        }

        buffer.set_size(
            Some(text.bounds.width * hint_factor),
            Some(text.bounds.height * hint_factor),
        );

        buffer.set_wrap(text::to_wrap(text.wrapping));
        buffer.set_ellipsize(text::to_ellipsize(
            text.ellipsis,
            text.bounds.height * hint_factor,
        ));

        buffer.set_text(
            text.content,
            &text::to_attributes(text.font),
            text::to_shaping(text.shaping, text.content),
            None,
        );
        buffer.shape_until_scroll(font_system.raw(), false);

        let min_bounds = text::align(&mut buffer, font_system.raw(), text.align_x) / hint_factor;

        Self(Arc::new(Internal {
            buffer,
            hint,
            hint_factor,
            font: text.font,
            align_x: text.align_x,
            align_y: text.align_y,
            shaping: text.shaping,
            wrapping: text.wrapping,
            ellipsis: text.ellipsis,
            bounds: text.bounds,
            min_bounds,
            version: font_system.version(),
        }))
    }

    fn with_spans<Link>(text: Text<&[Span<'_, Link>]>) -> Self {
        log::trace!("Allocating rich paragraph: {} spans", text.content.len());

        let mut font_system = text::font_system().write().expect("Write font system");

        let (hint, hint_factor) = match text::hint_factor(text.size, text.hint_factor) {
            Some(hint_factor) => (true, hint_factor),
            _ => (false, 1.0),
        };

        let mut buffer = cosmic_text::Buffer::new(
            font_system.raw(),
            cosmic_text::Metrics::new(
                f32::from(text.size) * hint_factor,
                f32::from(text.line_height.to_absolute(text.size)) * hint_factor,
            ),
        );

        if hint {
            buffer.set_hinting(cosmic_text::Hinting::Enabled);
        }

        buffer.set_size(
            Some(text.bounds.width * hint_factor),
            Some(text.bounds.height * hint_factor),
        );

        buffer.set_wrap(text::to_wrap(text.wrapping));

        buffer.set_rich_text(
            text.content.iter().enumerate().map(|(i, span)| {
                let attrs = text::to_attributes(span.font.unwrap_or(text.font));

                let attrs = match (span.size, span.line_height) {
                    (None, None) => attrs,
                    _ => {
                        let size = span.size.unwrap_or(text.size);

                        attrs.metrics(cosmic_text::Metrics::new(
                            f32::from(size) * hint_factor,
                            f32::from(
                                span.line_height
                                    .unwrap_or(text.line_height)
                                    .to_absolute(size),
                            ) * hint_factor,
                        ))
                    }
                };

                let attrs = if let Some(color) = span.color {
                    attrs.color(text::to_color(color))
                } else {
                    attrs
                };

                let attrs = attrs.padding(cosmic_text::SpanPadding {
                    // top: span.padding.top,
                    // bottom: span.padding.bottom,
                    start: span.padding.left,
                    end: span.padding.right,
                });

                (span.text.as_ref(), attrs.metadata(i))
            }),
            &text::to_attributes(text.font),
            cosmic_text::Shaping::Advanced,
            None,
        );

        buffer.shape_until_scroll(font_system.raw(), false);

        let min_bounds = text::align(&mut buffer, font_system.raw(), text.align_x) / hint_factor;

        Self(Arc::new(Internal {
            buffer,
            hint,
            hint_factor,
            font: text.font,
            align_x: text.align_x,
            align_y: text.align_y,
            shaping: text.shaping,
            wrapping: text.wrapping,
            ellipsis: text.ellipsis,
            bounds: text.bounds,
            min_bounds,
            version: font_system.version(),
        }))
    }

    fn resize(&mut self, new_bounds: Size) {
        let paragraph = Arc::make_mut(&mut self.0);

        let mut font_system = text::font_system().write().expect("Write font system");

        paragraph.buffer.set_size(
            Some(new_bounds.width * paragraph.hint_factor),
            Some(new_bounds.height * paragraph.hint_factor),
        );
        paragraph
            .buffer
            .shape_until_scroll(font_system.raw(), false);

        let min_bounds = text::align(&mut paragraph.buffer, font_system.raw(), paragraph.align_x)
            / paragraph.hint_factor;

        paragraph.bounds = new_bounds;
        paragraph.min_bounds = min_bounds;
    }

    fn compare(&self, text: Text<()>) -> core::text::Difference {
        let font_system = text::font_system().read().expect("Read font system");
        let paragraph = self.internal();
        let metrics = paragraph.buffer.metrics();

        if paragraph.version != font_system.version
            || metrics.font_size != text.size.0 * paragraph.hint_factor
            || metrics.line_height
                != text.line_height.to_absolute(text.size).0 * paragraph.hint_factor
            || paragraph.font != text.font
            || paragraph.shaping != text.shaping
            || paragraph.wrapping != text.wrapping
            || paragraph.ellipsis != text.ellipsis
            || paragraph.align_x != text.align_x
            || paragraph.align_y != text.align_y
            || paragraph.hint.then_some(paragraph.hint_factor)
                != text::hint_factor(text.size, text.hint_factor)
        {
            core::text::Difference::Shape
        } else if paragraph.bounds != text.bounds {
            core::text::Difference::Bounds
        } else {
            core::text::Difference::None
        }
    }

    fn hint_factor(&self) -> Option<f32> {
        self.0.hint.then_some(self.0.hint_factor)
    }

    fn size(&self) -> Pixels {
        Pixels(self.0.buffer.metrics().font_size / self.0.hint_factor)
    }

    fn font(&self) -> Font {
        self.0.font
    }

    fn line_height(&self) -> LineHeight {
        LineHeight::Absolute(Pixels(
            self.0.buffer.metrics().line_height / self.0.hint_factor,
        ))
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

    fn ellipsis(&self) -> Ellipsis {
        self.0.ellipsis
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
        let cursor = self
            .internal()
            .buffer
            .hit(point.x * self.0.hint_factor, point.y * self.0.hint_factor)?;

        Some(Hit::CharOffset(cursor.index))
    }

    fn hit_span(&self, point: Point) -> Option<usize> {
        let internal = self.internal();

        let cursor = internal
            .buffer
            .hit(point.x * self.0.hint_factor, point.y * self.0.hint_factor)?;
        let line = internal.buffer.lines.get(cursor.line)?;

        if cursor.index >= line.text().len() {
            return None;
        }

        let index = match cursor.affinity {
            cosmic_text::Affinity::Before => cursor.index.saturating_sub(1),
            cosmic_text::Affinity::After => cursor.index,
        };

        let mut hit = None;
        let glyphs = line
            .layout_opt()
            .as_ref()?
            .iter()
            .flat_map(|line| line.glyphs.iter());

        for glyph in glyphs {
            if glyph.start <= index && index < glyph.end {
                hit = Some(glyph);
                break;
            }
        }

        Some(hit?.metadata)
    }

    fn span_bounds(&self, index: usize) -> Vec<Rectangle> {
        let internal = self.internal();

        let scale = 1.0 / internal.hint_factor;

        let mut bounds = Vec::new();
        let mut current = None;
        let mut current_baseline = 0.0;

        let mut y = 0.0;
        let buffer_height = internal.buffer.metrics().line_height;
        let glyphs = internal
            .buffer
            .lines
            .iter()
            .filter_map(|paragraph| paragraph.layout_opt().map(Vec::as_slice))
            .flat_map(|lines| lines.iter())
            .flat_map(move |line| {
                let line_height = line.line_height(buffer_height);
                let ink_height = line.max_ascent + line.max_descent;

                // The renderer centers the line's ink within the line
                // height and places the baseline `max_ascent` below the
                // top of the ink:
                let baseline = y + (line_height - ink_height) / 2.0 + line.max_ascent;

                let glyphs = line.glyphs.iter().map(move |glyph| (baseline, glyph));

                y += line_height;

                glyphs
            })
            .skip_while(|(_, glyph)| glyph.metadata != index)
            .take_while(|(_, glyph)| glyph.metadata == index);

        for (baseline, glyph) in glyphs {
            // Glyphs can be offset from the line's baseline (e.g. by complex
            // scripts); mirror the anchor used in `LayoutGlyph::physical`:
            let anchor = baseline + glyph.y - glyph.font_size * glyph.y_offset;
            let ink_top = anchor - glyph.ascender;
            let ink_bottom = anchor + glyph.descender;

            match current.as_mut() {
                None => {
                    current_baseline = baseline;
                    current = Some(
                        Rectangle::new(
                            Point::new(glyph.x, ink_top),
                            Size::new(glyph.w, ink_bottom - ink_top),
                        ) * scale,
                    );
                }
                Some(current) if baseline != current_baseline => {
                    bounds.push(*current);
                    current_baseline = baseline;
                    *current = Rectangle::new(
                        Point::new(glyph.x, ink_top),
                        Size::new(glyph.w, ink_bottom - ink_top),
                    ) * scale;
                }
                Some(current) => {
                    // Union the glyph's ink with the span's bounds on this
                    // line:
                    let left = current.x.min(glyph.x * scale);
                    let top = current.y.min(ink_top * scale);
                    let right = (current.x + current.width).max((glyph.x + glyph.w) * scale);
                    let bottom = (current.y + current.height).max(ink_bottom * scale);
                    *current = Rectangle::new(
                        Point::new(left, top),
                        Size::new(right - left, bottom - top),
                    );
                }
            }
        }

        bounds.extend(current);
        bounds
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
            ellipsis: Ellipsis::default(),
            align_x: Alignment::Default,
            align_y: alignment::Vertical::Top,
            bounds: Size::ZERO,
            min_bounds: Size::ZERO,
            version: text::Version::default(),
            hint: false,
            hint_factor: 1.0,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::text::Paragraph as _;

    fn rich_paragraph(content: &[Span<'_, ()>]) -> Paragraph {
        let text = Text {
            content,
            bounds: Size::new(1000.0, f32::INFINITY),
            size: Pixels(20.0),
            line_height: LineHeight::Relative(1.5),
            font: Font::default(),
            align_x: Alignment::Default,
            align_y: alignment::Vertical::Top,
            shaping: Shaping::default(),
            wrapping: Wrapping::default(),
            ellipsis: Ellipsis::default(),
            hint_factor: None,
        };

        Paragraph::with_spans(text)
    }

    /// Bounds should cover the ink of the span, without stretching to the
    /// line height.
    #[test]
    fn span_bounds_cover_ink() {
        let paragraph = rich_paragraph(&[Span::new("a"), Span::new("a").size(40.0)]);

        let small = paragraph.span_bounds(0);
        let big = paragraph.span_bounds(1);

        assert_eq!(small.len(), 1);
        assert_eq!(big.len(), 1);

        let buffer = paragraph.buffer();
        let line = &buffer.lines[0].layout_opt().unwrap()[0];
        let line_height = line.line_height(buffer.metrics().line_height);
        let ink_height = line.max_ascent + line.max_descent;
        let centering = (line_height - ink_height) / 2.0;

        // The larger span determines the line's ink:
        assert!((big[0].y - centering).abs() < 1e-2);
        assert!((big[0].height - ink_height).abs() < 1e-2);

        // Both spans share the line's baseline:
        let baseline = centering + line.max_ascent;
        assert!((small[0].y + line.max_ascent / 2.0 - baseline).abs() < 1e-2);
        assert!((big[0].y + line.max_ascent - baseline).abs() < 1e-2);

        // The smaller span is strictly inside the line's ink box...
        assert!(small[0].y > centering);
        assert!(small[0].y + small[0].height < centering + ink_height);
        assert!((small[0].height - ink_height / 2.0).abs() < 1e-2);

        // ...and the ink stays inside the line box:
        if ink_height < line_height {
            assert!(big[0].height < line_height);
        }
    }

    /// Bounds should cover the horizontal extent of the span's glyphs.
    #[test]
    fn span_bounds_cover_width() {
        let paragraph = rich_paragraph(&[Span::new("Hello"), Span::new(", world!")]);

        let first = paragraph.span_bounds(0);
        let second = paragraph.span_bounds(1);

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);

        let buffer = paragraph.buffer();
        let line = &buffer.lines[0].layout_opt().unwrap()[0];

        // The first span starts at the beginning of the line and is a
        // proper prefix of it:
        assert!((first[0].x - 0.0).abs() < 1e-2);
        assert!(first[0].width > 0.0);
        assert!(first[0].width < line.w);

        // The second span starts where the first span ends, and ends at
        // the end of the line:
        assert!((second[0].x - (first[0].x + first[0].width)).abs() < 1e-2);
        assert!((second[0].x + second[0].width - line.w).abs() < 1e-2);
    }

    /// A span on multiple lines produces one bounds per line.
    #[test]
    fn span_bounds_cover_each_line() {
        let paragraph = rich_paragraph(&[Span::new("ab\ncd")]);

        let bounds = paragraph.span_bounds(0);

        assert_eq!(bounds.len(), 2);

        let buffer = paragraph.buffer();
        let line_height = buffer.metrics().line_height;

        for bounds in &bounds {
            assert!((bounds.x - 0.0).abs() < 1e-2);
            assert!(bounds.height > 0.0);
            assert!(bounds.height < line_height);
        }

        // The lines are stacked without overlap, and both lines have the
        // same ink height:
        assert!(bounds[0].y + bounds[0].height <= bounds[1].y + 1e-2);
        assert!((bounds[0].height - bounds[1].height).abs() < 1e-2);
    }
}
