use crate::program::Version;
use crate::triangle::{simple_triangle_program, update_transform};
use glow::{Context, HasContext, NativeProgram};
use iced_graphics::gradient::Gradient;
use iced_graphics::widget::canvas::gradient::Linear;
use iced_graphics::Transformation;

#[derive(Debug)]
pub(super) struct GradientProgram {
    pub(super) program: <Context as HasContext>::Program,
    pub(super) uniform_data: GradientUniformData,
}

impl GradientProgram {
    pub(super) fn new(gl: &Context, shader_version: &Version) -> Self {
        let program = simple_triangle_program(
            gl,
            shader_version,
            include_str!("../shader/common/gradient.frag"),
        );

        Self {
            program,
            uniform_data: GradientUniformData::new(gl, program),
        }
    }

    pub(super) fn set_uniforms<'a>(
        &mut self,
        gl: &Context,
        gradient: &Gradient,
        transform: Option<Transformation>,
    ) {
        update_transform(gl, self.program, transform);

        if &self.uniform_data.current_gradient != gradient {
            match gradient {
                Gradient::Linear(linear) => {
                    let gradient_start: [f32; 2] = (linear.start).into();
                    let gradient_end: [f32; 2] = (linear.end).into();

                    unsafe {
                        gl.uniform_2_f32(
                            Some(
                                &self
                                    .uniform_data
                                    .uniform_locations
                                    .gradient_start_location,
                            ),
                            gradient_start[0],
                            gradient_start[1],
                        );

                        gl.uniform_2_f32(
                            Some(
                                &self
                                    .uniform_data
                                    .uniform_locations
                                    .gradient_end_location,
                            ),
                            gradient_end[0],
                            gradient_end[1],
                        );

                        gl.uniform_1_u32(
                            Some(
                                &self
                                    .uniform_data
                                    .uniform_locations
                                    .color_stops_size_location,
                            ),
                            linear.color_stops.len() as u32,
                        );

                        for (index, stop) in
                            linear.color_stops.iter().enumerate()
                        {
                            gl.uniform_1_f32(
                                Some(
                                    &self
                                        .uniform_data
                                        .uniform_locations
                                        .color_stops_locations[index]
                                        .offset,
                                ),
                                stop.offset,
                            );

                            gl.uniform_4_f32(
                                Some(
                                    &self
                                        .uniform_data
                                        .uniform_locations
                                        .color_stops_locations[index]
                                        .color,
                                ),
                                stop.color.r,
                                stop.color.g,
                                stop.color.b,
                                stop.color.a,
                            );
                        }
                    }
                }
            }

            self.uniform_data.current_gradient = gradient.clone();
        }
    }
}

#[derive(Debug)]
pub(super) struct GradientUniformData {
    current_gradient: Gradient,
    uniform_locations: GradientUniformLocations,
}

#[derive(Debug)]
struct GradientUniformLocations {
    gradient_start_location: <Context as HasContext>::UniformLocation,
    gradient_end_location: <Context as HasContext>::UniformLocation,
    color_stops_size_location: <Context as HasContext>::UniformLocation,
    //currently the maximum number of stops is 64 due to needing to allocate the
    //memory for the array of stops with a const value in GLSL
    color_stops_locations: [ColorStopLocation; 64],
}

#[derive(Copy, Debug, Clone)]
struct ColorStopLocation {
    color: <Context as HasContext>::UniformLocation,
    offset: <Context as HasContext>::UniformLocation,
}

impl GradientUniformData {
    fn new(gl: &Context, program: NativeProgram) -> Self {
        let gradient_start_location =
            unsafe { gl.get_uniform_location(program, "gradient_start") }
                .expect("Gradient - Get gradient_start.");

        let gradient_end_location =
            unsafe { gl.get_uniform_location(program, "gradient_end") }
                .expect("Gradient - Get gradient_end.");

        let color_stops_size_location =
            unsafe { gl.get_uniform_location(program, "color_stops_size") }
                .expect("Gradient - Get color_stops_size.");

        let color_stops_locations: [ColorStopLocation; 64] =
            core::array::from_fn(|index| {
                let offset = unsafe {
                    gl.get_uniform_location(
                        program,
                        &format!("color_stop_offsets[{}]", index),
                    )
                }
                .expect(&format!(
                    "Gradient - Color stop offset with index {}",
                    index
                ));

                let color = unsafe {
                    gl.get_uniform_location(
                        program,
                        &format!("color_stop_colors[{}]", index),
                    )
                }
                .expect(&format!(
                    "Gradient - Color stop colors with index {}",
                    index
                ));

                ColorStopLocation { color, offset }
            });

        GradientUniformData {
            current_gradient: Gradient::Linear(Linear {
                start: Default::default(),
                end: Default::default(),
                color_stops: vec![],
            }),
            uniform_locations: GradientUniformLocations {
                gradient_start_location,
                gradient_end_location,
                color_stops_size_location,
                color_stops_locations,
            },
        }
    }
}
