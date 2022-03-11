//! Write some text for your users to read.
use unicode_segmentation::UnicodeSegmentation;

use crate::alignment;
use crate::layout;
use crate::renderer;
use crate::text;
use crate::{Color, Element, Layout, Length, Point, Rectangle, Size, Widget};

use std::cmp::Ordering;

/// The background color for part of a [`Text`].
#[derive(Clone, Debug)]
pub struct Highlight {
    /// The starting grapheme index of the [`Highlight`].
    pub start: usize,
    /// The ending grapheme index of the [`Highlight`].
    pub end: usize,
    /// The color of the [`Highlight`].
    pub color: Color,
}

/// A paragraph of text.
///
/// # Example
///
/// ```
/// # type Text = iced_native::widget::Text<iced_native::renderer::Null>;
/// #
/// Text::new("I <3 iced!")
///     .color([0.0, 0.0, 1.0])
///     .size(40);
/// ```
///
/// ![Text drawn by `iced_wgpu`](https://github.com/iced-rs/iced/blob/7760618fb112074bc40b148944521f312152012a/docs/images/text.png?raw=true)
#[derive(Debug)]
pub struct Text<Renderer: text::Renderer> {
    content: String,
    size: Option<u16>,
    color: Option<Color>,
    highlights: Vec<Highlight>,
    font: Renderer::Font,
    width: Length,
    height: Length,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
}

impl<Renderer: text::Renderer> Text<Renderer> {
    /// Create a new fragment of [`Text`] with the given contents.
    pub fn new<T: Into<String>>(label: T) -> Self {
        Text {
            content: label.into(),
            size: None,
            color: None,
            highlights: Default::default(),
            font: Default::default(),
            width: Length::Shrink,
            height: Length::Shrink,
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
        }
    }

    /// Sets the size of the [`Text`].
    pub fn size(mut self, size: u16) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets the [`Color`] of the [`Text`].
    pub fn color<C: Into<Color>>(mut self, color: C) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Sets the background [`Color`] of the [`Text`] between the given grapheme indexes.
    ///
    /// Can be called multiple times to highlight multiple parts of the text.
    pub fn highlight(mut self, start: usize, end: usize, color: Color) -> Self {
        self.highlights.push(Highlight { start, end, color });
        self
    }

    /// Sets the [`Font`] of the [`Text`].
    ///
    /// [`Font`]: Renderer::Font
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = font.into();
        self
    }

    /// Sets the width of the [`Text`] boundaries.
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Text`] boundaries.
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the [`HorizontalAlignment`] of the [`Text`].
    pub fn horizontal_alignment(
        mut self,
        alignment: alignment::Horizontal,
    ) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    /// Sets the [`VerticalAlignment`] of the [`Text`].
    pub fn vertical_alignment(
        mut self,
        alignment: alignment::Vertical,
    ) -> Self {
        self.vertical_alignment = alignment;
        self
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for Text<Renderer>
where
    Renderer: text::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);

        let size = self.size.unwrap_or(renderer.default_size());

        let bounds = limits.max();

        let (width, height) =
            renderer.measure(&self.content, size, self.font.clone(), bounds);

        let size = limits.resolve(Size::new(width, height));

        layout::Node::new(size)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        style: &renderer::Style,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        draw(
            renderer,
            style,
            layout,
            &self.content,
            self.font.clone(),
            self.size,
            self.color,
            &self.highlights,
            self.horizontal_alignment,
            self.vertical_alignment,
        );
    }
}

/// Draws text using the same logic as the [`Text`] widget.
///
/// Specifically:
///
/// * If no `size` is provided, the default text size of the `Renderer` will be
///   used.
/// * If no `color` is provided, the [`renderer::Style::text_color`] will be
///   used.
/// * The alignment attributes do not affect the position of the bounds of the
///   [`Layout`].
pub fn draw<Renderer>(
    renderer: &mut Renderer,
    style: &renderer::Style,
    layout: Layout<'_>,
    content: &str,
    font: Renderer::Font,
    size: Option<u16>,
    color: Option<Color>,
    highlights: &[Highlight],
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
) where
    Renderer: text::Renderer,
{
    let bounds = layout.bounds();

    let x = match horizontal_alignment {
        alignment::Horizontal::Left => bounds.x,
        alignment::Horizontal::Center => bounds.center_x(),
        alignment::Horizontal::Right => bounds.x + bounds.width,
    };

    let y = match vertical_alignment {
        alignment::Vertical::Top => bounds.y,
        alignment::Vertical::Center => bounds.center_y(),
        alignment::Vertical::Bottom => bounds.y + bounds.height,
    };

    let size = size.unwrap_or(renderer.default_size());

    // Cache byte offsets up to the highest accessed index
    let mut byte_offsets = Vec::new();
    let mut grapheme_indices = content.grapheme_indices(true).map(|(i, _)| i);
    let mut get_byte_offset = |grapheme_index| {
        byte_offsets.get(grapheme_index).copied().or_else(|| {
            byte_offsets.extend(
                grapheme_indices
                    .by_ref()
                    .take((grapheme_index - byte_offsets.len()) + 1),
            );
            byte_offsets.get(grapheme_index).copied()
        })
    };

    for &Highlight { start, end, color } in highlights {
        let start_index = if let Some(index) = get_byte_offset(start.min(end)) {
            index
        } else {
            continue;
        };

        let end_index = if let Some(index) = get_byte_offset(start.max(end)) {
            index
        } else {
            continue;
        };

        // The content prior to the start of the highlight is relevant for calculating offsets:
        // The total height of all the lines of text above the line with the highlight,
        // and the width of all the text in the line with the highlight, up until the start of the highlight.
        let before_start = &content[..start_index];

        // Iced's text layouting treats standalone carriage returns (\r) as line endings, but `str::lines`
        // does not, so we must manually search for the relevant line ending* instead.
        let r = before_start.rfind('\r');
        let n = before_start.rfind('\n');
        let before_start_linebreak = r
            .zip(n)
            // If `zip` returns `Some(_)`, there may be multiple line endings
            .map(|(r, n)| match (r + 1).cmp(&n) {
                // The rightmost line ending is `\n`
                Ordering::Less => (n, n),
                // The rightmost line ending is `\r\n`
                Ordering::Equal => (r, n),
                // The rightmost line ending is `\r`
                Ordering::Greater => (r, r),
            })
            // If `zip` returns `None`, there is either 1 or 0 line endings - if 1, `xor` returns `Some(_)`
            .or_else(|| r.xor(n).map(|i| (i, i)));

        // *Get the text preceding and following the rightmost line ending
        let (above_lines, before_start_line) = match before_start_linebreak {
            Some((l, r)) => (&before_start[..l], &before_start[(r + 1)..]),
            None => (Default::default(), before_start),
        };

        // Measure height of lines up until the line that contains the start of the highlight
        let (_, mut height_offset) =
            renderer.measure(above_lines, size, font.clone(), Size::INFINITY);

        // If the highlight crosses over multiple lines, draw a seperate rect on each line
        // BUG: This ignores single `\r` but Iced's text layouting does not (See above)
        // BUG #2: Text wrapping caused by the text not being given wide enough bounds is not handled at all
        //         (And furthermore it currently _can't_ be handled because there's no way to get information about it)
        let mut lines = content[start_index..end_index].lines();

        // Unroll the first iteration of the loop as only the first line needs this offset
        let first_line_offset =
            renderer.measure_width(before_start_line, size, font.clone());

        let (width, height) = renderer.measure(
            lines.next().unwrap_or_default(),
            size,
            font.clone(),
            Size::INFINITY,
        );

        let quad = renderer::Quad {
            bounds: Rectangle {
                x: x + first_line_offset,
                y: y + height_offset,
                width,
                height,
            },
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        };

        renderer.fill_quad(quad, color);

        height_offset += height;

        for line in lines {
            let (width, height) =
                renderer.measure(line, size, font.clone(), Size::INFINITY);

            let quad = renderer::Quad {
                bounds: Rectangle {
                    x,
                    y: y + height_offset,
                    width,
                    height,
                },
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            };

            renderer.fill_quad(quad, color);

            height_offset += height;
        }
    }

    renderer.fill_text(crate::text::Text {
        content,
        size: f32::from(size),
        bounds: Rectangle { x, y, ..bounds },
        color: color.unwrap_or(style.text_color),
        font,
        horizontal_alignment,
        vertical_alignment,
    });
}

impl<'a, Message, Renderer> From<Text<Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: text::Renderer + 'a,
{
    fn from(text: Text<Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(text)
    }
}

impl<Renderer: text::Renderer> Clone for Text<Renderer> {
    fn clone(&self) -> Self {
        Self {
            content: self.content.clone(),
            size: self.size,
            color: self.color,
            highlights: self.highlights.clone(),
            font: self.font.clone(),
            width: self.width,
            height: self.height,
            horizontal_alignment: self.horizontal_alignment,
            vertical_alignment: self.vertical_alignment,
        }
    }
}
