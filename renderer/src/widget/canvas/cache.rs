use crate::widget::canvas::{Frame, Geometry};
use crate::{Primitive, Renderer, Size};

use std::cell::RefCell;
use std::sync::Arc;

/// A simple cache that stores generated [`Geometry`] to avoid recomputation.
///
/// A [`Cache`] will not redraw its geometry unless the dimensions of its layer
/// change or it is explicitly cleared.
#[derive(Debug, Default)]
pub struct Cache {
    state: RefCell<State>,
}

#[derive(Debug, Default)]
enum State {
    #[default]
    Empty,
    Filled {
        bounds: Size,
        primitive: Arc<Primitive>,
    },
}

impl Cache {
    /// Creates a new empty [`Cache`].
    pub fn new() -> Self {
        Cache {
            state: Default::default(),
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
    pub fn draw<Theme>(
        &self,
        renderer: &Renderer<Theme>,
        bounds: Size,
        draw_fn: impl FnOnce(&mut Frame),
    ) -> Geometry {
        use std::ops::Deref;

        if let State::Filled {
            bounds: cached_bounds,
            primitive,
        } = self.state.borrow().deref()
        {
            if *cached_bounds == bounds {
                return Geometry(Primitive::Cache {
                    content: primitive.clone(),
                });
            }
        }

        let mut frame = Frame::new(renderer, bounds);
        draw_fn(&mut frame);

        let primitive = {
            let geometry = frame.into_geometry();

            Arc::new(geometry.0)
        };

        *self.state.borrow_mut() = State::Filled {
            bounds,
            primitive: primitive.clone(),
        };

        Geometry(Primitive::Cache { content: primitive })
    }
}
