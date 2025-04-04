use crate::graphics::Antialiasing;
use crate::primitive;
use crate::quad;
use crate::text;
use crate::triangle;

use std::sync::{Arc, RwLock};

#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct Engine {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) format: wgpu::TextureFormat,

    pub(crate) quad_pipeline: quad::Pipeline,
    pub(crate) text_pipeline: text::Pipeline,
    pub(crate) triangle_pipeline: triangle::Pipeline,
    #[cfg(any(feature = "image", feature = "svg"))]
    pub(crate) image_pipeline: crate::image::Pipeline,
    pub(crate) primitive_storage: Arc<RwLock<primitive::Storage>>,
}

impl Engine {
    pub fn new(
        _adapter: &wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
        format: wgpu::TextureFormat,
        antialiasing: Option<Antialiasing>, // TODO: Initialize AA pipelines lazily
    ) -> Self {
        Self {
            format,

            quad_pipeline: quad::Pipeline::new(&device, format),
            text_pipeline: text::Pipeline::new(&device, &queue, format),
            triangle_pipeline: triangle::Pipeline::new(
                &device,
                format,
                antialiasing,
            ),

            #[cfg(any(feature = "image", feature = "svg"))]
            image_pipeline: {
                let backend = _adapter.get_info().backend;

                crate::image::Pipeline::new(&device, format, backend)
            },

            primitive_storage: Arc::new(RwLock::new(
                primitive::Storage::default(),
            )),

            device,
            queue,
        }
    }

    #[cfg(any(feature = "image", feature = "svg"))]
    pub fn create_image_cache(
        &self,
        device: &wgpu::Device,
    ) -> crate::image::Cache {
        self.image_pipeline.create_cache(device)
    }
}
