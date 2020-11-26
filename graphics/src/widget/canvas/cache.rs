use crate::{
    canvas::{Frame, Geometry},
    Primitive,
};

use iced_native::Size;
use std::{cell::RefCell, sync::Arc};

enum State {
    Empty,
    Filled {
        bounds: Size,
        primitive: Arc<Primitive>,
    },
}

impl Default for State {
    fn default() -> Self {
        State::Empty
    }
}
/// A simple cache that stores generated [`Geometry`] to avoid recomputation.
///
/// A [`Cache`] will not redraw its geometry unless the dimensions of its layer
/// change or it is explicitly cleared.
#[derive(Debug, Default)]
pub struct Cache {
    state: RefCell<State>,
}

impl Cache {
    /// Creates a new empty [`Cache`].
    pub fn new() -> Self {
        Cache {
            state: Default::default(),
        }
    }

    /// Clears the [`Cache`], forcing a redraw the next time it is used.
    pub fn clear(&mut self) {
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
    pub fn draw(&self, bounds: Size, draw_fn: impl Fn(&mut Frame)) -> Geometry {
        use std::ops::Deref;

        if let State::Filled {
            bounds: cached_bounds,
            primitive,
        } = self.state.borrow().deref()
        {
            if *cached_bounds == bounds {
                return Geometry::from_primitive(Primitive::Cached {
                    cache: primitive.clone(),
                });
            }
        }

        let mut frame = Frame::new(bounds);
        draw_fn(&mut frame);

        let primitive = {
            let geometry = frame.into_geometry();

            Arc::new(geometry.into_primitive())
        };

        *self.state.borrow_mut() = State::Filled {
            bounds,
            primitive: primitive.clone(),
        };

        Geometry::from_primitive(Primitive::Cached { cache: primitive })
    }
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Empty => write!(f, "Empty"),
            State::Filled { primitive, bounds } => f
                .debug_struct("Filled")
                .field("primitive", primitive)
                .field("bounds", bounds)
                .finish(),
        }
    }
}
