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
pub struct Scene {
    pub size: f32,
    pub cubes: Vec<Cube>,
    pub camera: Camera,
    pub show_depth_buffer: bool,
    pub light_color: Color,
}

impl Scene {
    pub fn new() -> Self {
        let mut scene = Self {
            size: 0.2,
            cubes: vec![],
            camera: Camera::default(),
            show_depth_buffer: false,
            light_color: Color::WHITE,
        };

        scene.change_amount(MAX);

        scene
    }

    pub fn update(&mut self, time: Duration) {
        for cube in self.cubes.iter_mut() {
            cube.update(self.size, time.as_secs_f32());
        }
    }

    pub fn change_amount(&mut self, amount: u32) {
        let curr_cubes = self.cubes.len() as u32;

        match amount.cmp(&curr_cubes) {
            Ordering::Greater => {
                // spawn
                let cubes_2_spawn = (amount - curr_cubes) as usize;

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
                let cubes_2_cut = curr_cubes - amount;
                let new_len = self.cubes.len() - cubes_2_cut as usize;
                self.cubes.truncate(new_len);
            }
            Ordering::Equal => {}
        }
    }
}

impl<Message> shader::Program<Message> for Scene {
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
