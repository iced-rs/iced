use crate::blur;
use crate::cached_scale;
use crate::gradient_fade;
use crate::graphics::{Antialiasing, Shell};
use crate::primitive;
use crate::quad;
use crate::text;
use crate::triangle;

use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct Engine {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) format: wgpu::TextureFormat,

    pub(crate) quad_pipeline: quad::Pipeline,
    pub(crate) text_pipeline: text::Pipeline,
    pub(crate) triangle_pipeline: triangle::Pipeline,
    #[cfg(any(feature = "image", feature = "svg"))]
    pub(crate) image_pipeline: crate::image::Pipeline,
    pub(crate) gradient_fade_pipeline: gradient_fade::Pipeline,
    pub(crate) cached_scale_pipeline: cached_scale::Pipeline,
    pub(crate) blur_pipeline: blur::Pipeline,
    pub(crate) primitive_storage: Arc<RwLock<primitive::Storage>>,
    _shell: Shell,
}

impl Engine {
    pub fn new(
        _adapter: &wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
        format: wgpu::TextureFormat,
        antialiasing: Option<Antialiasing>, // TODO: Initialize AA pipelines lazily
        shell: Shell,
    ) -> Self {
        Self {
            format,

            quad_pipeline: quad::Pipeline::new(&device, format),
            text_pipeline: text::Pipeline::new(&device, &queue, format),
            triangle_pipeline: triangle::Pipeline::new(&device, format, antialiasing),

            #[cfg(any(feature = "image", feature = "svg"))]
            image_pipeline: {
                let backend = _adapter.get_info().backend;

                crate::image::Pipeline::new(&device, format, backend)
            },

            gradient_fade_pipeline: gradient_fade::Pipeline::new(&device, format),
            cached_scale_pipeline: cached_scale::Pipeline::new(&device, format),
            blur_pipeline: blur::Pipeline::new(&device, format),
            primitive_storage: Arc::new(RwLock::new(primitive::Storage::default())),

            device,
            queue,
            _shell: shell,
        }
    }

    #[cfg(any(feature = "image", feature = "svg"))]
    pub fn create_image_cache(&self) -> crate::image::Cache {
        self.image_pipeline
            .create_cache(&self.device, &self.queue, &self._shell)
    }

    pub fn trim(&mut self) {
        self.text_pipeline.trim();

        self.primitive_storage
            .write()
            .expect("primitive storage should be writable")
            .trim();
    }
}
