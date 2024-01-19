use glam::{mat4, vec3, vec4};
use iced::Rectangle;

#[derive(Copy, Clone)]
pub struct Camera {
    eye: glam::Vec3,
    target: glam::Vec3,
    up: glam::Vec3,
    fov_y: f32,
    near: f32,
    far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: vec3(0.0, 2.0, 3.0),
            target: glam::Vec3::ZERO,
            up: glam::Vec3::Y,
            fov_y: 45.0,
            near: 0.1,
            far: 100.0,
        }
    }
}

pub const OPENGL_TO_WGPU_MATRIX: glam::Mat4 = mat4(
    vec4(1.0, 0.0, 0.0, 0.0),
    vec4(0.0, 1.0, 0.0, 0.0),
    vec4(0.0, 0.0, 0.5, 0.0),
    vec4(0.0, 0.0, 0.5, 1.0),
);

impl Camera {
    pub fn build_view_proj_matrix(&self, bounds: Rectangle) -> glam::Mat4 {
        //TODO looks distorted without padding; base on surface texture size instead?
        let aspect_ratio = bounds.width / (bounds.height + 150.0);

        let view = glam::Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = glam::Mat4::perspective_rh(
            self.fov_y,
            aspect_ratio,
            self.near,
            self.far,
        );

        OPENGL_TO_WGPU_MATRIX * proj * view
    }

    pub fn position(&self) -> glam::Vec4 {
        glam::Vec4::from((self.eye, 0.0))
    }
}
