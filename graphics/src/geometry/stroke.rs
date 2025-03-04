//! Create lines from a [`Path`] and assigns them various attributes/styles.
//!
//! [`Path`]: super::Path
pub use crate::geometry::Style;

use iced_core::Color;

/// The style of a stroke.
#[derive(Debug, Clone, Copy)]
pub struct Stroke<'a> {
    /// The color or gradient of the stroke.
    ///
    /// By default, it is set to a [`Style::Solid`] with [`Color::BLACK`].
    pub style: Style,
    /// The distance between the two edges of the stroke.
    pub width: f32,
    /// The shape to be used at the end of open subpaths when they are stroked.
    pub line_cap: LineCap,
    /// The shape to be used at the corners of paths or basic shapes when they
    /// are stroked.
    pub line_join: LineJoin,
    /// The dash pattern used when stroking the line.
    pub line_dash: LineDash<'a>,
}

impl Stroke<'_> {
    /// Sets the color of the [`Stroke`].
    pub fn with_color(self, color: Color) -> Self {
        Stroke {
            style: Style::Solid(color),
            ..self
        }
    }

    /// Sets the width of the [`Stroke`].
    pub fn with_width(self, width: f32) -> Self {
        Stroke { width, ..self }
    }

    /// Sets the [`LineCap`] of the [`Stroke`].
    pub fn with_line_cap(self, line_cap: LineCap) -> Self {
        Stroke { line_cap, ..self }
    }

    /// Sets the [`LineJoin`] of the [`Stroke`].
    pub fn with_line_join(self, line_join: LineJoin) -> Self {
        Stroke { line_join, ..self }
    }
}

impl Default for Stroke<'_> {
    fn default() -> Self {
        Stroke {
            style: Style::Solid(Color::BLACK),
            width: 1.0,
            line_cap: LineCap::default(),
            line_join: LineJoin::default(),
            line_dash: LineDash::default(),
        }
    }
}

/// The shape used at the end of open subpaths when they are stroked.
#[derive(Debug, Clone, Copy, Default)]
pub enum LineCap {
    /// The stroke for each sub-path does not extend beyond its two endpoints.
    #[default]
    Butt,
    /// At the end of each sub-path, the shape representing the stroke will be
    /// extended by a square.
    Square,
    /// At the end of each sub-path, the shape representing the stroke will be
    /// extended by a semicircle.
    Round,
}

/// The shape used at the corners of paths or basic shapes when they are
/// stroked.
#[derive(Debug, Clone, Copy, Default)]
pub enum LineJoin {
    /// A sharp corner.
    #[default]
    Miter,
    /// A round corner.
    Round,
    /// A bevelled corner.
    Bevel,
}

/// The dash pattern used when stroking the line.
#[derive(Debug, Clone, Copy, Default)]
pub struct LineDash<'a> {
    /// The alternating lengths of lines and gaps which describe the pattern.
    pub segments: &'a [f32],

    /// The offset of [`LineDash::segments`] to start the pattern.
    pub offset: usize,
}
