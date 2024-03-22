use crate::core::Size;
use crate::geometry::{self, Frame, Geometry};
use crate::Primitive;

use std::cell::RefCell;
use std::sync::Arc;

/// A simple cache that stores generated [`Geometry`] to avoid recomputation.
///
/// A [`Cache`] will not redraw its geometry unless the dimensions of its layer
/// change or it is explicitly cleared.
pub struct Cache<Renderer>
where
    Renderer: geometry::Renderer,
{
    state: RefCell<State<Renderer::Geometry>>,
}

impl<Renderer> Cache<Renderer>
where
    Renderer: geometry::Renderer,
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
        draw_fn: impl FnOnce(&mut Frame<Renderer>),
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

        let mut frame = Frame::new(renderer, bounds);
        draw_fn(&mut frame);

        let geometry = frame.into_geometry().cache();
        let result = Geometry::load(&geometry);

        *self.state.borrow_mut() = State::Filled { bounds, geometry };

        result
    }
}

impl<Renderer> std::fmt::Debug for Cache<Renderer>
where
    Renderer: geometry::Renderer,
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
    Renderer: geometry::Renderer,
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
