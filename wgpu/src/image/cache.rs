use crate::core::{self, Size};
use crate::image::atlas::{self, Atlas};

use std::sync::Arc;

#[derive(Debug)]
pub struct Cache {
    atlas: Atlas,
    #[cfg(feature = "image")]
    raster: crate::image::raster::Cache,
    #[cfg(feature = "svg")]
    vector: crate::image::vector::Cache,
}

impl Cache {
    pub fn new(
        device: &wgpu::Device,
        backend: wgpu::Backend,
        layout: Arc<wgpu::BindGroupLayout>,
    ) -> Self {
        Self {
            atlas: Atlas::new(device, backend, layout),
            #[cfg(feature = "image")]
            raster: crate::image::raster::Cache::default(),
            #[cfg(feature = "svg")]
            vector: crate::image::vector::Cache::default(),
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        self.atlas.bind_group()
    }

    pub fn layer_count(&self) -> usize {
        self.atlas.layer_count()
    }

    #[cfg(feature = "image")]
    pub fn measure_image(&mut self, handle: &core::image::Handle) -> Size<u32> {
        self.raster.load(handle).dimensions()
    }

    #[cfg(feature = "svg")]
    pub fn measure_svg(&mut self, handle: &core::svg::Handle) -> Size<u32> {
        self.vector.load(handle).viewport_dimensions()
    }

    #[cfg(feature = "image")]
    pub fn upload_raster(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        handle: &core::image::Handle,
    ) -> Option<&atlas::Entry> {
        self.raster.upload(device, encoder, handle, &mut self.atlas)
    }

    #[cfg(feature = "svg")]
    pub fn upload_vector(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        handle: &core::svg::Handle,
        color: Option<core::Color>,
        size: [f32; 2],
        scale: f32,
    ) -> Option<&atlas::Entry> {
        self.vector.upload(
            device,
            encoder,
            handle,
            color,
            size,
            scale,
            &mut self.atlas,
        )
    }

    pub fn trim(&mut self) {
        #[cfg(feature = "image")]
        self.raster.trim(&mut self.atlas);

        #[cfg(feature = "svg")]
        self.vector.trim(&mut self.atlas);
    }
}
