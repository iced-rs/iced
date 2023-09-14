use crate::core::{Rectangle, Size};
use crate::graphics::Transformation;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;

/// Stores custom, user-provided pipelines.
#[derive(Default, Debug)]
pub struct Storage {
    pipelines: HashMap<TypeId, Box<dyn Any>>,
}

impl Storage {
    /// Returns `true` if `Storage` contains a pipeline with type `T`.
    pub fn has<T: 'static>(&self) -> bool {
        self.pipelines.get(&TypeId::of::<T>()).is_some()
    }

    /// Inserts the pipeline `T` in to [`Storage`].
    pub fn store<T: 'static>(&mut self, pipeline: T) {
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

/// A set of methods which allows a [`Primitive`] to be rendered.
pub trait Primitive: Debug + Send + Sync + 'static {
    /// Processes the [`Primitive`], allowing for GPU buffer allocation.
    fn prepare(
        &self,
        format: wgpu::TextureFormat,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_size: Size<u32>,
        scale_factor: f32,
        transform: Transformation,
        storage: &mut Storage,
    );

    /// Renders the [`Primitive`].
    fn render(
        &self,
        storage: &Storage,
        bounds: Rectangle<u32>,
        target: &wgpu::TextureView,
        target_size: Size<u32>,
        encoder: &mut wgpu::CommandEncoder,
    );
}
