//! Draw meshes of triangles.
use crate::program;
use crate::Transformation;

use iced_graphics::gradient::Gradient;
use iced_graphics::layer::mesh::{self, Mesh};
use iced_graphics::triangle::{ColoredVertex2D, Vertex2D};

use glow::HasContext;
use std::marker::PhantomData;

const DEFAULT_VERTICES: usize = 1_000;
const DEFAULT_INDICES: usize = 1_000;

#[derive(Debug)]
pub(crate) struct Pipeline {
    indices: Buffer<u32>,
    solid: solid::Program,
    gradient: gradient::Program,
}

impl Pipeline {
    pub fn new(gl: &glow::Context, shader_version: &program::Version) -> Self {
        let mut indices = unsafe {
            Buffer::new(
                gl,
                glow::ELEMENT_ARRAY_BUFFER,
                glow::DYNAMIC_DRAW,
                DEFAULT_INDICES,
            )
        };

        let solid = solid::Program::new(gl, shader_version);
        let gradient = gradient::Program::new(gl, shader_version);

        unsafe {
            gl.bind_vertex_array(Some(solid.vertex_array));
            indices.bind(gl, 0);

            gl.bind_vertex_array(Some(gradient.vertex_array));
            indices.bind(gl, 0);

            gl.bind_vertex_array(None);
        }

        Self {
            indices,
            solid,
            gradient,
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
        }

        // Count the total amount of vertices & indices we need to handle
        let count = mesh::attribute_count_of(meshes);

        // Then we ensure the current attribute buffers are big enough, resizing if necessary
        unsafe {
            self.indices.bind(gl, count.indices);
        }

        // We upload all the vertices and indices upfront
        let mut solid_vertex_offset = 0;
        let mut gradient_vertex_offset = 0;
        let mut index_offset = 0;

        for mesh in meshes {
            let indices = mesh.indices();

            unsafe {
                gl.buffer_sub_data_u8_slice(
                    glow::ELEMENT_ARRAY_BUFFER,
                    (index_offset * std::mem::size_of::<u32>()) as i32,
                    bytemuck::cast_slice(indices),
                );

                index_offset += indices.len();
            }

            match mesh {
                Mesh::Solid { buffers, .. } => {
                    unsafe {
                        self.solid.vertices.bind(gl, count.solid_vertices);

                        gl.buffer_sub_data_u8_slice(
                            glow::ARRAY_BUFFER,
                            (solid_vertex_offset
                                * std::mem::size_of::<ColoredVertex2D>())
                                as i32,
                            bytemuck::cast_slice(&buffers.vertices),
                        );
                    }

                    solid_vertex_offset += buffers.vertices.len();
                }
                Mesh::Gradient { buffers, .. } => {
                    unsafe {
                        self.gradient
                            .vertices
                            .bind(gl, count.gradient_vertices);

                        gl.buffer_sub_data_u8_slice(
                            glow::ARRAY_BUFFER,
                            (gradient_vertex_offset
                                * std::mem::size_of::<Vertex2D>())
                                as i32,
                            bytemuck::cast_slice(&buffers.vertices),
                        );
                    }

                    gradient_vertex_offset += buffers.vertices.len();
                }
            }
        }

        // Then we draw each mesh using offsets
        let mut last_solid_vertex = 0;
        let mut last_gradient_vertex = 0;
        let mut last_index = 0;

        for mesh in meshes {
            let indices = mesh.indices();
            let origin = mesh.origin();

            let transform =
                transformation * Transformation::translate(origin.x, origin.y);

            let clip_bounds = (mesh.clip_bounds() * scale_factor).snap();

            unsafe {
                gl.scissor(
                    clip_bounds.x as i32,
                    (target_height - (clip_bounds.y + clip_bounds.height))
                        as i32,
                    clip_bounds.width as i32,
                    clip_bounds.height as i32,
                );
            }

            match mesh {
                Mesh::Solid { buffers, .. } => unsafe {
                    gl.use_program(Some(self.solid.program));
                    gl.bind_vertex_array(Some(self.solid.vertex_array));

                    if transform != self.solid.uniforms.transform {
                        gl.uniform_matrix_4_f32_slice(
                            Some(&self.solid.uniforms.transform_location),
                            false,
                            transform.as_ref(),
                        );

                        self.solid.uniforms.transform = transform;
                    }

                    gl.draw_elements_base_vertex(
                        glow::TRIANGLES,
                        indices.len() as i32,
                        glow::UNSIGNED_INT,
                        (last_index * std::mem::size_of::<u32>()) as i32,
                        last_solid_vertex as i32,
                    );

                    last_solid_vertex += buffers.vertices.len();
                },
                Mesh::Gradient {
                    buffers, gradient, ..
                } => unsafe {
                    gl.use_program(Some(self.gradient.program));
                    gl.bind_vertex_array(Some(self.gradient.vertex_array));

                    if transform != self.gradient.uniforms.transform {
                        gl.uniform_matrix_4_f32_slice(
                            Some(&self.gradient.uniforms.locations.transform),
                            false,
                            transform.as_ref(),
                        );

                        self.gradient.uniforms.transform = transform;
                    }

                    if &self.gradient.uniforms.gradient != *gradient {
                        match gradient {
                            Gradient::Linear(linear) => {
                                gl.uniform_4_f32(
                                    Some(
                                        &self
                                            .gradient
                                            .uniforms
                                            .locations
                                            .gradient_direction,
                                    ),
                                    linear.start.x,
                                    linear.start.y,
                                    linear.end.x,
                                    linear.end.y,
                                );

                                gl.uniform_1_i32(
                                    Some(
                                        &self
                                            .gradient
                                            .uniforms
                                            .locations
                                            .color_stops_size,
                                    ),
                                    (linear.color_stops.len() * 2) as i32,
                                );

                                let mut stops = [0.0; 128];

                                for (index, stop) in linear
                                    .color_stops
                                    .iter()
                                    .enumerate()
                                    .take(16)
                                {
                                    let [r, g, b, a] = stop.color.into_linear();

                                    stops[index * 8] = r;
                                    stops[(index * 8) + 1] = g;
                                    stops[(index * 8) + 2] = b;
                                    stops[(index * 8) + 3] = a;
                                    stops[(index * 8) + 4] = stop.offset;
                                    stops[(index * 8) + 5] = 0.;
                                    stops[(index * 8) + 6] = 0.;
                                    stops[(index * 8) + 7] = 0.;
                                }

                                gl.uniform_4_f32_slice(
                                    Some(
                                        &self
                                            .gradient
                                            .uniforms
                                            .locations
                                            .color_stops,
                                    ),
                                    &stops,
                                );
                            }
                        }

                        self.gradient.uniforms.gradient = (*gradient).clone();
                    }

                    gl.draw_elements_base_vertex(
                        glow::TRIANGLES,
                        indices.len() as i32,
                        glow::UNSIGNED_INT,
                        (last_index * std::mem::size_of::<u32>()) as i32,
                        last_gradient_vertex as i32,
                    );

                    last_gradient_vertex += buffers.vertices.len();
                },
            }

            last_index += indices.len();
        }

        unsafe {
            gl.bind_vertex_array(None);
            gl.disable(glow::SCISSOR_TEST);
            gl.disable(glow::MULTISAMPLE);
        }
    }
}

#[derive(Debug)]
pub struct Buffer<T> {
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

mod solid {
    use crate::program;
    use crate::triangle;
    use glow::{Context, HasContext, NativeProgram};
    use iced_graphics::triangle::ColoredVertex2D;
    use iced_graphics::Transformation;

    #[derive(Debug)]
    pub struct Program {
        pub program: <Context as HasContext>::Program,
        pub vertex_array: <glow::Context as HasContext>::VertexArray,
        pub vertices: triangle::Buffer<ColoredVertex2D>,
        pub uniforms: Uniforms,
    }

    impl Program {
        pub fn new(gl: &Context, shader_version: &program::Version) -> Self {
            let program = unsafe {
                let vertex_shader = program::Shader::vertex(
                    gl,
                    shader_version,
                    include_str!("shader/common/solid.vert"),
                );

                let fragment_shader = program::Shader::fragment(
                    gl,
                    shader_version,
                    include_str!("shader/common/solid.frag"),
                );

                program::create(
                    gl,
                    &[vertex_shader, fragment_shader],
                    &[(0, "i_Position"), (1, "i_Color")],
                )
            };

            let vertex_array = unsafe {
                gl.create_vertex_array().expect("Create vertex array")
            };

            let vertices = unsafe {
                triangle::Buffer::new(
                    gl,
                    glow::ARRAY_BUFFER,
                    glow::DYNAMIC_DRAW,
                    super::DEFAULT_VERTICES,
                )
            };

            unsafe {
                gl.bind_vertex_array(Some(vertex_array));

                let stride = std::mem::size_of::<ColoredVertex2D>() as i32;

                gl.enable_vertex_attrib_array(0);
                gl.vertex_attrib_pointer_f32(
                    0,
                    2,
                    glow::FLOAT,
                    false,
                    stride,
                    0,
                );

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
            };

            Self {
                program,
                vertex_array,
                vertices,
                uniforms: Uniforms::new(gl, program),
            }
        }
    }

    #[derive(Debug)]
    pub struct Uniforms {
        pub transform: Transformation,
        pub transform_location: <Context as HasContext>::UniformLocation,
    }

    impl Uniforms {
        fn new(gl: &Context, program: NativeProgram) -> Self {
            let transform = Transformation::identity();
            let transform_location =
                unsafe { gl.get_uniform_location(program, "u_Transform") }
                    .expect("Solid - Get u_Transform.");

            unsafe {
                gl.use_program(Some(program));

                gl.uniform_matrix_4_f32_slice(
                    Some(&transform_location),
                    false,
                    transform.as_ref(),
                );

                gl.use_program(None);
            }

            Self {
                transform,
                transform_location,
            }
        }
    }
}

mod gradient {
    use crate::program;
    use crate::triangle;
    use glow::{Context, HasContext, NativeProgram};
    use iced_graphics::gradient::{self, Gradient};
    use iced_graphics::triangle::Vertex2D;
    use iced_graphics::Transformation;

    #[derive(Debug)]
    pub struct Program {
        pub program: <Context as HasContext>::Program,
        pub vertex_array: <glow::Context as HasContext>::VertexArray,
        pub vertices: triangle::Buffer<Vertex2D>,
        pub uniforms: Uniforms,
    }

    impl Program {
        pub fn new(gl: &Context, shader_version: &program::Version) -> Self {
            let program = unsafe {
                let vertex_shader = program::Shader::vertex(
                    gl,
                    shader_version,
                    include_str!("shader/common/gradient.vert"),
                );

                let fragment_shader = program::Shader::fragment(
                    gl,
                    shader_version,
                    include_str!("shader/common/gradient.frag"),
                );

                program::create(
                    gl,
                    &[vertex_shader, fragment_shader],
                    &[(0, "i_Position")],
                )
            };

            let vertex_array = unsafe {
                gl.create_vertex_array().expect("Create vertex array")
            };

            let vertices = unsafe {
                triangle::Buffer::new(
                    gl,
                    glow::ARRAY_BUFFER,
                    glow::DYNAMIC_DRAW,
                    super::DEFAULT_VERTICES,
                )
            };

            unsafe {
                gl.bind_vertex_array(Some(vertex_array));

                let stride = std::mem::size_of::<Vertex2D>() as i32;

                gl.enable_vertex_attrib_array(0);
                gl.vertex_attrib_pointer_f32(
                    0,
                    2,
                    glow::FLOAT,
                    false,
                    stride,
                    0,
                );

                gl.bind_vertex_array(None);
            };

            Self {
                program,
                vertex_array,
                vertices,
                uniforms: Uniforms::new(gl, program),
            }
        }
    }

    #[derive(Debug)]
    pub struct Uniforms {
        pub gradient: Gradient,
        pub transform: Transformation,
        pub locations: Locations,
    }

    #[derive(Debug)]
    pub struct Locations {
        pub gradient_direction: <Context as HasContext>::UniformLocation,
        pub color_stops_size: <Context as HasContext>::UniformLocation,
        //currently the maximum number of stops is 16 due to lack of SSBO in GL2.1
        pub color_stops: <Context as HasContext>::UniformLocation,
        pub transform: <Context as HasContext>::UniformLocation,
    }

    impl Uniforms {
        fn new(gl: &Context, program: NativeProgram) -> Self {
            let gradient_direction = unsafe {
                gl.get_uniform_location(program, "gradient_direction")
            }
            .expect("Gradient - Get gradient_direction.");

            let color_stops_size =
                unsafe { gl.get_uniform_location(program, "color_stops_size") }
                    .expect("Gradient - Get color_stops_size.");

            let color_stops = unsafe {
                gl.get_uniform_location(program, "color_stops")
                    .expect("Gradient - Get color_stops.")
            };

            let transform = Transformation::identity();
            let transform_location =
                unsafe { gl.get_uniform_location(program, "u_Transform") }
                    .expect("Solid - Get u_Transform.");

            unsafe {
                gl.use_program(Some(program));

                gl.uniform_matrix_4_f32_slice(
                    Some(&transform_location),
                    false,
                    transform.as_ref(),
                );

                gl.use_program(None);
            }

            Self {
                gradient: Gradient::Linear(gradient::Linear {
                    start: Default::default(),
                    end: Default::default(),
                    color_stops: vec![],
                }),
                transform: Transformation::identity(),
                locations: Locations {
                    gradient_direction,
                    color_stops_size,
                    color_stops,
                    transform: transform_location,
                },
            }
        }
    }
}
