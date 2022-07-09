use crate::program::{self, Shader};
use crate::Transformation;
use glow::HasContext;
use iced_graphics::layer;
use iced_native::Rectangle;

// Only change `MAX_QUADS`, otherwise you could cause problems
// by splitting a triangle into different render passes.
const MAX_QUADS: usize = 100_000;
const MAX_VERTICES: usize = MAX_QUADS * 4;
const MAX_INDICES: usize = MAX_QUADS * 6;

#[derive(Debug)]
pub struct Pipeline {
    program: <glow::Context as HasContext>::Program,
    vertex_array: <glow::Context as HasContext>::VertexArray,
    vertex_buffer: <glow::Context as HasContext>::Buffer,
    index_buffer: <glow::Context as HasContext>::Buffer,
    transform_location: <glow::Context as HasContext>::UniformLocation,
    scale_location: <glow::Context as HasContext>::UniformLocation,
    screen_height_location: <glow::Context as HasContext>::UniformLocation,
    current_transform: Transformation,
    current_scale: f32,
    current_target_height: u32,
}

impl Pipeline {
    pub fn new(
        gl: &glow::Context,
        shader_version: &program::Version,
    ) -> Pipeline {
        let program = unsafe {
            let vertex_shader = Shader::vertex(
                gl,
                shader_version,
                include_str!("../shader/compatibility/quad.vert"),
            );
            let fragment_shader = Shader::fragment(
                gl,
                shader_version,
                include_str!("../shader/compatibility/quad.frag"),
            );

            program::create(
                gl,
                &[vertex_shader, fragment_shader],
                &[
                    (0, "i_Pos"),
                    (1, "i_Scale"),
                    (2, "i_Color"),
                    (3, "i_BorderColor"),
                    (4, "i_BorderRadius"),
                    (5, "i_BorderWidth"),
                ],
            )
        };

        let transform_location =
            unsafe { gl.get_uniform_location(program, "u_Transform") }
                .expect("Get transform location");

        let scale_location =
            unsafe { gl.get_uniform_location(program, "u_Scale") }
                .expect("Get scale location");

        let screen_height_location =
            unsafe { gl.get_uniform_location(program, "u_ScreenHeight") }
                .expect("Get target height location");

        unsafe {
            gl.use_program(Some(program));

            let matrix: [f32; 16] = Transformation::identity().into();
            gl.uniform_matrix_4_f32_slice(
                Some(&transform_location),
                false,
                &matrix,
            );

            gl.uniform_1_f32(Some(&scale_location), 1.0);
            gl.uniform_1_f32(Some(&screen_height_location), 0.0);

            gl.use_program(None);
        }

        let (vertex_array, vertex_buffer, index_buffer) =
            unsafe { create_buffers(gl, MAX_VERTICES) };

        Pipeline {
            program,
            vertex_array,
            vertex_buffer,
            index_buffer,
            transform_location,
            scale_location,
            screen_height_location,
            current_transform: Transformation::identity(),
            current_scale: 1.0,
            current_target_height: 0,
        }
    }

    pub fn draw(
        &mut self,
        gl: &glow::Context,
        target_height: u32,
        instances: &[layer::Quad],
        transformation: Transformation,
        scale: f32,
        bounds: Rectangle<u32>,
    ) {
        // TODO: Remove this allocation (probably by changing the shader and removing the need of two `position`)
        let vertices: Vec<Vertex> =
            instances.iter().flat_map(Vertex::from_quad).collect();

        // TODO: Remove this allocation (or allocate only when needed)
        let indices: Vec<i32> = (0..instances.len().min(MAX_QUADS) as i32)
            .flat_map(|i| {
                [i * 4, 1 + i * 4, 2 + i * 4, 2 + i * 4, 1 + i * 4, 3 + i * 4]
            })
            .cycle()
            .take(instances.len() * 6)
            .collect();

        unsafe {
            gl.enable(glow::SCISSOR_TEST);
            gl.scissor(
                bounds.x as i32,
                (target_height - (bounds.y + bounds.height)) as i32,
                bounds.width as i32,
                bounds.height as i32,
            );

            gl.use_program(Some(self.program));
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vertex_buffer));
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.index_buffer));
        }

        if transformation != self.current_transform {
            unsafe {
                let matrix: [f32; 16] = transformation.into();
                gl.uniform_matrix_4_f32_slice(
                    Some(&self.transform_location),
                    false,
                    &matrix,
                );

                self.current_transform = transformation;
            }
        }

        if scale != self.current_scale {
            unsafe {
                gl.uniform_1_f32(Some(&self.scale_location), scale);
            }

            self.current_scale = scale;
        }

        if target_height != self.current_target_height {
            unsafe {
                gl.uniform_1_f32(
                    Some(&self.screen_height_location),
                    target_height as f32,
                );
            }

            self.current_target_height = target_height;
        }

        let passes = vertices
            .chunks(MAX_VERTICES)
            .zip(indices.chunks(MAX_INDICES));

        for (vertices, indices) in passes {
            unsafe {
                gl.buffer_sub_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    0,
                    bytemuck::cast_slice(vertices),
                );

                gl.buffer_sub_data_u8_slice(
                    glow::ELEMENT_ARRAY_BUFFER,
                    0,
                    bytemuck::cast_slice(indices),
                );

                gl.draw_elements(
                    glow::TRIANGLES,
                    indices.len() as i32,
                    glow::UNSIGNED_INT,
                    0,
                );
            }
        }

        unsafe {
            gl.bind_vertex_array(None);
            gl.use_program(None);
            gl.disable(glow::SCISSOR_TEST);
        }
    }
}

unsafe fn create_buffers(
    gl: &glow::Context,
    size: usize,
) -> (
    <glow::Context as HasContext>::VertexArray,
    <glow::Context as HasContext>::Buffer,
    <glow::Context as HasContext>::Buffer,
) {
    let vertex_array = gl.create_vertex_array().expect("Create vertex array");
    let vertex_buffer = gl.create_buffer().expect("Create vertex buffer");
    let index_buffer = gl.create_buffer().expect("Create index buffer");

    gl.bind_vertex_array(Some(vertex_array));

    gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(index_buffer));
    gl.buffer_data_size(
        glow::ELEMENT_ARRAY_BUFFER,
        12 * size as i32,
        glow::DYNAMIC_DRAW,
    );

    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
    gl.buffer_data_size(
        glow::ARRAY_BUFFER,
        (size * Vertex::SIZE) as i32,
        glow::DYNAMIC_DRAW,
    );

    let stride = Vertex::SIZE as i32;

    gl.enable_vertex_attrib_array(0);
    gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);

    gl.enable_vertex_attrib_array(1);
    gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, stride, 4 * 2);

    gl.enable_vertex_attrib_array(2);
    gl.vertex_attrib_pointer_f32(2, 4, glow::FLOAT, false, stride, 4 * (2 + 2));

    gl.enable_vertex_attrib_array(3);
    gl.vertex_attrib_pointer_f32(
        3,
        4,
        glow::FLOAT,
        false,
        stride,
        4 * (2 + 2 + 4),
    );

    gl.enable_vertex_attrib_array(4);
    gl.vertex_attrib_pointer_f32(
        4,
        1,
        glow::FLOAT,
        false,
        stride,
        4 * (2 + 2 + 4 + 4),
    );

    gl.enable_vertex_attrib_array(5);
    gl.vertex_attrib_pointer_f32(
        5,
        1,
        glow::FLOAT,
        false,
        stride,
        4 * (2 + 2 + 4 + 4 + 1),
    );

    gl.enable_vertex_attrib_array(6);
    gl.vertex_attrib_pointer_f32(
        6,
        2,
        glow::FLOAT,
        false,
        stride,
        4 * (2 + 2 + 4 + 4 + 1 + 1),
    );

    gl.bind_vertex_array(None);
    gl.bind_buffer(glow::ARRAY_BUFFER, None);
    gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);

    (vertex_array, vertex_buffer, index_buffer)
}

/// The vertex of a colored rectangle with a border.
///
/// This type can be directly uploaded to GPU memory.
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vertex {
    /// The position of the [`Vertex`].
    pub position: [f32; 2],

    /// The size of the [`Vertex`].
    pub size: [f32; 2],

    /// The color of the [`Vertex`], in __linear RGB__.
    pub color: [f32; 4],

    /// The border color of the [`Vertex`], in __linear RGB__.
    pub border_color: [f32; 4],

    /// The border radius of the [`Vertex`].
    pub border_radius: f32,

    /// The border width of the [`Vertex`].
    pub border_width: f32,

    /// The __quad__ position of the [`Vertex`].
    pub q_position: [f32; 2],
}

impl Vertex {
    const SIZE: usize = std::mem::size_of::<Self>();

    fn from_quad(quad: &layer::Quad) -> [Vertex; 4] {
        let base = Vertex {
            position: quad.position,
            size: quad.size,
            color: quad.color,
            border_color: quad.color,
            border_radius: quad.border_radius,
            border_width: quad.border_width,
            q_position: [0.0, 0.0],
        };

        [
            base,
            Self {
                q_position: [0.0, 1.0],
                ..base
            },
            Self {
                q_position: [1.0, 0.0],
                ..base
            },
            Self {
                q_position: [1.0, 1.0],
                ..base
            },
        ]
    }
}
