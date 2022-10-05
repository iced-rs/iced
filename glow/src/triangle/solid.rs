use crate::program::Version;
use crate::triangle::{set_transform, simple_triangle_program};
use crate::Color;
use glow::{Context, HasContext, NativeProgram};
use iced_graphics::Transformation;

#[derive(Debug)]
pub struct SolidProgram {
    program: <Context as HasContext>::Program,
    uniform_data: SolidUniformData,
}

#[derive(Debug)]
pub(crate) struct SolidUniformData {
    pub color: Color,
    pub color_location: <Context as HasContext>::UniformLocation,
    pub transform: Transformation,
    pub transform_location: <Context as HasContext>::UniformLocation,
}

impl SolidUniformData {
    fn new(gl: &Context, program: NativeProgram) -> Self {
        Self {
            color: Color::TRANSPARENT,
            color_location: unsafe {
                gl.get_uniform_location(program, "color")
            }
                .expect("Solid - Color uniform location."),
            transform: Transformation::identity(),
            transform_location: unsafe {
                gl.get_uniform_location(program, "u_Transform")
            }
                .expect("Get transform location."),
        }
    }
}

impl SolidProgram {
    pub fn new(gl: &Context, shader_version: &Version) -> Self {
        let program = simple_triangle_program(
            gl,
            shader_version,
            include_str!("../shader/common/triangle.frag"),
        );

        Self {
            program,
            uniform_data: SolidUniformData::new(gl, program),
        }
    }

    pub fn write_uniforms(
        &mut self,
        gl: &Context,
        color: &Color,
        transform: &Transformation,
    ) {
        if transform != &self.uniform_data.transform {
            set_transform(gl, self.uniform_data.transform_location, *transform)
        }

        if color != &self.uniform_data.color {
            unsafe {
                gl.uniform_4_f32(
                    Some(&self.uniform_data.color_location),
                    color.r,
                    color.g,
                    color.b,
                    color.a,
                );
            }

            self.uniform_data.color = *color;
        }
    }

    pub fn use_program(&mut self, gl: &glow::Context, color: &Color, transform: &Transformation) {
        unsafe {
            gl.use_program(Some(self.program))
        }
        self.write_uniforms(gl, color, transform)
    }
}