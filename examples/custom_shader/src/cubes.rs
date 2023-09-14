use crate::camera::Camera;
use crate::primitive;
use crate::primitive::cube::Cube;
use glam::Vec3;
use iced::widget::shader;
use iced::{mouse, Color, Rectangle};
use rand::Rng;
use std::cmp::Ordering;
use std::iter;
use std::time::Duration;

pub const MAX: u32 = 500;

#[derive(Clone)]
pub struct Cubes {
    pub size: f32,
    pub cubes: Vec<Cube>,
    pub camera: Camera,
    pub show_depth_buffer: bool,
    pub light_color: Color,
}

impl Cubes {
    pub fn new() -> Self {
        let mut cubes = Self {
            size: 0.2,
            cubes: vec![],
            camera: Camera::default(),
            show_depth_buffer: false,
            light_color: Color::WHITE,
        };

        cubes.adjust_num_cubes(MAX);

        cubes
    }

    pub fn update(&mut self, time: Duration) {
        for cube in self.cubes.iter_mut() {
            cube.update(self.size, time.as_secs_f32());
        }
    }

    pub fn adjust_num_cubes(&mut self, num_cubes: u32) {
        let curr_cubes = self.cubes.len() as u32;

        match num_cubes.cmp(&curr_cubes) {
            Ordering::Greater => {
                // spawn
                let cubes_2_spawn = (num_cubes - curr_cubes) as usize;

                let mut cubes = 0;
                self.cubes.extend(iter::from_fn(|| {
                    if cubes < cubes_2_spawn {
                        cubes += 1;
                        Some(Cube::new(self.size, rnd_origin()))
                    } else {
                        None
                    }
                }));
            }
            Ordering::Less => {
                // chop
                let cubes_2_cut = curr_cubes - num_cubes;
                let new_len = self.cubes.len() - cubes_2_cut as usize;
                self.cubes.truncate(new_len);
            }
            _ => {}
        }
    }
}

impl<Message> shader::Program<Message> for Cubes {
    type State = ();
    type Primitive = primitive::Primitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        primitive::Primitive::new(
            &self.cubes,
            &self.camera,
            bounds,
            self.show_depth_buffer,
            self.light_color,
        )
    }
}

fn rnd_origin() -> Vec3 {
    Vec3::new(
        rand::thread_rng().gen_range(-4.0..4.0),
        rand::thread_rng().gen_range(-4.0..4.0),
        rand::thread_rng().gen_range(-4.0..2.0),
    )
}
