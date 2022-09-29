//! Draw meshes of triangle.
mod gradient;
mod solid;

use crate::program::{self, Shader};
use crate::Transformation;
use glow::HasContext;
use iced_graphics::layer::{Mesh, Meshes};
use iced_graphics::shader;
use std::marker::PhantomData;

use crate::triangle::gradient::GradientProgram;
use crate::triangle::solid::SolidProgram;
pub use iced_graphics::triangle::{Mesh2D, Vertex2D};

#[derive(Debug)]
pub(crate) struct Pipeline {
    vertex_array: <glow::Context as HasContext>::VertexArray,
    vertices: Buffer<Vertex2D>,
    indices: Buffer<u32>,
    current_transform: Transformation,
    programs: TrianglePrograms,
}

#[derive(Debug)]
struct TrianglePrograms {
    solid: TriangleProgram,
    gradient: TriangleProgram,
}

#[derive(Debug)]
enum TriangleProgram {
    Solid(SolidProgram),
    Gradient(GradientProgram),
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
            current_transform: Transformation::identity(),
            programs: TrianglePrograms {
                solid: TriangleProgram::Solid(SolidProgram::new(
                    gl,
                    shader_version,
                )),
                gradient: TriangleProgram::Gradient(GradientProgram::new(
                    gl,
                    shader_version,
                )),
            },
        }
    }

    pub fn draw(
        &mut self,
        meshes: &Meshes<'_>,
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

        //count the total number of vertices & indices we need to handle for all meshes
        let (total_vertices, total_indices) = meshes.attribute_count();

        // Then we ensure the current attribute buffers are big enough, resizing if necessary
        unsafe {
            self.vertices.bind(gl, total_vertices);
            self.indices.bind(gl, total_indices);
        }

        // We upload all the vertices and indices upfront
        let mut last_vertex = 0;
        let mut last_index = 0;

        for Mesh { buffers, .. } in meshes.0.iter() {
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

        for Mesh {
            buffers,
            origin,
            clip_bounds,
            shader,
        } in meshes.0.iter()
        {
            let transform =
                transformation * Transformation::translate(origin.x, origin.y);

            let clip_bounds = (*clip_bounds * scale_factor).snap();

            unsafe {
                gl.scissor(
                    clip_bounds.x as i32,
                    (target_height - (clip_bounds.y + clip_bounds.height))
                        as i32,
                    clip_bounds.width as i32,
                    clip_bounds.height as i32,
                );

                let t = if self.current_transform != transform {
                    self.current_transform = transform;
                    Some(transform)
                } else {
                    None
                };

                self.use_with_shader(gl, shader, t);

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
            gl.disable(glow::SCISSOR_TEST);
            gl.disable(glow::MULTISAMPLE);
        }
    }

    fn use_with_shader(
        &mut self,
        gl: &glow::Context,
        shader: &shader::Shader,
        transform: Option<Transformation>,
    ) {
        match shader {
            shader::Shader::Solid(color) => {
                if let TriangleProgram::Solid(solid_program) =
                    &mut self.programs.solid
                {
                    unsafe { gl.use_program(Some(solid_program.program)) }
                    solid_program.set_uniforms(gl, color, transform);
                }
            }
            shader::Shader::Gradient(gradient) => {
                if let TriangleProgram::Gradient(gradient_program) =
                    &mut self.programs.gradient
                {
                    unsafe { gl.use_program(Some(gradient_program.program)) }
                    gradient_program.set_uniforms(gl, gradient, transform);
                }
            }
        }
    }
}

/// A simple shader program. Uses [`triangle.vert`] for its vertex shader and only binds position
/// attribute location.
pub(super) fn simple_triangle_program(
    gl: &glow::Context,
    shader_version: &program::Version,
    fragment_shader: &'static str,
) -> <glow::Context as HasContext>::Program {
    unsafe {
        let vertex_shader = Shader::vertex(
            gl,
            shader_version,
            include_str!("shader/common/triangle.vert"),
        );

        let fragment_shader =
            Shader::fragment(gl, shader_version, fragment_shader);

        program::create(
            gl,
            &[vertex_shader, fragment_shader],
            &[(0, "i_Position")],
        )
    }
}

pub(super) fn update_transform(
    gl: &glow::Context,
    program: <glow::Context as HasContext>::Program,
    transform: Option<Transformation>
) {
    if let Some(t) = transform {
        let transform_location =
            unsafe { gl.get_uniform_location(program, "u_Transform") }
                .expect("Get transform location.");

        unsafe {
            gl.uniform_matrix_4_f32_slice(
                Some(&transform_location),
                false,
                t.as_ref(),
            );
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
