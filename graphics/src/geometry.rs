//! Build and draw geometry.
pub mod fill;
pub mod path;
pub mod stroke;

mod style;
mod text;

pub use fill::Fill;
pub use path::Path;
pub use stroke::{LineCap, LineDash, LineJoin, Stroke};
pub use style::Style;
pub use text::Text;

pub use crate::gradient::{self, Gradient};

use crate::core::{Point, Radians, Rectangle, Size, Vector};
use crate::Primitive;

use std::cell::RefCell;
use std::sync::Arc;

pub fn frame<Renderer>(renderer: &Renderer, size: Size) -> Renderer::Frame
where
    Renderer: self::Renderer,
{
    renderer.new_frame(size)
}

/// A renderer capable of drawing some [`Self::Geometry`].
pub trait Renderer: crate::core::Renderer {
    /// The kind of geometry this renderer can draw.
    type Geometry: Geometry;

    /// The kind of [`Frame`] this renderer supports.
    type Frame: Frame<Geometry = Self::Geometry>;

    fn new_frame(&self, size: Size) -> Self::Frame;

    /// Draws the given [`Self::Geometry`].
    fn draw_geometry(&mut self, geometry: Self::Geometry);
}

pub trait Backend {
    /// The kind of [`Frame`] this backend supports.
    type Frame: Frame;

    fn new_frame(&self, size: Size) -> Self::Frame;
}

pub trait Frame: Sized + Into<Self::Geometry> {
    /// The kind of geometry this frame can draw.
    type Geometry: Geometry;

    /// Returns the width of the [`Frame`].
    fn width(&self) -> f32;

    /// Returns the height of the [`Frame`].
    fn height(&self) -> f32;

    /// Returns the dimensions of the [`Frame`].
    fn size(&self) -> Size;

    /// Returns the coordinate of the center of the [`Frame`].
    fn center(&self) -> Point;

    /// Draws the given [`Path`] on the [`Frame`] by filling it with the
    /// provided style.
    fn fill(&mut self, path: &Path, fill: impl Into<Fill>);

    /// Draws an axis-aligned rectangle given its top-left corner coordinate and
    /// its `Size` on the [`Frame`] by filling it with the provided style.
    fn fill_rectangle(
        &mut self,
        top_left: Point,
        size: Size,
        fill: impl Into<Fill>,
    );

    /// Draws the stroke of the given [`Path`] on the [`Frame`] with the
    /// provided style.
    fn stroke<'a>(&mut self, path: &Path, stroke: impl Into<Stroke<'a>>);

    /// Draws the characters of the given [`Text`] on the [`Frame`], filling
    /// them with the given color.
    ///
    /// __Warning:__ Text currently does not work well with rotations and scale
    /// transforms! The position will be correctly transformed, but the
    /// resulting glyphs will not be rotated or scaled properly.
    ///
    /// Additionally, all text will be rendered on top of all the layers of
    /// a `Canvas`. Therefore, it is currently only meant to be used for
    /// overlays, which is the most common use case.
    ///
    /// Support for vectorial text is planned, and should address all these
    /// limitations.
    fn fill_text(&mut self, text: impl Into<Text>);

    /// Stores the current transform of the [`Frame`] and executes the given
    /// drawing operations, restoring the transform afterwards.
    ///
    /// This method is useful to compose transforms and perform drawing
    /// operations in different coordinate systems.
    #[inline]
    fn with_save<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.push_transform();

        let result = f(self);

        self.pop_transform();

        result
    }

    /// Pushes the current transform in the transform stack.
    fn push_transform(&mut self);

    /// Pops a transform from the transform stack and sets it as the current transform.
    fn pop_transform(&mut self);

    /// Executes the given drawing operations within a [`Rectangle`] region,
    /// clipping any geometry that overflows its bounds. Any transformations
    /// performed are local to the provided closure.
    ///
    /// This method is useful to perform drawing operations that need to be
    /// clipped.
    #[inline]
    fn with_clip<R>(
        &mut self,
        region: Rectangle,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let mut frame = self.draft(region.size());

        let result = f(&mut frame);

        let origin = Point::new(region.x, region.y);

        self.paste(frame, origin);

        result
    }

    /// Creates a new [`Frame`] with the given [`Size`].
    ///
    /// Draw its contents back to this [`Frame`] with [`paste`].
    ///
    /// [`paste`]: Self::paste
    fn draft(&mut self, size: Size) -> Self;

    /// Draws the contents of the given [`Frame`] with origin at the given [`Point`].
    fn paste(&mut self, frame: Self, at: Point);

    /// Applies a translation to the current transform of the [`Frame`].
    fn translate(&mut self, translation: Vector);

    /// Applies a rotation in radians to the current transform of the [`Frame`].
    fn rotate(&mut self, angle: impl Into<Radians>);

    /// Applies a uniform scaling to the current transform of the [`Frame`].
    fn scale(&mut self, scale: impl Into<f32>);

    /// Applies a non-uniform scaling to the current transform of the [`Frame`].
    fn scale_nonuniform(&mut self, scale: impl Into<Vector>);
}

pub trait Geometry: Sized {
    type Cache;

    fn load(cache: &Self::Cache) -> Self;

    fn cache(self) -> Self::Cache;
}

/// A simple cache that stores generated [`Geometry`] to avoid recomputation.
///
/// A [`Cache`] will not redraw its geometry unless the dimensions of its layer
/// change or it is explicitly cleared.
pub struct Cache<Renderer>
where
    Renderer: self::Renderer,
{
    state: RefCell<State<Renderer::Geometry>>,
}

impl<Renderer> Cache<Renderer>
where
    Renderer: self::Renderer,
{
    /// Creates a new empty [`Cache`].
    pub fn new() -> Self {
        Cache {
            state: RefCell::new(State::Empty),
        }
    }

    /// Clears the [`Cache`], forcing a redraw the next time it is used.
    pub fn clear(&self) {
        *self.state.borrow_mut() = State::Empty;
    }

    /// Draws [`Geometry`] using the provided closure and stores it in the
    /// [`Cache`].
    ///
    /// The closure will only be called when
    /// - the bounds have changed since the previous draw call.
    /// - the [`Cache`] is empty or has been explicitly cleared.
    ///
    /// Otherwise, the previously stored [`Geometry`] will be returned. The
    /// [`Cache`] is not cleared in this case. In other words, it will keep
    /// returning the stored [`Geometry`] if needed.
    pub fn draw(
        &self,
        renderer: &Renderer,
        bounds: Size,
        draw_fn: impl FnOnce(&mut Renderer::Frame),
    ) -> Renderer::Geometry {
        use std::ops::Deref;

        if let State::Filled {
            bounds: cached_bounds,
            geometry,
        } = self.state.borrow().deref()
        {
            if *cached_bounds == bounds {
                return Geometry::load(geometry);
            }
        }

        let mut frame = frame(renderer, bounds);
        draw_fn(&mut frame);

        let geometry = frame.into().cache();
        let result = Geometry::load(&geometry);

        *self.state.borrow_mut() = State::Filled { bounds, geometry };

        result
    }
}

impl<Renderer> std::fmt::Debug for Cache<Renderer>
where
    Renderer: self::Renderer,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self.state.borrow();

        match *state {
            State::Empty => write!(f, "Cache::Empty"),
            State::Filled { bounds, .. } => {
                write!(f, "Cache::Filled {{ bounds: {bounds:?} }}")
            }
        }
    }
}

impl<Renderer> Default for Cache<Renderer>
where
    Renderer: self::Renderer,
{
    fn default() -> Self {
        Self::new()
    }
}

enum State<Geometry>
where
    Geometry: self::Geometry,
{
    Empty,
    Filled {
        bounds: Size,
        geometry: Geometry::Cache,
    },
}

impl<T> Geometry for Primitive<T> {
    type Cache = Arc<Self>;

    fn load(cache: &Arc<Self>) -> Self {
        Self::Cache {
            content: cache.clone(),
        }
    }

    fn cache(self) -> Arc<Self> {
        Arc::new(self)
    }
}
