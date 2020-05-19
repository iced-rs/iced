use crate::{Transformation, Viewport};
use glow::HasContext;
use iced_graphics::layer;
use iced_native::Rectangle;

const MAX_INSTANCES: usize = 100_000;

#[derive(Debug)]
pub struct Pipeline {
    program: <glow::Context as HasContext>::Program,
    vertex_array: <glow::Context as HasContext>::VertexArray,
    instances: <glow::Context as HasContext>::Buffer,
    current_transform: Transformation,
    current_scale: f32,
}

impl Pipeline {
    pub fn new(gl: &glow::Context) -> Pipeline {
        let program = unsafe {
            create_program(
                gl,
                &[
                    (glow::VERTEX_SHADER, include_str!("shader/quad.vert")),
                    (glow::FRAGMENT_SHADER, include_str!("shader/quad.frag")),
                ],
            )
        };

        unsafe {
            gl.use_program(Some(program));

            gl.uniform_matrix_4_f32_slice(
                Some(0),
                false,
                &Transformation::identity().into(),
            );
            gl.uniform_1_f32(Some(1), 1.0);

            gl.use_program(None);
        }

        let (vertex_array, instances) =
            unsafe { create_instance_buffer(gl, MAX_INSTANCES) };

        Pipeline {
            program,
            vertex_array,
            instances,
            current_transform: Transformation::identity(),
            current_scale: 1.0,
        }
    }

    pub fn draw(
        &mut self,
        gl: &glow::Context,
        viewport: &Viewport,
        instances: &[layer::Quad],
        transformation: Transformation,
        scale: f32,
        bounds: Rectangle<u32>,
    ) {
        unsafe {
            gl.enable(glow::SCISSOR_TEST);
            gl.scissor(
                bounds.x as i32,
                (viewport.height()
                    - (bounds.y + bounds.height).min(viewport.height()))
                    as i32,
                bounds.width as i32,
                bounds.height as i32,
            );

            gl.use_program(Some(self.program));
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.instances));
        }

        if transformation != self.current_transform {
            unsafe {
                gl.uniform_matrix_4_f32_slice(
                    Some(0),
                    false,
                    &transformation.into(),
                );

                self.current_transform = transformation;
            }
        }

        if scale != self.current_scale {
            unsafe {
                gl.uniform_1_f32(Some(1), scale);
            }

            self.current_scale = scale;
        }

        let mut i = 0;
        let total = instances.len();

        while i < total {
            let end = (i + MAX_INSTANCES).min(total);
            let amount = end - i;

            unsafe {
                gl.buffer_sub_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    0,
                    bytemuck::cast_slice(&instances[i..end]),
                );

                gl.draw_arrays_instanced(
                    glow::TRIANGLE_STRIP,
                    0,
                    4,
                    amount as i32,
                );
            }

            i += MAX_INSTANCES;
        }

        unsafe {
            gl.bind_vertex_array(None);
            gl.use_program(None);
            gl.disable(glow::SCISSOR_TEST);
        }
    }
}

unsafe fn create_program(
    gl: &glow::Context,
    shader_sources: &[(u32, &str)],
) -> <glow::Context as HasContext>::Program {
    let program = gl.create_program().expect("Cannot create program");

    let mut shaders = Vec::with_capacity(shader_sources.len());

    for (shader_type, shader_source) in shader_sources.iter() {
        let shader = gl
            .create_shader(*shader_type)
            .expect("Cannot create shader");

        gl.shader_source(shader, shader_source);
        gl.compile_shader(shader);

        if !gl.get_shader_compile_status(shader) {
            panic!(gl.get_shader_info_log(shader));
        }

        gl.attach_shader(program, shader);

        shaders.push(shader);
    }

    gl.link_program(program);
    if !gl.get_program_link_status(program) {
        panic!(gl.get_program_info_log(program));
    }

    for shader in shaders {
        gl.detach_shader(program, shader);
        gl.delete_shader(shader);
    }

    program
}

unsafe fn create_instance_buffer(
    gl: &glow::Context,
    size: usize,
) -> (
    <glow::Context as HasContext>::VertexArray,
    <glow::Context as HasContext>::Buffer,
) {
    let vertex_array = gl.create_vertex_array().expect("Create vertex array");
    let buffer = gl.create_buffer().expect("Create instance buffer");

    gl.bind_vertex_array(Some(vertex_array));
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));
    gl.buffer_data_size(
        glow::ARRAY_BUFFER,
        (size * std::mem::size_of::<layer::Quad>()) as i32,
        glow::DYNAMIC_DRAW,
    );

    let stride = std::mem::size_of::<layer::Quad>() as i32;

    gl.enable_vertex_attrib_array(0);
    gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);
    gl.vertex_attrib_divisor(0, 1);

    gl.enable_vertex_attrib_array(1);
    gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, stride, 4 * 2);
    gl.vertex_attrib_divisor(1, 1);

    gl.enable_vertex_attrib_array(2);
    gl.vertex_attrib_pointer_f32(2, 4, glow::FLOAT, false, stride, 4 * (2 + 2));
    gl.vertex_attrib_divisor(2, 1);

    gl.enable_vertex_attrib_array(3);
    gl.vertex_attrib_pointer_f32(
        3,
        4,
        glow::FLOAT,
        false,
        stride,
        4 * (2 + 2 + 4),
    );
    gl.vertex_attrib_divisor(3, 1);

    gl.enable_vertex_attrib_array(4);
    gl.vertex_attrib_pointer_f32(
        4,
        1,
        glow::FLOAT,
        false,
        stride,
        4 * (2 + 2 + 4 + 4),
    );
    gl.vertex_attrib_divisor(4, 1);

    gl.enable_vertex_attrib_array(5);
    gl.vertex_attrib_pointer_f32(
        5,
        1,
        glow::FLOAT,
        false,
        stride,
        4 * (2 + 2 + 4 + 4 + 1),
    );
    gl.vertex_attrib_divisor(5, 1);

    gl.bind_vertex_array(None);
    gl.bind_buffer(glow::ARRAY_BUFFER, None);

    (vertex_array, buffer)
}
