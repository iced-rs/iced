//! Draw meshes of triangles.
mod gradient;
mod solid;

use crate::program;
use crate::Transformation;

use iced_graphics::layer::mesh::{self, Mesh};
use iced_graphics::triangle::{self, Vertex2D};

use glow::HasContext;
use std::marker::PhantomData;

#[derive(Debug)]
pub(crate) struct Pipeline {
    vertex_array: <glow::Context as HasContext>::VertexArray,
    vertices: Buffer<Vertex2D>,
    indices: Buffer<u32>,
    programs: ProgramList,
}

#[derive(Debug)]
struct ProgramList {
    solid: solid::Program,
    gradient: gradient::Program,
}

impl Pipeline {
    pub fn new(gl: &glow::Context, shader_version: &program::Version) -> Self {
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
                std::mem::size_of::<Vertex2D>() as usize,
            )
        };

        let indices = unsafe {
            Buffer::new(
                gl,
                glow::ELEMENT_ARRAY_BUFFER,
                glow::DYNAMIC_DRAW,
                std::mem::size_of::<u32>() as usize,
            )
        };

        unsafe {
            let stride = std::mem::size_of::<Vertex2D>() as i32;

            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);

            gl.bind_vertex_array(None);
        };

        Self {
            vertex_array,
            vertices,
            indices,
            programs: ProgramList {
                solid: solid::Program::new(gl, shader_version),
                gradient: gradient::Program::new(gl, shader_version),
            },
        }
    }

    pub fn draw(
        &mut self,
        meshes: &[Mesh<'_>],
        gl: &glow::Context,
        target_height: u32,
        transformation: Transformation,
        scale_factor: f32,
    ) {
        unsafe {
            gl.enable(glow::MULTISAMPLE);
            gl.enable(glow::SCISSOR_TEST);
            gl.bind_vertex_array(Some(self.vertex_array))
        }

        //count the total amount of vertices & indices we need to handle
        let (total_vertices, total_indices) = mesh::attribute_count_of(meshes);

        // Then we ensure the current attribute buffers are big enough, resizing if necessary
        unsafe {
            self.vertices.bind(gl, total_vertices);
            self.indices.bind(gl, total_indices);
        }

        // We upload all the vertices and indices upfront
        let mut vertex_offset = 0;
        let mut index_offset = 0;

        for mesh in meshes {
            unsafe {
                gl.buffer_sub_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    (vertex_offset * std::mem::size_of::<Vertex2D>()) as i32,
                    bytemuck::cast_slice(&mesh.buffers.vertices),
                );

                gl.buffer_sub_data_u8_slice(
                    glow::ELEMENT_ARRAY_BUFFER,
                    (index_offset * std::mem::size_of::<u32>()) as i32,
                    bytemuck::cast_slice(&mesh.buffers.indices),
                );

                vertex_offset += mesh.buffers.vertices.len();
                index_offset += mesh.buffers.indices.len();
            }
        }

        // Then we draw each mesh using offsets
        let mut last_vertex = 0;
        let mut last_index = 0;

        for mesh in meshes {
            let transform = transformation
                * Transformation::translate(mesh.origin.x, mesh.origin.y);

            let clip_bounds = (mesh.clip_bounds * scale_factor).snap();

            unsafe {
                gl.scissor(
                    clip_bounds.x as i32,
                    (target_height - (clip_bounds.y + clip_bounds.height))
                        as i32,
                    clip_bounds.width as i32,
                    clip_bounds.height as i32,
                );

                match mesh.style {
                    triangle::Style::Solid(color) => {
                        self.programs.solid.use_program(gl, color, &transform);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    triangle::Style::Gradient(gradient) => {
                        self.programs
                            .gradient
                            .use_program(gl, gradient, &transform);
                    }
                }

                gl.draw_elements_base_vertex(
                    glow::TRIANGLES,
                    mesh.buffers.indices.len() as i32,
                    glow::UNSIGNED_INT,
                    (last_index * std::mem::size_of::<u32>()) as i32,
                    last_vertex as i32,
                );

                last_vertex += mesh.buffers.vertices.len();
                last_index += mesh.buffers.indices.len();
            }
        }

        unsafe {
            gl.bind_vertex_array(None);
            gl.disable(glow::SCISSOR_TEST);
            gl.disable(glow::MULTISAMPLE);
        }
    }
}

/// A simple shader program. Uses [`triangle.vert`] for its vertex shader and only binds position
/// attribute location.
pub(super) fn program(
    gl: &glow::Context,
    shader_version: &program::Version,
    fragment_shader: &'static str,
) -> <glow::Context as HasContext>::Program {
    unsafe {
        let vertex_shader = program::Shader::vertex(
            gl,
            shader_version,
            include_str!("shader/common/triangle.vert"),
        );

        let fragment_shader =
            program::Shader::fragment(gl, shader_version, fragment_shader);

        program::create(
            gl,
            &[vertex_shader, fragment_shader],
            &[(0, "i_Position")],
        )
    }
}

pub fn set_transform(
    gl: &glow::Context,
    location: <glow::Context as HasContext>::UniformLocation,
    transform: Transformation,
) {
    unsafe {
        gl.uniform_matrix_4_f32_slice(
            Some(&location),
            false,
            transform.as_ref(),
        );
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
