use iced_native::Color;

use crate::widget::canvas::Gradient;

/// The style of a stroke.
#[derive(Debug, Clone, Copy)]
pub struct Stroke<'a> {
    /// The color or gradient of the stroke.
    ///
    /// By default, it is set to [`StrokeStyle::Solid`] `BLACK`.
    pub style: StrokeStyle<'a>,
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

impl<'a> Stroke<'a> {
    /// Sets the color of the [`Stroke`].
    pub fn with_color(self, color: Color) -> Self {
        Stroke {
            style: StrokeStyle::Solid(color),
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

impl<'a> Default for Stroke<'a> {
    fn default() -> Self {
        Stroke {
            style: StrokeStyle::Solid(Color::BLACK),
            width: 1.0,
            line_cap: LineCap::default(),
            line_join: LineJoin::default(),
            line_dash: LineDash::default(),
        }
    }
}

/// The color or gradient of a [`Stroke`].
#[derive(Debug, Clone, Copy)]
pub enum StrokeStyle<'a> {
    /// A solid color
    Solid(Color),
    /// A color gradient
    Gradient(&'a Gradient),
}

/// The shape used at the end of open subpaths when they are stroked.
#[derive(Debug, Clone, Copy)]
pub enum LineCap {
    /// The stroke for each sub-path does not extend beyond its two endpoints.
    Butt,
    /// At the end of each sub-path, the shape representing the stroke will be
    /// extended by a square.
    Square,
    /// At the end of each sub-path, the shape representing the stroke will be
    /// extended by a semicircle.
    Round,
}

impl Default for LineCap {
    fn default() -> LineCap {
        LineCap::Butt
    }
}

impl From<LineCap> for lyon::tessellation::LineCap {
    fn from(line_cap: LineCap) -> lyon::tessellation::LineCap {
        match line_cap {
            LineCap::Butt => lyon::tessellation::LineCap::Butt,
            LineCap::Square => lyon::tessellation::LineCap::Square,
            LineCap::Round => lyon::tessellation::LineCap::Round,
        }
    }
}

/// The shape used at the corners of paths or basic shapes when they are
/// stroked.
#[derive(Debug, Clone, Copy)]
pub enum LineJoin {
    /// A sharp corner.
    Miter,
    /// A round corner.
    Round,
    /// A bevelled corner.
    Bevel,
}

impl Default for LineJoin {
    fn default() -> LineJoin {
        LineJoin::Miter
    }
}

impl From<LineJoin> for lyon::tessellation::LineJoin {
    fn from(line_join: LineJoin) -> lyon::tessellation::LineJoin {
        match line_join {
            LineJoin::Miter => lyon::tessellation::LineJoin::Miter,
            LineJoin::Round => lyon::tessellation::LineJoin::Round,
            LineJoin::Bevel => lyon::tessellation::LineJoin::Bevel,
        }
    }
}

/// The dash pattern used when stroking the line.
#[derive(Debug, Clone, Copy, Default)]
pub struct LineDash<'a> {
    /// The alternating lengths of lines and gaps which describe the pattern.
    pub segments: &'a [f32],

    /// The offset of [`LineDash::segments`] to start the pattern.
    pub offset: usize,
}
