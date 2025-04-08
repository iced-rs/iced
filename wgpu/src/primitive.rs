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

/// Stores custom, user-provided pipelines.
#[derive(Default, Debug)]
pub struct Storage {
    pipelines: FxHashMap<TypeId, Box<dyn AnyPipeline>>,
}

impl Storage {
    /// Returns `true` if `Storage` contains a type `T`.
    pub fn has<T: 'static>(&self) -> bool {
        self.pipelines.contains_key(&TypeId::of::<T>())
    }

    /// Inserts the pipeline `T` in to [`Storage`].
    pub fn store<T: 'static + Pipeline + Send + Sync>(&mut self, data: T) {
        let _ = self.pipelines.insert(TypeId::of::<T>(), Box::new(data));
    }

    /// Returns a reference to the pipeline with type `T` if it exists in [`Storage`].
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.pipelines.get(&TypeId::of::<T>()).map(|pipeline| {
            pipeline
                .as_any()
                .downcast_ref::<T>()
                .expect("Value with this type does not exist in Storage.")
        })
    }

    /// Returns a mutable reference to the pipeline with type `T` if it exists in [`Storage`].
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.pipelines.get_mut(&TypeId::of::<T>()).map(|pipeline| {
            pipeline
                .as_any_mut()
                .downcast_mut::<T>()
                .expect("Value with this type does not exist in Storage.")
        })
    }

    /// Triggers the `end_frame` method on all pipelines that exist within [`Storage`].
    pub(crate) fn end_frame(&mut self) {
        self.pipelines.values_mut().for_each(|pipeline| {
            pipeline.end_frame();
        });
    }
}

/// A pipeline that manages the resources of a shader.
pub trait Pipeline {
    /// Called after each frame. Useful for cleaning up unused resources.
    fn end_frame(&mut self) {}
}

trait AnyPipeline: Pipeline + Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Pipeline + Any + Send + Sync> AnyPipeline for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Debug for dyn AnyPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnyPipeline").finish_non_exhaustive()
    }
}
