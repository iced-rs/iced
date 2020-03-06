use crate::{
    canvas::{Drawable, Frame, Layer},
    Primitive,
};

use iced_native::{Point, Size};
use std::{cell::RefCell, marker::PhantomData};

/// A simple cache that stores generated geometry to avoid recomputation.
///
/// A [`Cache`] will not redraw its geometry unless the dimensions of its layer
/// change or it is explicitly cleared.
///
/// [`Layer`]: ../trait.Layer.html
/// [`Cached`]: struct.Cached.html
#[derive(Debug)]
pub struct Cache<T: Drawable> {
    input: PhantomData<T>,
    state: RefCell<State>,
}

#[derive(Debug)]
enum State {
    Empty,
    Filled { bounds: Size, primitive: Primitive },
}

impl<T> Cache<T>
where
    T: Drawable + std::fmt::Debug,
{
    /// Creates a new empty [`Cache`].
    ///
    /// [`Cache`]: struct.Cache.html
    pub fn new() -> Self {
        Cache {
            input: PhantomData,
            state: RefCell::new(State::Empty),
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
    pub fn with<'a>(&'a self, input: &'a T) -> impl Layer + 'a {
        Bind {
            cache: self,
            input: input,
        }
    }
}

#[derive(Debug)]
struct Bind<'a, T: Drawable> {
    cache: &'a Cache<T>,
    input: &'a T,
}

impl<'a, T> Layer for Bind<'a, T>
where
    T: Drawable + std::fmt::Debug,
{
    fn draw(&self, origin: Point, current_bounds: Size) -> Primitive {
        use std::ops::Deref;

        if let State::Filled { bounds, primitive } =
            self.cache.state.borrow().deref()
        {
            if *bounds == current_bounds {
                return primitive.clone();
            }
        }

        let mut frame = Frame::new(current_bounds.width, current_bounds.height);
        self.input.draw(&mut frame);

        let primitive = frame.into_primitive(origin);

        *self.cache.state.borrow_mut() = State::Filled {
            bounds: current_bounds,
            primitive: primitive.clone(),
        };

        primitive
    }
}
