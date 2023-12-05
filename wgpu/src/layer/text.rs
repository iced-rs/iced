use crate::core::alignment;
use crate::core::text;
use crate::core::{Color, Font, Pixels, Point, Rectangle};
use crate::graphics;
use crate::graphics::text::editor;
use crate::graphics::text::paragraph;

/// A text primitive.
#[derive(Debug, Clone)]
pub enum Text<'a> {
    /// A paragraph.
    #[allow(missing_docs)]
    Paragraph {
        paragraph: paragraph::Weak,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    },
    /// An editor.
    #[allow(missing_docs)]
    Editor {
        editor: editor::Weak,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    },
    /// Some cached text.
    Cached(Cached<'a>),
    /// Some raw text.
    Raw(graphics::text::Raw),
}

#[derive(Debug, Clone)]
pub struct Cached<'a> {
    /// The content of the [`Text`].
    pub content: &'a str,

    /// The layout bounds of the [`Text`].
    pub bounds: Rectangle,

    /// The color of the [`Text`], in __linear RGB_.
    pub color: Color,

    /// The size of the [`Text`] in logical pixels.
    pub size: Pixels,

    /// The line height of the [`Text`].
    pub line_height: text::LineHeight,

    /// The font of the [`Text`].
    pub font: Font,

    /// The horizontal alignment of the [`Text`].
    pub horizontal_alignment: alignment::Horizontal,

    /// The vertical alignment of the [`Text`].
    pub vertical_alignment: alignment::Vertical,

    /// The shaping strategy of the text.
    pub shaping: text::Shaping,

    /// The clip bounds of the text.
    pub clip_bounds: Rectangle,
}
