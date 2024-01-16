//! Draw primitives using custom pipelines.
use crate::core::{Rectangle, Size};

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Clone, Debug)]
/// A custom primitive which can be used to render primitives associated with a custom pipeline.
pub struct Pipeline {
    /// The bounds of the [`Pipeline`].
    pub bounds: Rectangle,

    /// The [`Primitive`] to render.
    pub primitive: Arc<dyn Primitive>,
}

impl Pipeline {
    /// Creates a new [`Pipeline`] with the given [`Primitive`].
    pub fn new(bounds: Rectangle, primitive: impl Primitive) -> Self {
        Pipeline {
            bounds,
            primitive: Arc::new(primitive),
        }
    }
}

impl PartialEq for Pipeline {
    fn eq(&self, other: &Self) -> bool {
        self.primitive.type_id() == other.primitive.type_id()
    }
}

/// A set of methods which allows a [`Primitive`] to be rendered.
pub trait Primitive: Debug + Send + Sync + 'static {
    /// Processes the [`Primitive`], allowing for GPU buffer allocation.
    fn prepare(
        &self,
        format: wgpu::TextureFormat,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: Rectangle,
        target_size: Size<u32>,
        scale_factor: f32,
        storage: &mut Storage,
    );

    /// Renders the [`Primitive`].
    fn render(
        &self,
        storage: &Storage,
        target: &wgpu::TextureView,
        target_size: Size<u32>,
        viewport: Rectangle<u32>,
        encoder: &mut wgpu::CommandEncoder,
    );
}

/// A renderer than can draw custom pipeline primitives.
pub trait Renderer: crate::core::Renderer {
    /// Draws a custom pipeline primitive.
    fn draw_pipeline_primitive(
        &mut self,
        bounds: Rectangle,
        primitive: impl Primitive,
    );
}

impl<Theme> Renderer for crate::Renderer<Theme> {
    fn draw_pipeline_primitive(
        &mut self,
        bounds: Rectangle,
        primitive: impl Primitive,
    ) {
        self.draw_primitive(super::Primitive::Custom(super::Custom::Pipeline(
            Pipeline::new(bounds, primitive),
        )));
    }
}

/// Stores custom, user-provided pipelines.
#[derive(Default, Debug)]
pub struct Storage {
    pipelines: HashMap<TypeId, Box<dyn Any + Send>>,
}

impl Storage {
    /// Returns `true` if `Storage` contains a pipeline with type `T`.
    pub fn has<T: 'static>(&self) -> bool {
        self.pipelines.get(&TypeId::of::<T>()).is_some()
    }

    /// Inserts the pipeline `T` in to [`Storage`].
    pub fn store<T: 'static + Send>(&mut self, pipeline: T) {
        let _ = self.pipelines.insert(TypeId::of::<T>(), Box::new(pipeline));
    }

    /// Returns a reference to pipeline with type `T` if it exists in [`Storage`].
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.pipelines.get(&TypeId::of::<T>()).map(|pipeline| {
            pipeline
                .downcast_ref::<T>()
                .expect("Pipeline with this type does not exist in Storage.")
        })
    }

    /// Returns a mutable reference to pipeline `T` if it exists in [`Storage`].
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.pipelines.get_mut(&TypeId::of::<T>()).map(|pipeline| {
            pipeline
                .downcast_mut::<T>()
                .expect("Pipeline with this type does not exist in Storage.")
        })
    }
}
