use crate::program::Version;
use crate::triangle::{simple_triangle_program, update_transform};
use crate::Color;
use glow::{Context, HasContext, NativeProgram};
use iced_graphics::Transformation;

#[derive(Debug)]
pub struct SolidProgram {
    pub(crate) program: <Context as HasContext>::Program,
    pub(crate) uniform_data: SolidUniformData,
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

    pub fn set_uniforms<'a>(
        &mut self,
        gl: &Context,
        color: &Color,
        transform: Option<Transformation>,
    ) {
        update_transform(gl, self.program, transform);

        if &self.uniform_data.color != color {
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
}

#[derive(Debug)]
pub(crate) struct SolidUniformData {
    pub color: Color,
    pub color_location: <Context as HasContext>::UniformLocation,
}

impl SolidUniformData {
    fn new(gl: &Context, program: NativeProgram) -> Self {
        Self {
            color: Color::TRANSPARENT,
            color_location: unsafe {
                gl.get_uniform_location(program, "color")
            }
            .expect("Solid - Color uniform location."),
        }
    }
}
