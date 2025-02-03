use crate::buffer;
use crate::graphics::Antialiasing;
use crate::primitive;
use crate::quad;
use crate::text;
use crate::triangle;

#[allow(missing_debug_implementations)]
pub struct Engine {
    pub(crate) staging_belt: wgpu::util::StagingBelt,
    pub(crate) format: wgpu::TextureFormat,

    pub(crate) quad_pipeline: quad::Pipeline,
    pub(crate) text_pipeline: text::Pipeline,
    pub(crate) triangle_pipeline: triangle::Pipeline,
    #[cfg(any(feature = "image", feature = "svg"))]
    pub(crate) image_pipeline: crate::image::Pipeline,
    pub(crate) primitive_storage: primitive::Storage,
}

impl Engine {
    pub fn new(
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        antialiasing: Option<Antialiasing>, // TODO: Initialize AA pipelines lazily
    ) -> Self {
        let text_pipeline = text::Pipeline::new(device, queue, format);
        let quad_pipeline = quad::Pipeline::new(device, format);
        let triangle_pipeline =
            triangle::Pipeline::new(device, format, antialiasing);

        #[cfg(any(feature = "image", feature = "svg"))]
        let image_pipeline = {
            let backend = _adapter.get_info().backend;

            crate::image::Pipeline::new(device, format, backend)
        };

        Self {
            // TODO: Resize belt smartly (?)
            // It would be great if the `StagingBelt` API exposed methods
            // for introspection to detect when a resize may be worth it.
            staging_belt: wgpu::util::StagingBelt::new(
                buffer::MAX_WRITE_SIZE as u64,
            ),
            format,

            quad_pipeline,
            text_pipeline,
            triangle_pipeline,

            #[cfg(any(feature = "image", feature = "svg"))]
            image_pipeline,

            primitive_storage: primitive::Storage::default(),
        }
    }

    #[cfg(any(feature = "image", feature = "svg"))]
    pub fn create_image_cache(
        &self,
        device: &wgpu::Device,
    ) -> crate::image::Cache {
        self.image_pipeline.create_cache(device)
    }

    pub fn submit(
        &mut self,
        queue: &wgpu::Queue,
        encoder: wgpu::CommandEncoder,
    ) -> wgpu::SubmissionIndex {
        self.finish();
        let index = queue.submit(Some(encoder.finish()));
        self.end_frame();

        index
    }

    pub fn finish(&mut self) {
        self.staging_belt.finish();
    }

    pub fn end_frame(&mut self) {
        self.staging_belt.recall();

        self.quad_pipeline.end_frame();
        self.text_pipeline.end_frame();
        self.triangle_pipeline.end_frame();

        #[cfg(any(feature = "image", feature = "svg"))]
        self.image_pipeline.end_frame();
    }
}
