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
/// A simple cache that stores generated geometry to avoid recomputation.
///
/// A [`Cache`] will not redraw its geometry unless the dimensions of its layer
/// change or it is explicitly cleared.
///
/// [`Layer`]: ../trait.Layer.html
/// [`Cache`]: struct.Cache.html
#[derive(Debug, Default)]
pub struct Cache {
    state: RefCell<State>,
}

impl Cache {
    /// Creates a new empty [`Cache`].
    ///
    /// [`Cache`]: struct.Cache.html
    pub fn new() -> Self {
        Cache {
            state: Default::default(),
        }
    }

    /// Clears the cache, forcing a redraw the next time it is used.
    ///
    /// [`Cached`]: struct.Cached.html
    pub fn clear(&mut self) {
        *self.state.borrow_mut() = State::Empty;
    }

    pub fn draw(
        &self,
        new_bounds: Size,
        draw_fn: impl Fn(&mut Frame),
    ) -> Geometry {
        use std::ops::Deref;

        if let State::Filled { bounds, primitive } = self.state.borrow().deref()
        {
            if *bounds == new_bounds {
                return Geometry::from_primitive(Primitive::Cached {
                    cache: primitive.clone(),
                });
            }
        }

        let mut frame = Frame::new(new_bounds);
        draw_fn(&mut frame);

        let primitive = {
            let geometry = frame.into_geometry();

            Arc::new(geometry.into_primitive())
        };

        *self.state.borrow_mut() = State::Filled {
            bounds: new_bounds,
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
