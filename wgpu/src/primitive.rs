//! Draw custom primitives.
use crate::core::{self, Rectangle};
use crate::graphics::Viewport;

use rustc_hash::FxHashMap;
use std::any::{Any, TypeId};
use std::fmt::Debug;

/// A batch of primitives.
pub type Batch = Vec<Instance>;

/// A set of methods which allows a [`Primitive`] to be rendered.
pub trait Primitive: Debug + Send + Sync + 'static {
    /// Processes the [`Primitive`], allowing for GPU buffer allocation.
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        storage: &mut Storage,
        bounds: &Rectangle,
        viewport: &Viewport,
    );

    /// Renders the [`Primitive`].
    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        storage: &Storage,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    );
}

#[derive(Debug)]
/// An instance of a specific [`Primitive`].
pub struct Instance {
    /// The bounds of the [`Instance`].
    pub bounds: Rectangle,

    /// The [`Primitive`] to render.
    pub primitive: Box<dyn Primitive>,
}

impl Instance {
    /// Creates a new [`Instance`] with the given [`Primitive`].
    pub fn new(bounds: Rectangle, primitive: impl Primitive) -> Self {
        Instance {
            bounds,
            primitive: Box::new(primitive),
        }
    }
}

/// A renderer than can draw custom primitives.
pub trait Renderer: core::Renderer {
    /// Draws a custom primitive.
    fn draw_primitive(&mut self, bounds: Rectangle, primitive: impl Primitive);
}

/// Stores custom, user-provided types.
#[derive(Default, Debug)]
pub struct Storage {
    pipelines: FxHashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Storage {
    /// Returns `true` if `Storage` contains a type `T`.
    pub fn has<T: 'static>(&self) -> bool {
        self.pipelines.contains_key(&TypeId::of::<T>())
    }

    /// Inserts the data `T` in to [`Storage`].
    pub fn store<T: 'static + Send + Sync>(&mut self, data: T) {
        let _ = self.pipelines.insert(TypeId::of::<T>(), Box::new(data));
    }

    /// Returns a reference to the data with type `T` if it exists in [`Storage`].
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.pipelines.get(&TypeId::of::<T>()).map(|pipeline| {
            pipeline
                .downcast_ref::<T>()
                .expect("Value with this type does not exist in Storage.")
        })
    }

    /// Returns a mutable reference to the data with type `T` if it exists in [`Storage`].
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.pipelines.get_mut(&TypeId::of::<T>()).map(|pipeline| {
            pipeline
                .downcast_mut::<T>()
                .expect("Value with this type does not exist in Storage.")
        })
    }
}
