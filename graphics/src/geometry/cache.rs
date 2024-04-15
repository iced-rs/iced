use crate::core::Size;
use crate::geometry::{self, Frame};
use crate::Cached;

use std::cell::RefCell;

/// A simple cache that stores generated geometry to avoid recomputation.
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
            state: RefCell::new(State::Empty { previous: None }),
        }
    }

    /// Clears the [`Cache`], forcing a redraw the next time it is used.
    pub fn clear(&self) {
        use std::ops::Deref;

        let previous = match self.state.borrow().deref() {
            State::Empty { previous } => previous.clone(),
            State::Filled { geometry, .. } => Some(geometry.clone()),
        };

        *self.state.borrow_mut() = State::Empty { previous };
    }

    /// Draws geometry using the provided closure and stores it in the
    /// [`Cache`].
    ///
    /// The closure will only be called when
    /// - the bounds have changed since the previous draw call.
    /// - the [`Cache`] is empty or has been explicitly cleared.
    ///
    /// Otherwise, the previously stored geometry will be returned. The
    /// [`Cache`] is not cleared in this case. In other words, it will keep
    /// returning the stored geometry if needed.
    pub fn draw(
        &self,
        renderer: &Renderer,
        bounds: Size,
        draw_fn: impl FnOnce(&mut Frame<Renderer>),
    ) -> Renderer::Geometry {
        use std::ops::Deref;

        let previous = match self.state.borrow().deref() {
            State::Empty { previous } => previous.clone(),
            State::Filled {
                bounds: cached_bounds,
                geometry,
            } => {
                if *cached_bounds == bounds {
                    return Cached::load(geometry);
                }

                Some(geometry.clone())
            }
        };

        let mut frame = Frame::new(renderer, bounds);
        draw_fn(&mut frame);

        let geometry = frame.into_geometry().cache(previous);
        let result = Cached::load(&geometry);

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
            State::Empty { .. } => write!(f, "Cache::Empty"),
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
    Geometry: Cached,
{
    Empty {
        previous: Option<Geometry::Cache>,
    },
    Filled {
        bounds: Size,
        geometry: Geometry::Cache,
    },
}
