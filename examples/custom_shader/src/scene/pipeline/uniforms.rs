use crate::scene::Camera;

use iced::{Color, Rectangle};

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Uniforms {
    camera_proj: glam::Mat4,
    camera_pos: glam::Vec4,
    light_color: glam::Vec4,
}

impl Uniforms {
    pub fn new(camera: &Camera, bounds: Rectangle, light_color: Color) -> Self {
        let camera_proj = camera.build_view_proj_matrix(bounds);

        Self {
            camera_proj,
            camera_pos: camera.position(),
            light_color: glam::Vec4::from(light_color.into_linear()),
        }
    }
}
