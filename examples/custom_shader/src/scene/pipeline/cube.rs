use crate::scene::pipeline::Vertex;
use crate::wgpu;

use glam::{vec2, vec3, Vec3};
use rand::{thread_rng, Rng};

/// A single instance of a cube.
#[derive(Debug, Clone)]
pub struct Cube {
    pub rotation: glam::Quat,
    pub position: Vec3,
    pub size: f32,
    rotation_dir: f32,
    rotation_axis: glam::Vec3,
}

impl Default for Cube {
    fn default() -> Self {
        Self {
            rotation: glam::Quat::IDENTITY,
            position: glam::Vec3::ZERO,
            size: 0.1,
            rotation_dir: 1.0,
            rotation_axis: glam::Vec3::Y,
        }
    }
}

impl Cube {
    pub fn new(size: f32, origin: Vec3) -> Self {
        let rnd = thread_rng().gen_range(0.0..=1.0f32);

        Self {
            rotation: glam::Quat::IDENTITY,
            position: origin + Vec3::new(0.1, 0.1, 0.1),
            size,
            rotation_dir: if rnd <= 0.5 { -1.0 } else { 1.0 },
            rotation_axis: if rnd <= 0.33 {
                glam::Vec3::Y
            } else if rnd <= 0.66 {
                glam::Vec3::X
            } else {
                glam::Vec3::Z
            },
        }
    }

    pub fn update(&mut self, size: f32, time: f32) {
        self.rotation = glam::Quat::from_axis_angle(
            self.rotation_axis,
            time / 2.0 * self.rotation_dir,
        );
        self.size = size;
    }
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
#[repr(C)]
pub struct Raw {
    transformation: glam::Mat4,
    normal: glam::Mat3,
    _padding: [f32; 3],
}

impl Raw {
    const ATTRIBS: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![
        //cube transformation matrix
        4 => Float32x4,
        5 => Float32x4,
        6 => Float32x4,
        7 => Float32x4,
        //normal rotation matrix
        8 => Float32x3,
        9 => Float32x3,
        10 => Float32x3,
    ];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

impl Raw {
    pub fn from_cube(cube: &Cube) -> Raw {
        Raw {
            transformation: glam::Mat4::from_scale_rotation_translation(
                glam::vec3(cube.size, cube.size, cube.size),
                cube.rotation,
                cube.position,
            ),
            normal: glam::Mat3::from_quat(cube.rotation),
            _padding: [0.0; 3],
        }
    }

    pub fn vertices() -> [Vertex; 36] {
        [
            //face 1
            Vertex {
                pos: vec3(-0.5, -0.5, -0.5),
                normal: vec3(0.0, 0.0, -1.0),
                tangent: vec3(1.0, 0.0, 0.0),
                uv: vec2(0.0, 1.0),
            },
            Vertex {
                pos: vec3(0.5, -0.5, -0.5),
                normal: vec3(0.0, 0.0, -1.0),
                tangent: vec3(1.0, 0.0, 0.0),
                uv: vec2(1.0, 1.0),
            },
            Vertex {
                pos: vec3(0.5, 0.5, -0.5),
                normal: vec3(0.0, 0.0, -1.0),
                tangent: vec3(1.0, 0.0, 0.0),
                uv: vec2(1.0, 0.0),
            },
            Vertex {
                pos: vec3(0.5, 0.5, -0.5),
                normal: vec3(0.0, 0.0, -1.0),
                tangent: vec3(1.0, 0.0, 0.0),
                uv: vec2(1.0, 0.0),
            },
            Vertex {
                pos: vec3(-0.5, 0.5, -0.5),
                normal: vec3(0.0, 0.0, -1.0),
                tangent: vec3(1.0, 0.0, 0.0),
                uv: vec2(0.0, 0.0),
            },
            Vertex {
                pos: vec3(-0.5, -0.5, -0.5),
                normal: vec3(0.0, 0.0, -1.0),
                tangent: vec3(1.0, 0.0, 0.0),
                uv: vec2(0.0, 1.0),
            },
            //face 2
            Vertex {
                pos: vec3(-0.5, -0.5, 0.5),
                normal: vec3(0.0, 0.0, 1.0),
                tangent: vec3(1.0, 0.0, 0.0),
                uv: vec2(0.0, 1.0),
            },
            Vertex {
                pos: vec3(0.5, -0.5, 0.5),
                normal: vec3(0.0, 0.0, 1.0),
                tangent: vec3(1.0, 0.0, 0.0),
                uv: vec2(1.0, 1.0),
            },
            Vertex {
                pos: vec3(0.5, 0.5, 0.5),
                normal: vec3(0.0, 0.0, 1.0),
                tangent: vec3(1.0, 0.0, 0.0),
                uv: vec2(1.0, 0.0),
            },
            Vertex {
                pos: vec3(0.5, 0.5, 0.5),
                normal: vec3(0.0, 0.0, 1.0),
                tangent: vec3(1.0, 0.0, 0.0),
                uv: vec2(1.0, 0.0),
            },
            Vertex {
                pos: vec3(-0.5, 0.5, 0.5),
                normal: vec3(0.0, 0.0, 1.0),
                tangent: vec3(1.0, 0.0, 0.0),
                uv: vec2(0.0, 0.0),
            },
            Vertex {
                pos: vec3(-0.5, -0.5, 0.5),
                normal: vec3(0.0, 0.0, 1.0),
                tangent: vec3(1.0, 0.0, 0.0),
                uv: vec2(0.0, 1.0),
            },
            //face 3
            Vertex {
                pos: vec3(-0.5, 0.5, 0.5),
                normal: vec3(-1.0, 0.0, 0.0),
                tangent: vec3(0.0, 0.0, -1.0),
                uv: vec2(0.0, 1.0),
            },
            Vertex {
                pos: vec3(-0.5, 0.5, -0.5),
                normal: vec3(-1.0, 0.0, 0.0),
                tangent: vec3(0.0, 0.0, -1.0),
                uv: vec2(1.0, 1.0),
            },
            Vertex {
                pos: vec3(-0.5, -0.5, -0.5),
                normal: vec3(-1.0, 0.0, 0.0),
                tangent: vec3(0.0, 0.0, -1.0),
                uv: vec2(1.0, 0.0),
            },
            Vertex {
                pos: vec3(-0.5, -0.5, -0.5),
                normal: vec3(-1.0, 0.0, 0.0),
                tangent: vec3(0.0, 0.0, -1.0),
                uv: vec2(1.0, 0.0),
            },
            Vertex {
                pos: vec3(-0.5, -0.5, 0.5),
                normal: vec3(-1.0, 0.0, 0.0),
                tangent: vec3(0.0, 0.0, -1.0),
                uv: vec2(0.0, 0.0),
            },
            Vertex {
                pos: vec3(-0.5, 0.5, 0.5),
                normal: vec3(-1.0, 0.0, 0.0),
                tangent: vec3(0.0, 0.0, -1.0),
                uv: vec2(0.0, 1.0),
            },
            //face 4
            Vertex {
                pos: vec3(0.5, 0.5, 0.5),
                normal: vec3(1.0, 0.0, 0.0),
                tangent: vec3(0.0, 0.0, -1.0),
                uv: vec2(0.0, 1.0),
            },
            Vertex {
                pos: vec3(0.5, 0.5, -0.5),
                normal: vec3(1.0, 0.0, 0.0),
                tangent: vec3(0.0, 0.0, -1.0),
                uv: vec2(1.0, 1.0),
            },
            Vertex {
                pos: vec3(0.5, -0.5, -0.5),
                normal: vec3(1.0, 0.0, 0.0),
                tangent: vec3(0.0, 0.0, -1.0),
                uv: vec2(1.0, 0.0),
            },
            Vertex {
                pos: vec3(0.5, -0.5, -0.5),
                normal: vec3(1.0, 0.0, 0.0),
                tangent: vec3(0.0, 0.0, -1.0),
                uv: vec2(1.0, 0.0),
            },
            Vertex {
                pos: vec3(0.5, -0.5, 0.5),
                normal: vec3(1.0, 0.0, 0.0),
                tangent: vec3(0.0, 0.0, -1.0),
                uv: vec2(0.0, 0.0),
            },
            Vertex {
                pos: vec3(0.5, 0.5, 0.5),
                normal: vec3(1.0, 0.0, 0.0),
                tangent: vec3(0.0, 0.0, -1.0),
                uv: vec2(0.0, 1.0),
            },
            //face 5
            Vertex {
                pos: vec3(-0.5, -0.5, -0.5),
                normal: vec3(0.0, -1.0, 0.0),
                tangent: vec3(1.0, 0.0, 0.0),
                uv: vec2(0.0, 1.0),
            },
            Vertex {
                pos: vec3(0.5, -0.5, -0.5),
                normal: vec3(0.0, -1.0, 0.0),
                tangent: vec3(1.0, 0.0, 0.0),
                uv: vec2(1.0, 1.0),
            },
            Vertex {
                pos: vec3(0.5, -0.5, 0.5),
                normal: vec3(0.0, -1.0, 0.0),
                tangent: vec3(1.0, 0.0, 0.0),
                uv: vec2(1.0, 0.0),
            },
            Vertex {
                pos: vec3(0.5, -0.5, 0.5),
                normal: vec3(0.0, -1.0, 0.0),
                tangent: vec3(1.0, 0.0, 0.0),
                uv: vec2(1.0, 0.0),
            },
            Vertex {
                pos: vec3(-0.5, -0.5, 0.5),
                normal: vec3(0.0, -1.0, 0.0),
                tangent: vec3(1.0, 0.0, 0.0),
                uv: vec2(0.0, 0.0),
            },
            Vertex {
                pos: vec3(-0.5, -0.5, -0.5),
                normal: vec3(0.0, -1.0, 0.0),
                tangent: vec3(1.0, 0.0, 0.0),
                uv: vec2(0.0, 1.0),
            },
            //face 6
            Vertex {
                pos: vec3(-0.5, 0.5, -0.5),
                normal: vec3(0.0, 1.0, 0.0),
                tangent: vec3(1.0, 0.0, 0.0),
                uv: vec2(0.0, 1.0),
            },
            Vertex {
                pos: vec3(0.5, 0.5, -0.5),
                normal: vec3(0.0, 1.0, 0.0),
                tangent: vec3(1.0, 0.0, 0.0),
                uv: vec2(1.0, 1.0),
            },
            Vertex {
                pos: vec3(0.5, 0.5, 0.5),
                normal: vec3(0.0, 1.0, 0.0),
                tangent: vec3(1.0, 0.0, 0.0),
                uv: vec2(1.0, 0.0),
            },
            Vertex {
                pos: vec3(0.5, 0.5, 0.5),
                normal: vec3(0.0, 1.0, 0.0),
                tangent: vec3(1.0, 0.0, 0.0),
                uv: vec2(1.0, 0.0),
            },
            Vertex {
                pos: vec3(-0.5, 0.5, 0.5),
                normal: vec3(0.0, 1.0, 0.0),
                tangent: vec3(1.0, 0.0, 0.0),
                uv: vec2(0.0, 0.0),
            },
            Vertex {
                pos: vec3(-0.5, 0.5, -0.5),
                normal: vec3(0.0, 1.0, 0.0),
                tangent: vec3(1.0, 0.0, 0.0),
                uv: vec2(0.0, 1.0),
            },
        ]
    }
}
