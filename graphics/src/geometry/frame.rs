//! Draw and generate geometry.
use crate::core::{Point, Radians, Rectangle, Size, Vector};
use crate::geometry::{self, Fill, Image, Path, Stroke, Svg, Text};

/// The region of a surface that can be used to draw geometry.
#[allow(missing_debug_implementations)]
pub struct Frame<Renderer>
where
    Renderer: geometry::Renderer,
{
    raw: Renderer::Frame,
}

impl<Renderer> Frame<Renderer>
where
    Renderer: geometry::Renderer,
{
    /// Creates a new [`Frame`] with the given dimensions.
    pub fn new(renderer: &Renderer, size: Size) -> Self {
        Self {
            raw: renderer.new_frame(size),
        }
    }

    /// Returns the width of the [`Frame`].
    pub fn width(&self) -> f32 {
        self.raw.width()
    }

    /// Returns the height of the [`Frame`].
    pub fn height(&self) -> f32 {
        self.raw.height()
    }

    /// Returns the dimensions of the [`Frame`].
    pub fn size(&self) -> Size {
        self.raw.size()
    }

    /// Returns the coordinate of the center of the [`Frame`].
    pub fn center(&self) -> Point {
        self.raw.center()
    }

    /// Draws the given [`Path`] on the [`Frame`] by filling it with the
    /// provided style.
    pub fn fill(&mut self, path: &Path, fill: impl Into<Fill>) {
        self.raw.fill(path, fill);
    }

    /// Draws an axis-aligned rectangle given its top-left corner coordinate and
    /// its `Size` on the [`Frame`] by filling it with the provided style.
    pub fn fill_rectangle(
        &mut self,
        top_left: Point,
        size: Size,
        fill: impl Into<Fill>,
    ) {
        self.raw.fill_rectangle(top_left, size, fill);
    }

    /// Draws the stroke of the given [`Path`] on the [`Frame`] with the
    /// provided style.
    pub fn stroke<'a>(&mut self, path: &Path, stroke: impl Into<Stroke<'a>>) {
        self.raw.stroke(path, stroke);
    }

    /// Draws the stroke of an axis-aligned rectangle with the provided style
    /// given its top-left corner coordinate and its `Size` on the [`Frame`] .
    pub fn stroke_rectangle<'a>(
        &mut self,
        top_left: Point,
        size: Size,
        stroke: impl Into<Stroke<'a>>,
    ) {
        self.raw.stroke_rectangle(top_left, size, stroke);
    }

    /// Draws the characters of the given [`Text`] on the [`Frame`], filling
    /// them with the given color.
    ///
    /// __Warning:__ All text will be rendered on top of all the layers of
    /// a `Canvas`. Therefore, it is currently only meant to be used for
    /// overlays, which is the most common use case.
    pub fn fill_text(&mut self, text: impl Into<Text>) {
        self.raw.fill_text(text);
    }

    /// Draws the given [`Image`] on the [`Frame`] inside the given bounds.
    #[cfg(feature = "image")]
    pub fn draw_image(&mut self, bounds: Rectangle, image: impl Into<Image>) {
        self.raw.draw_image(bounds, image);
    }

    /// Draws the given [`Svg`] on the [`Frame`] inside the given bounds.
    #[cfg(feature = "svg")]
    pub fn draw_svg(&mut self, bounds: Rectangle, svg: impl Into<Svg>) {
        self.raw.draw_svg(bounds, svg);
    }

    /// Stores the current transform of the [`Frame`] and executes the given
    /// drawing operations, restoring the transform afterwards.
    ///
    /// This method is useful to compose transforms and perform drawing
    /// operations in different coordinate systems.
    #[inline]
    pub fn with_save<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.push_transform();

        let result = f(self);

        self.pop_transform();

        result
    }

    /// Pushes the current transform in the transform stack.
    pub fn push_transform(&mut self) {
        self.raw.push_transform();
    }

    /// Pops a transform from the transform stack and sets it as the current transform.
    pub fn pop_transform(&mut self) {
        self.raw.pop_transform();
    }

    /// Executes the given drawing operations within a [`Rectangle`] region,
    /// clipping any geometry that overflows its bounds. Any transformations
    /// performed are local to the provided closure.
    ///
    /// This method is useful to perform drawing operations that need to be
    /// clipped.
    #[inline]
    pub fn with_clip<R>(
        &mut self,
        region: Rectangle,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let mut frame = self.draft(region);

        let result = f(&mut frame);
        self.paste(frame);

        result
    }

    /// Creates a new [`Frame`] with the given [`Size`].
    ///
    /// Draw its contents back to this [`Frame`] with [`paste`].
    ///
    /// [`paste`]: Self::paste
    fn draft(&mut self, clip_bounds: Rectangle) -> Self {
        Self {
            raw: self.raw.draft(clip_bounds),
        }
    }

    /// Draws the contents of the given [`Frame`] with origin at the given [`Point`].
    fn paste(&mut self, frame: Self) {
        self.raw.paste(frame.raw);
    }

    /// Applies a translation to the current transform of the [`Frame`].
    pub fn translate(&mut self, translation: Vector) {
        self.raw.translate(translation);
    }

    /// Applies a rotation in radians to the current transform of the [`Frame`].
    pub fn rotate(&mut self, angle: impl Into<Radians>) {
        self.raw.rotate(angle);
    }

    /// Applies a uniform scaling to the current transform of the [`Frame`].
    pub fn scale(&mut self, scale: impl Into<f32>) {
        self.raw.scale(scale);
    }

    /// Applies a non-uniform scaling to the current transform of the [`Frame`].
    pub fn scale_nonuniform(&mut self, scale: impl Into<Vector>) {
        self.raw.scale_nonuniform(scale);
    }

    /// Turns the [`Frame`] into its underlying geometry.
    pub fn into_geometry(self) -> Renderer::Geometry {
        self.raw.into_geometry()
    }
}

/// The internal implementation of a [`Frame`].
///
/// Analogous to [`Frame`]. See [`Frame`] for the documentation
/// of each method.
#[allow(missing_docs)]
pub trait Backend: Sized {
    type Geometry;

    fn width(&self) -> f32;
    fn height(&self) -> f32;
    fn size(&self) -> Size;
    fn center(&self) -> Point;

    fn push_transform(&mut self);
    fn pop_transform(&mut self);

    fn translate(&mut self, translation: Vector);
    fn rotate(&mut self, angle: impl Into<Radians>);
    fn scale(&mut self, scale: impl Into<f32>);
    fn scale_nonuniform(&mut self, scale: impl Into<Vector>);

    fn draft(&mut self, clip_bounds: Rectangle) -> Self;
    fn paste(&mut self, frame: Self);

    fn stroke<'a>(&mut self, path: &Path, stroke: impl Into<Stroke<'a>>);
    fn stroke_rectangle<'a>(
        &mut self,
        top_left: Point,
        size: Size,
        stroke: impl Into<Stroke<'a>>,
    );
    fn stroke_text<'a>(
        &mut self,
        text: impl Into<Text>,
        stroke: impl Into<Stroke<'a>>,
    );

    fn fill(&mut self, path: &Path, fill: impl Into<Fill>);
    fn fill_text(&mut self, text: impl Into<Text>);
    fn fill_rectangle(
        &mut self,
        top_left: Point,
        size: Size,
        fill: impl Into<Fill>,
    );

    fn draw_image(&mut self, bounds: Rectangle, image: impl Into<Image>);
    fn draw_svg(&mut self, bounds: Rectangle, svg: impl Into<Svg>);

    fn into_geometry(self) -> Self::Geometry;
}

#[cfg(debug_assertions)]
impl Backend for () {
    type Geometry = ();

    fn width(&self) -> f32 {
        0.0
    }

    fn height(&self) -> f32 {
        0.0
    }

    fn size(&self) -> Size {
        Size::ZERO
    }

    fn center(&self) -> Point {
        Point::ORIGIN
    }

    fn push_transform(&mut self) {}
    fn pop_transform(&mut self) {}

    fn translate(&mut self, _translation: Vector) {}
    fn rotate(&mut self, _angle: impl Into<Radians>) {}
    fn scale(&mut self, _scale: impl Into<f32>) {}
    fn scale_nonuniform(&mut self, _scale: impl Into<Vector>) {}

    fn draft(&mut self, _clip_bounds: Rectangle) -> Self {}
    fn paste(&mut self, _frame: Self) {}

    fn stroke<'a>(&mut self, _path: &Path, _stroke: impl Into<Stroke<'a>>) {}
    fn stroke_rectangle<'a>(
        &mut self,
        _top_left: Point,
        _size: Size,
        _stroke: impl Into<Stroke<'a>>,
    ) {
    }
    fn stroke_text<'a>(
        &mut self,
        _text: impl Into<Text>,
        _stroke: impl Into<Stroke<'a>>,
    ) {
    }

    fn fill(&mut self, _path: &Path, _fill: impl Into<Fill>) {}
    fn fill_text(&mut self, _text: impl Into<Text>) {}
    fn fill_rectangle(
        &mut self,
        _top_left: Point,
        _size: Size,
        _fill: impl Into<Fill>,
    ) {
    }

    fn draw_image(&mut self, _bounds: Rectangle, _image: impl Into<Image>) {}
    fn draw_svg(&mut self, _bounds: Rectangle, _svg: impl Into<Svg>) {}

    fn into_geometry(self) -> Self::Geometry {}
}
