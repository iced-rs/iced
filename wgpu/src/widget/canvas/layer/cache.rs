use crate::{
    canvas::{Drawable, Frame, Layer},
    Primitive,
};

use iced_native::Size;
use std::{borrow::Borrow, cell::RefCell, marker::PhantomData, sync::Arc};

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

    /// Binds the [`Cache`] with some data, producing a [`Layer`] that can be
    /// added to a [`Canvas`].
    ///
    /// [`Cache`]: struct.Cache.html
    /// [`Layer`]: ../trait.Layer.html
    /// [`Canvas`]: ../../struct.Canvas.html
    pub fn with<'a, T>(
        &'a self,
        input: impl Borrow<T> + std::fmt::Debug + 'a,
    ) -> impl Layer + 'a
    where
        T: Drawable + std::fmt::Debug + 'a,
    {
        Bind {
            cache: self,
            input: input,
            drawable: PhantomData,
        }
    }
}

#[derive(Debug)]
struct Bind<'a, T: Drawable, I: Borrow<T> + 'a> {
    cache: &'a Cache,
    input: I,
    drawable: PhantomData<T>,
}

impl<'a, T, I> Layer for Bind<'a, T, I>
where
    T: Drawable + std::fmt::Debug,
    I: Borrow<T> + std::fmt::Debug + 'a,
{
    fn draw(&self, current_bounds: Size) -> Arc<Primitive> {
        use std::ops::Deref;

        if let State::Filled { bounds, primitive } =
            self.cache.state.borrow().deref()
        {
            if *bounds == current_bounds {
                return primitive.clone();
            }
        }

        let mut frame = Frame::new(current_bounds.width, current_bounds.height);
        self.input.borrow().draw(&mut frame);

        let primitive = Arc::new(frame.into_primitive());

        *self.cache.state.borrow_mut() = State::Filled {
            bounds: current_bounds,
            primitive: primitive.clone(),
        };

        primitive
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
