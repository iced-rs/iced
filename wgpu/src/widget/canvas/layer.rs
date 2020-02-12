use crate::{canvas::Frame, triangle};

use iced_native::Size;
use std::cell::RefCell;
use std::sync::Arc;

pub trait Layer: std::fmt::Debug {
    fn draw(&self, bounds: Size) -> Arc<triangle::Mesh2D>;
}

use std::marker::PhantomData;

#[derive(Debug)]
pub struct Cached<T: Drawable> {
    input: PhantomData<T>,
    cache: RefCell<Cache>,
}

#[derive(Debug)]
enum Cache {
    Empty,
    Filled {
        mesh: Arc<triangle::Mesh2D>,
        bounds: Size,
    },
}

impl<T> Cached<T>
where
    T: Drawable + std::fmt::Debug,
{
    pub fn new() -> Self {
        Cached {
            input: PhantomData,
            cache: RefCell::new(Cache::Empty),
        }
    }

    pub fn clear(&mut self) {
        *self.cache.borrow_mut() = Cache::Empty;
    }

    pub fn with<'a>(&'a self, input: &'a T) -> impl Layer + 'a {
        Bind {
            layer: self,
            input: input,
        }
    }
}

#[derive(Debug)]
struct Bind<'a, T: Drawable> {
    layer: &'a Cached<T>,
    input: &'a T,
}

impl<'a, T> Layer for Bind<'a, T>
where
    T: Drawable + std::fmt::Debug,
{
    fn draw(&self, current_bounds: Size) -> Arc<triangle::Mesh2D> {
        use std::ops::Deref;

        if let Cache::Filled { mesh, bounds } =
            self.layer.cache.borrow().deref()
        {
            if *bounds == current_bounds {
                return mesh.clone();
            }
        }

        let mut frame = Frame::new(current_bounds.width, current_bounds.height);
        self.input.draw(&mut frame);

        let mesh = Arc::new(frame.into_mesh());

        *self.layer.cache.borrow_mut() = Cache::Filled {
            mesh: mesh.clone(),
            bounds: current_bounds,
        };

        mesh
    }
}

pub trait Drawable {
    fn draw(&self, frame: &mut Frame);
}
