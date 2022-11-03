use crate::program::Version;
use crate::{triangle, Color};
use glow::{Context, HasContext, NativeProgram};
use iced_graphics::Transformation;

#[derive(Debug)]
pub struct Program {
    program: <Context as HasContext>::Program,
    uniform_data: UniformData,
}

#[derive(Debug)]
struct UniformData {
    pub color: Color,
    pub color_location: <Context as HasContext>::UniformLocation,
    pub transform: Transformation,
    pub transform_location: <Context as HasContext>::UniformLocation,
}

impl UniformData {
    fn new(gl: &Context, program: NativeProgram) -> Self {
        Self {
            color: Color::TRANSPARENT,
            color_location: unsafe {
                gl.get_uniform_location(program, "color")
            }
            .expect("Solid - Get color."),
            transform: Transformation::identity(),
            transform_location: unsafe {
                gl.get_uniform_location(program, "u_Transform")
            }
            .expect("Solid - Get u_Transform."),
        }
    }
}

impl Program {
    pub fn new(gl: &Context, shader_version: &Version) -> Self {
        let program = triangle::program(
            gl,
            shader_version,
            include_str!("../shader/common/triangle.frag"),
        );

        Self {
            program,
            uniform_data: UniformData::new(gl, program),
        }
    }

    pub fn write_uniforms(
        &mut self,
        gl: &Context,
        color: &Color,
        transform: &Transformation,
    ) {
        if transform != &self.uniform_data.transform {
            triangle::set_transform(
                gl,
                self.uniform_data.transform_location,
                *transform,
            )
        }

        if color != &self.uniform_data.color {
            let [r, g, b, a] = color.into_linear();

            unsafe {
                gl.uniform_4_f32(
                    Some(&self.uniform_data.color_location),
                    r,
                    g,
                    b,
                    a,
                );
            }

            self.uniform_data.color = *color;
        }
    }

    pub fn use_program(
        &mut self,
        gl: &Context,
        color: &Color,
        transform: &Transformation,
    ) {
        unsafe { gl.use_program(Some(self.program)) }
        self.write_uniforms(gl, color, transform)
    }
}
