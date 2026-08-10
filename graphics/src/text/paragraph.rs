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

// Byte stride of a buffer line in the original content, including its `\n`
// separator. cosmic-text indexes glyphs per line; selection offsets are
// global. The byte-mapping methods below share this to bridge the two.
// Assumes single-byte separators (`\n`), which is what the app produces.
fn buffer_line_byte_len(line: &cosmic_text::BufferLine) -> usize {
    line.text().len() + 1
}

impl core::text::Paragraph for Paragraph {
    type Font = Font;

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
        let internal = self.internal();
        let cursor = internal
            .buffer
            .hit(point.x * self.0.hint_factor, point.y * self.0.hint_factor)?;

        // `index` is relative to the line it landed on; make it global.
        let line_base: usize = internal
            .buffer
            .lines
            .iter()
            .take(cursor.line)
            .map(buffer_line_byte_len)
            .sum();

        Some(Hit::CharOffset(line_base + cursor.index))
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
                    Size::new(glyph.w, glyph.line_height_opt.unwrap_or(line_height)),
                ) * (1.0 / self.0.hint_factor)
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
                    current_bounds.width += glyph.w / self.0.hint_factor;
                }
            }
        }

        bounds.extend(current_bounds);
        bounds
    }

    fn selection_bounds(&self, start: usize, end: usize) -> Vec<Rectangle> {
        if start >= end {
            return Vec::new();
        }

        let internal = self.internal();
        let buffer = &internal.buffer;
        let line_height = buffer.metrics().line_height;
        let scroll_y = buffer.scroll().vertical;

        let mut rects = Vec::new();
        let mut visual_line: i32 = 0;
        let mut line_byte: usize = 0;

        for buffer_line in buffer.lines.iter() {
            let layout = buffer_line
                .layout_opt()
                .map(Vec::as_slice)
                .unwrap_or_default();

            // Bring the global range into this line's local glyph space, so
            // a range on one line can't match the same columns on every line.
            let local_start = start.saturating_sub(line_byte);
            let local_end = end.saturating_sub(line_byte);

            for vline in layout {
                let glyph_start = vline.glyphs.first().map(|g| g.start).unwrap_or(0);
                let glyph_end = vline.glyphs.last().map(|g| g.end).unwrap_or(0);

                let range_start = glyph_start.max(local_start);
                let range_end = glyph_end.min(local_end);

                if range_start < range_end {
                    let (x, width) = if range_start == glyph_start && range_end == glyph_end {
                        (0.0, vline.w)
                    } else {
                        let first_glyph_idx = vline
                            .glyphs
                            .iter()
                            .position(|g| range_start <= g.start)
                            .unwrap_or(0);
                        let mut iter = vline.glyphs.iter();
                        let x: f32 = iter.by_ref().take(first_glyph_idx).map(|g| g.w).sum();
                        let w: f32 = iter.take_while(|g| range_end > g.start).map(|g| g.w).sum();
                        (x, w)
                    };

                    if width > 0.0 {
                        let y = visual_line as f32 * line_height - scroll_y;
                        rects.push(
                            Rectangle {
                                x,
                                y,
                                width,
                                height: line_height,
                            } * (1.0 / self.0.hint_factor),
                        );
                    }
                }

                visual_line += 1;
            }

            line_byte += buffer_line_byte_len(buffer_line);
        }

        rects
    }

    fn byte_position(&self, byte: usize) -> Option<Point> {
        let internal = self.internal();
        let buffer = &internal.buffer;
        let line_height = buffer.metrics().line_height;
        let scroll_y = buffer.scroll().vertical;
        let inv = 1.0 / self.0.hint_factor;

        let mut visual_line: i32 = 0;
        let mut line_byte: usize = 0;
        let mut last_line_end: Option<(f32, i32)> = None;

        for buffer_line in buffer.lines.iter() {
            let layout = buffer_line
                .layout_opt()
                .map(Vec::as_slice)
                .unwrap_or_default();

            // Glyph offsets are line-local; bring the global byte into them.
            let local = byte.saturating_sub(line_byte);

            for vline in layout {
                for glyph in &vline.glyphs {
                    if local < glyph.start {
                        let y = visual_line as f32 * line_height - scroll_y;
                        return Some(Point::new(glyph.x * inv, y * inv));
                    }
                    if local >= glyph.start && local <= glyph.end {
                        let span = glyph.end.saturating_sub(glyph.start);
                        let frac = if span == 0 {
                            0.0
                        } else {
                            (local - glyph.start) as f32 / span as f32
                        };
                        let x = glyph.x + glyph.w * frac;
                        let y = visual_line as f32 * line_height - scroll_y;
                        return Some(Point::new(x * inv, y * inv));
                    }
                }
                if let Some(last) = vline.glyphs.last() {
                    last_line_end = Some((last.x + last.w, visual_line));
                }
                visual_line += 1;
            }

            line_byte += buffer_line_byte_len(buffer_line);
        }

        last_line_end.map(|(x, line)| {
            let y = line as f32 * line_height - scroll_y;
            Point::new(x * inv, y * inv)
        })
    }

    fn visual_line_height(&self) -> Option<f32> {
        Some(self.internal().buffer.metrics().line_height / self.0.hint_factor)
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
                    last_grapheme_count = run.text[glyph.start..glyph.end].graphemes(false).count();
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
            (glyph.x + glyph.x_offset * glyph.font_size + advance) / self.0.hint_factor,
            (glyph.y - glyph.y_offset * glyph.font_size) / self.0.hint_factor,
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

    fn plain(content: &str, width: f32) -> Paragraph {
        Paragraph::with_text(Text {
            content,
            bounds: Size::new(width, 1000.0),
            size: Pixels(16.0),
            line_height: LineHeight::default(),
            font: Font::default(),
            align_x: Alignment::default(),
            align_y: alignment::Vertical::Top,
            shaping: Shaping::Advanced,
            wrapping: Wrapping::default(),
            ellipsis: Ellipsis::default(),
            hint_factor: None,
        })
    }

    // Selecting one `\n`-separated line must not paint the same columns
    // on the others. This is the per-line vs global offset regression.
    #[test]
    fn selection_stays_on_its_own_line() {
        let para = plain("AAA\nBBB\nCCC", 1000.0);

        let y0 = para.byte_position(0).unwrap().y;
        let y1 = para.byte_position(4).unwrap().y;
        let y2 = para.byte_position(8).unwrap().y;
        assert!(y0 < y1 && y1 < y2, "lines should stack vertically");

        // "BBB" is bytes 4..7. Every rect must sit on line 1.
        let rects = para.selection_bounds(4, 7);
        assert!(!rects.is_empty(), "middle line should be highlighted");
        for r in &rects {
            assert!(
                (r.y - y1).abs() < (y1 - y0) / 2.0,
                "rect at y={} leaked off line 1 (y={y1})",
                r.y,
            );
        }
    }

    // A full selection spans all three lines, not just the first.
    #[test]
    fn full_selection_spans_every_line() {
        let content = "AAA\nBBB\nCCC";
        let para = plain(content, 1000.0);

        let rects = para.selection_bounds(0, content.len());
        let mut ys: Vec<f32> = rects.iter().map(|r| r.y).collect();
        ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
        ys.dedup_by(|a, b| (*a - *b).abs() < 0.5);
        assert_eq!(ys.len(), 3, "selection should cover all three lines");
    }

    // Geometry round-trips: a byte on a later line maps back to itself,
    // not to the same column on line 0 (off by at most a glyph edge).
    #[test]
    fn hit_test_round_trips_across_lines() {
        let para = plain("AAA\nBBB\nCCC", 1000.0);
        for byte in [5usize, 9] {
            let p = para.byte_position(byte).unwrap();
            let hit = para.hit_test(p).unwrap().cursor() as i64;
            assert!(
                (hit - byte as i64).abs() <= 1,
                "byte {byte} round-tripped to {hit}",
            );
        }
    }

    // Triple-click / line edges follow the logical line through a wrap.
    #[test]
    fn line_edges_are_logical_not_visual() {
        use crate::core::widget::operation::Selectable;

        struct Probe(String);
        impl Selectable for Probe {
            fn selection(&self) -> Option<(usize, usize)> {
                None
            }
            fn set_selection(&mut self, _: Option<(usize, usize)>) {}
            fn text(&self) -> &str {
                &self.0
            }
            fn byte_position(&self, _: usize) -> Option<Point> {
                None
            }
            fn hit_test(&self, _: Point) -> Option<usize> {
                None
            }
            fn visual_line_height(&self) -> Option<f32> {
                None
            }
            fn min_bounds_height(&self) -> f32 {
                0.0
            }
            fn set_externally_managed(&mut self, _: bool) {}
        }

        // One long logical line, no `\n`: both edges reach the ends
        // regardless of where it would wrap visually.
        let probe = Probe("a really long single logical line".into());
        let len = probe.text().len();
        assert_eq!(probe.line_edge_byte(10, -1), Some(0));
        assert_eq!(probe.line_edge_byte(10, 1), Some(len));

        // Middle line of a multi-line string stops at its own `\n`s.
        let probe = Probe("AAA\nBBB\nCCC".into());
        assert_eq!(probe.line_edge_byte(5, -1), Some(4));
        assert_eq!(probe.line_edge_byte(5, 1), Some(7));
    }
}
