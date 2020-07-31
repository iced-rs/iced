//! Draw meshes of triangles.
use crate::program;
use crate::Transformation;
use glow::HasContext;
use iced_graphics::layer;
use std::marker::PhantomData;

pub use iced_graphics::triangle::{Mesh2D, Vertex2D};

const VERTEX_BUFFER_SIZE: usize = 10_000;
const INDEX_BUFFER_SIZE: usize = 10_000;

#[derive(Debug)]
pub(crate) struct Pipeline {
    program: <glow::Context as HasContext>::Program,
    vertex_array: <glow::Context as HasContext>::VertexArray,
    vertices: Buffer<Vertex2D>,
    indices: Buffer<u32>,
    transform_location: <glow::Context as HasContext>::UniformLocation,
    current_transform: Transformation,
}

impl Pipeline {
    pub fn new(gl: &glow::Context) -> Pipeline {
        let program = unsafe {
            program::create(
                gl,
                &[
                    (glow::VERTEX_SHADER, include_str!("shader/triangle.vert")),
                    (
                        glow::FRAGMENT_SHADER,
                        include_str!("shader/triangle.frag"),
                    ),
                ],
            )
        };

        let transform_location =
            unsafe { gl.get_uniform_location(program, "u_Transform") }
                .expect("Get transform location");

        unsafe {
            gl.use_program(Some(program));

            let transform: [f32; 16] = Transformation::identity().into();
            gl.uniform_matrix_4_f32_slice(
                Some(&transform_location),
                false,
                &transform,
            );

            gl.use_program(None);
        }

        let vertex_array =
            unsafe { gl.create_vertex_array().expect("Create vertex array") };

        unsafe {
            gl.bind_vertex_array(Some(vertex_array));
        }

        let vertices = unsafe {
            Buffer::new(
                gl,
                glow::ARRAY_BUFFER,
                glow::DYNAMIC_DRAW,
                VERTEX_BUFFER_SIZE,
            )
        };

        let indices = unsafe {
            Buffer::new(
                gl,
                glow::ELEMENT_ARRAY_BUFFER,
                glow::DYNAMIC_DRAW,
                INDEX_BUFFER_SIZE,
            )
        };

        unsafe {
            let stride = std::mem::size_of::<Vertex2D>() as i32;

            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);

            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(
                1,
                4,
                glow::FLOAT,
                false,
                stride,
                4 * 2,
            );

            gl.bind_vertex_array(None);
        }

        Pipeline {
            program,
            vertex_array,
            vertices,
            indices,
            transform_location,
            current_transform: Transformation::identity(),
        }
    }

    pub fn draw(
        &mut self,
        gl: &glow::Context,
        target_height: u32,
        transformation: Transformation,
        scale_factor: f32,
        meshes: &[layer::Mesh<'_>],
    ) {
        unsafe {
            gl.enable(glow::MULTISAMPLE);
            gl.enable(glow::SCISSOR_TEST);
            gl.use_program(Some(self.program));
            gl.bind_vertex_array(Some(self.vertex_array));
        }

        // This looks a bit crazy, but we are just counting how many vertices
        // and indices we will need to handle.
        // TODO: Improve readability
        let (total_vertices, total_indices) = meshes
            .iter()
            .map(|layer::Mesh { buffers, .. }| {
                (buffers.vertices.len(), buffers.indices.len())
            })
            .fold((0, 0), |(total_v, total_i), (v, i)| {
                (total_v + v, total_i + i)
            });

        // Then we ensure the current buffers are big enough, resizing if
        // necessary
        unsafe {
            self.vertices.bind(gl, total_vertices);
            self.indices.bind(gl, total_indices);
        }

        // We upload all the vertices and indices upfront
        let mut last_vertex = 0;
        let mut last_index = 0;

        for layer::Mesh { buffers, .. } in meshes {
            unsafe {
                gl.buffer_sub_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    (last_vertex * std::mem::size_of::<Vertex2D>()) as i32,
                    bytemuck::cast_slice(&buffers.vertices),
                );

                gl.buffer_sub_data_u8_slice(
                    glow::ELEMENT_ARRAY_BUFFER,
                    (last_index * std::mem::size_of::<u32>()) as i32,
                    bytemuck::cast_slice(&buffers.indices),
                );

                last_vertex += buffers.vertices.len();
                last_index += buffers.indices.len();
            }
        }

        // Then we draw each mesh using offsets
        let mut last_vertex = 0;
        let mut last_index = 0;

        for layer::Mesh {
            buffers,
            origin,
            clip_bounds,
        } in meshes
        {
            let transform =
                transformation * Transformation::translate(origin.x, origin.y);

            let clip_bounds = (*clip_bounds * scale_factor).snap();

            unsafe {
                if self.current_transform != transform {
                    let matrix: [f32; 16] = transform.into();
                    gl.uniform_matrix_4_f32_slice(
                        Some(&self.transform_location),
                        false,
                        &matrix,
                    );

                    self.current_transform = transform;
                }

                gl.scissor(
                    clip_bounds.x as i32,
                    (target_height - (clip_bounds.y + clip_bounds.height))
                        as i32,
                    clip_bounds.width as i32,
                    clip_bounds.height as i32,
                );

                gl.draw_elements_base_vertex(
                    glow::TRIANGLES,
                    buffers.indices.len() as i32,
                    glow::UNSIGNED_INT,
                    (last_index * std::mem::size_of::<u32>()) as i32,
                    last_vertex as i32,
                );

                last_vertex += buffers.vertices.len();
                last_index += buffers.indices.len();
            }
        }

        unsafe {
            gl.bind_vertex_array(None);
            gl.use_program(None);
            gl.disable(glow::SCISSOR_TEST);
            gl.disable(glow::MULTISAMPLE);
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Uniforms {
    transform: [f32; 16],
}

unsafe impl bytemuck::Zeroable for Uniforms {}
unsafe impl bytemuck::Pod for Uniforms {}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            transform: *Transformation::identity().as_ref(),
        }
    }
}

impl From<Transformation> for Uniforms {
    fn from(transformation: Transformation) -> Uniforms {
        Self {
            transform: transformation.into(),
        }
    }
}

#[derive(Debug)]
struct Buffer<T> {
    raw: <glow::Context as HasContext>::Buffer,
    target: u32,
    usage: u32,
    size: usize,
    phantom: PhantomData<T>,
}

impl<T> Buffer<T> {
    pub unsafe fn new(
        gl: &glow::Context,
        target: u32,
        usage: u32,
        size: usize,
    ) -> Self {
        let raw = gl.create_buffer().expect("Create buffer");

        let mut buffer = Buffer {
            raw,
            target,
            usage,
            size: 0,
            phantom: PhantomData,
        };

        buffer.bind(gl, size);

        buffer
    }

    pub unsafe fn bind(&mut self, gl: &glow::Context, size: usize) {
        gl.bind_buffer(self.target, Some(self.raw));

        if self.size < size {
            gl.buffer_data_size(
                self.target,
                (size * std::mem::size_of::<T>()) as i32,
                self.usage,
            );

            self.size = size;
        }
    }
}
