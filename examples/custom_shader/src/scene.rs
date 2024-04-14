mod camera;
mod pipeline;

use camera::Camera;
use pipeline::Pipeline;

use crate::wgpu;
use pipeline::cube::{self, Cube};

use iced::mouse;
use iced::time::Duration;
use iced::widget::shader::{self, Viewport};
use iced::{Color, Rectangle};

use glam::Vec3;
use rand::Rng;
use std::cmp::Ordering;
use std::iter;

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
    type Primitive = Primitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        Primitive::new(
            &self.cubes,
            &self.camera,
            bounds,
            self.show_depth_buffer,
            self.light_color,
        )
    }
}

/// A collection of `Cube`s that can be rendered.
#[derive(Debug)]
pub struct Primitive {
    cubes: Vec<cube::Raw>,
    uniforms: pipeline::Uniforms,
    show_depth_buffer: bool,
}

impl Primitive {
    pub fn new(
        cubes: &[Cube],
        camera: &Camera,
        bounds: Rectangle,
        show_depth_buffer: bool,
        light_color: Color,
    ) -> Self {
        let uniforms = pipeline::Uniforms::new(camera, bounds, light_color);

        Self {
            cubes: cubes
                .iter()
                .map(cube::Raw::from_cube)
                .collect::<Vec<cube::Raw>>(),
            uniforms,
            show_depth_buffer,
        }
    }
}

impl shader::Primitive for Primitive {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        storage: &mut shader::Storage,
        _bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        if !storage.has::<Pipeline>() {
            storage.store(Pipeline::new(
                device,
                queue,
                format,
                viewport.physical_size(),
            ));
        }

        let pipeline = storage.get_mut::<Pipeline>().unwrap();

        // Upload data to GPU
        pipeline.update(
            device,
            queue,
            viewport.physical_size(),
            &self.uniforms,
            self.cubes.len(),
            &self.cubes,
        );
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        // At this point our pipeline should always be initialized
        let pipeline = storage.get::<Pipeline>().unwrap();

        // Render primitive
        pipeline.render(
            target,
            encoder,
            *clip_bounds,
            self.cubes.len() as u32,
            self.show_depth_buffer,
        );
    }
}

fn rnd_origin() -> Vec3 {
    Vec3::new(
        rand::thread_rng().gen_range(-4.0..4.0),
        rand::thread_rng().gen_range(-4.0..4.0),
        rand::thread_rng().gen_range(-4.0..2.0),
    )
}
