use crate::core::{self, Size};
use crate::image::atlas::{self, Atlas};

use std::cell::{RefCell, RefMut};
use std::rc::Rc;

#[derive(Debug)]
pub struct Cache {
    atlas: Atlas,
    #[cfg(feature = "image")]
    raster: crate::image::raster::Cache,
    #[cfg(feature = "svg")]
    vector: crate::image::vector::Cache,
}

impl Cache {
    pub fn new(device: &wgpu::Device, backend: wgpu::Backend) -> Self {
        Self {
            atlas: Atlas::new(device, backend),
            #[cfg(feature = "image")]
            raster: crate::image::raster::Cache::default(),
            #[cfg(feature = "svg")]
            vector: crate::image::vector::Cache::default(),
        }
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

    pub fn create_bind_group(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("iced_wgpu::image texture atlas bind group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(self.atlas.view()),
            }],
        })
    }

    pub fn trim(&mut self) {
        #[cfg(feature = "image")]
        self.raster.trim(&mut self.atlas);

        #[cfg(feature = "svg")]
        self.vector.trim(&mut self.atlas);
    }
}

#[derive(Debug, Clone)]
pub struct Shared(Rc<RefCell<Cache>>);

impl Shared {
    pub fn new(cache: Cache) -> Self {
        Self(Rc::new(RefCell::new(cache)))
    }

    pub fn lock(&self) -> RefMut<'_, Cache> {
        self.0.borrow_mut()
    }
}
