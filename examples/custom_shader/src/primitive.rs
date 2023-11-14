pub mod cube;
pub mod vertex;

mod buffer;
mod uniforms;

pub use buffer::Buffer;
pub use cube::Cube;
pub use uniforms::Uniforms;
pub use vertex::Vertex;

use crate::Camera;
use crate::Pipeline;

use iced::advanced::graphics::Transformation;
use iced::widget::shader;
use iced::{Color, Rectangle, Size};

/// A collection of `Cube`s that can be rendered.
#[derive(Debug)]
pub struct Primitive {
    cubes: Vec<cube::Raw>,
    uniforms: Uniforms,
    show_depth_buffer: bool,
}

impl Primitive {
    pub fn new(
        cubes: &[Cube],
        camera: &Camera,
        bounds: Rectangle,
        show_depth_buffer: bool,
        light_color: Color,
    ) -> Self {
        let uniforms = Uniforms::new(camera, bounds, light_color);

        Self {
            cubes: cubes
                .iter()
                .map(cube::Raw::from_cube)
                .collect::<Vec<cube::Raw>>(),
            uniforms,
            show_depth_buffer,
        }
    }
}

impl shader::Primitive for Primitive {
    fn prepare(
        &self,
        format: wgpu::TextureFormat,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_size: Size<u32>,
        _scale_factor: f32,
        _transform: Transformation,
        storage: &mut shader::Storage,
    ) {
        if !storage.has::<Pipeline>() {
            storage.store(Pipeline::new(device, queue, format, target_size));
        }

        let pipeline = storage.get_mut::<Pipeline>().unwrap();

        //upload data to GPU
        pipeline.update(
            device,
            queue,
            target_size,
            &self.uniforms,
            self.cubes.len(),
            &self.cubes,
        );
    }

    fn render(
        &self,
        storage: &shader::Storage,
        bounds: Rectangle<u32>,
        target: &wgpu::TextureView,
        _target_size: Size<u32>,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        //at this point our pipeline should always be initialized
        let pipeline = storage.get::<Pipeline>().unwrap();

        //render primitive
        pipeline.render(
            target,
            encoder,
            bounds,
            self.cubes.len() as u32,
            self.show_depth_buffer,
        );
    }
}
