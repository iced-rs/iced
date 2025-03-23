use iced::{
    Color, Element, Event, Length, Point, Rectangle, Subscription, Task, Theme,
    mouse::{Cursor, Interaction},
    widget::canvas::{self, Canvas, Frame, Geometry, Path},
};
use ndarray::{Array1, Array2};
use ndarray_rand::{RandomExt, rand_distr::Normal};
use rayon::prelude::*;
use std::f32::consts::PI;

pub const WIDTH: f32 = 1500.0;
pub const HEIGHT: f32 = 1500.0;
const D: usize = 3; // 3D positions (we ignore z for drawing)
const N: usize = 1000;
const DT: f32 = 0.1;

// Type definitions remain the same
#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Zoom(f32), // positive delta zooms in; negative zooms out
    NoOp,
}

/// The main struct, including the particle positions and their energies.
pub struct ParticleLenia {
    particles: Array2<f32>,
    energies: Vec<f32>,
    cache: canvas::Cache,
    // Lenia parameters
    r: f32,
    w: f32,
    m: f32,
    s: f32,
    c_rep: f32,
    kernel_sum: f32,
    zoom: f32,
}

impl ParticleLenia {
    /// Create a new instance with default parameters
    pub fn new() -> Self {
        // Lenia parameters; these may be tuned further.
        let r = 2.0;
        let w = 0.64;
        let m = 0.72;
        let s = 0.26;
        let c_rep = 1.0;
        let kernel_sum = compute_kernel_sum(D, r, w);

        // Initialize particles randomly.
        let particles = Array2::random((N, D), Normal::new(0.0, 1.0).unwrap());
        let energies = vec![0.0; N];

        Self {
            particles,
            energies,
            cache: canvas::Cache::new(),
            r,
            w,
            m,
            s,
            c_rep,
            kernel_sum,
            zoom: 0.33,
        }
    }

    /// Advance one simulation time step.
    fn step(&mut self) {
        let x_prev = self.particles.clone();
        // Copy scalar parameters so the parallel closure doesn't capture &self.
        let kernel_sum = self.kernel_sum;
        let r = self.r;
        let w = self.w;
        let m = self.m;
        let s = self.s;
        let c_rep = self.c_rep;

        // Update particle positions in parallel.
        let updates: Vec<Array1<f32>> = (0..N)
            .into_par_iter()
            .map(|i| {
                let xi = x_prev.row(i).to_owned();
                let grad = numerical_gradient(&x_prev, &xi, kernel_sum, r, w, m, s, c_rep, 1e-4);
                xi - grad * DT
            })
            .collect();

        let new_positions = Array2::from_shape_fn((N, D), |(i, j)| updates[i][j]);
        self.particles.assign(&new_positions);

        // Compute energies for each particle in parallel.
        let new_energies: Vec<f32> = (0..N)
            .into_par_iter()
            .map(|i| {
                let xi = new_positions.row(i).to_owned();
                energy(&new_positions, &xi, kernel_sum, r, w, m, s, c_rep)
            })
            .collect();

        self.energies = new_energies;
    }
}

// Standalone functions for application builder
pub fn update(state: &mut ParticleLenia, message: Message) -> Task<Message> {
    match message {
        Message::Tick => {
            state.step();
            state.cache.clear();
        }
        Message::Zoom(delta) => {
            // Adjust zoom factor (change multiplier as needed)
            state.zoom += 0.01 * delta;
            state.zoom = state.zoom.clamp(0.1, 10.0);
            state.cache.clear();
        }
        Message::NoOp => {}
    }

    Task::none()
}

pub fn view(state: &ParticleLenia) -> Element<Message> {
    Canvas::new(state)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub fn subscription(_state: &ParticleLenia) -> Subscription<Message> {
    Subscription::batch(vec![
        iced::time::every(std::time::Duration::from_millis(16)).map(|_| Message::Tick),
        iced_futures::event::listen().map(|event| {
            if let Event::Mouse(iced::mouse::Event::WheelScrolled { delta }) = event {
                // Use the vertical component of the scroll to adjust zoom.
                let zoom_delta = match delta {
                    iced::mouse::ScrollDelta::Lines { y, .. } => y,
                    iced::mouse::ScrollDelta::Pixels { y, .. } => y,
                };
                Message::Zoom(zoom_delta)
            } else {
                Message::NoOp
            }
        }),
    ])
}

// Main function using the builder pattern
pub fn main() -> iced::Result {
    iced::application("Particle Lenia Simulation", update, view)
        .subscription(subscription)
        .run_with(|| {
            // Initialize the application state and return it with an empty task
            let state = ParticleLenia::new();
            (state, Task::none())
        })
}

// The canvas::Program implementation remains the same
impl<Message> canvas::Program<Message, Theme> for ParticleLenia {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        // Compute min and max energy values to normalize our color mapping.
        let min_energy = self.energies.iter().cloned().fold(f32::INFINITY, f32::min);
        let max_energy = self
            .energies
            .iter()
            .cloned()
            .fold(f32::NEG_INFINITY, f32::max);

        let geometry = self
            .cache
            .draw(renderer, bounds.size(), |frame: &mut Frame| {
                // Draw background.
                frame.fill_rectangle(Point::ORIGIN, frame.size(), Color::BLACK);

                // Define a camera distance (or focal length)
                let camera_distance = 400.0;

                // Render each particle.
                for (i, particle) in self.particles.rows().into_iter().enumerate() {
                    // Get the 3D point.
                    let point = [particle[0], particle[1], particle[2]];

                    // Project the 3D point to 2D.
                    let (proj_x, proj_y) = project_point(&point, camera_distance);

                    // Transform the projected coordinates to canvas space.
                    let x = (bounds.width / 2.0) + proj_x * (bounds.width / 4.0) * self.zoom;
                    let y = (bounds.height / 2.0) + proj_y * (bounds.height / 4.0) * self.zoom;

                    // Normalize energy for color mapping.
                    let energy_value = self.energies[i];
                    let factor = if max_energy - min_energy > 0.0 {
                        (energy_value - min_energy) / (max_energy - min_energy)
                    } else {
                        0.5
                    };

                    // Map factor to a color
                    let circle_color = Color::from_rgb(1.0 - factor, factor, 0.5);

                    // Draw the particle as a circle.
                    let circle = Path::circle(Point::new(x, y), 10.0);
                    frame.fill(&circle, circle_color);
                }
            });

        vec![geometry]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> Interaction {
        Interaction::default()
    }
}

// Helper functions remain the same
/// Projects a 3D point to 2D using a simple perspective projection.
fn project_point(point: &[f32; 3], camera_distance: f32) -> (f32, f32) {
    // Calculate a scaling factor based on the z-value.
    let factor = camera_distance / (camera_distance - point[2]);
    let x_proj = point[0] * factor;
    let y_proj = point[1] * factor;
    (x_proj, y_proj)
}

/// The typical Lenia "bell" function.
fn bell(x: f32, m: f32, s: f32) -> f32 {
    (-((x - m) / s).powi(2)).exp()
}

/// A simple repulsion function.
fn repulse(x: f32) -> f32 {
    (1.0 - x).max(0.0).powi(2)
}

/// Numerically integrates bell(r, w) * surface_area_factor * r^(D-1)
/// over a radius interval, matching the Python reference normalization.
fn compute_kernel_sum(d: usize, r: f32, w: f32) -> f32 {
    let lower = (r - 4.0 * w).max(0.0);
    let upper = r + 4.0 * w;
    let steps = 51;
    let delta = (upper - lower) / (steps - 1) as f32;

    let dimension_factor = match d {
        2 => 2.0 * PI, // for 2D (circumference)
        3 => 4.0 * PI, // for 3D (surface area)
        _ => panic!("compute_kernel_sum: only d=2 or d=3 is implemented."),
    };

    let mut sum = 0.0;
    let mut last_val = None;

    for i in 0..steps {
        let dist = lower + (i as f32) * delta;
        let val = bell(dist, r, w) * dimension_factor * dist.powi((d - 1) as i32);

        if let Some(prev) = last_val {
            sum += 0.5 * (val + prev) * delta; // trapezoidal integration
        }

        last_val = Some(val);
    }

    sum
}

/// Compute the energy for one particle at position `x_i` against all others in `X`.
fn energy(
    #[allow(non_snake_case)] X: &Array2<f32>,
    x_i: &Array1<f32>,
    kernel_sum: f32,
    r: f32,
    w: f32,
    m: f32,
    s: f32,
    c_rep: f32,
) -> f32 {
    let distances = (X - x_i)
        .mapv(|v| v.powi(2))
        .sum_axis(ndarray::Axis(1))
        .mapv(f32::sqrt)
        .mapv(|val| val.max(1e-10));

    let u = distances.mapv(|d| bell(d, r, w)).sum() / kernel_sum;
    let g = bell(u, m, s);
    let r_ener = distances.mapv(repulse).sum() * c_rep / 2.0;

    r_ener - g
}

/// Compute a numerical gradient for `x_i` using finite differences.
fn numerical_gradient(
    #[allow(non_snake_case)] X: &Array2<f32>,
    xi: &Array1<f32>,
    kernel_sum: f32,
    r: f32,
    w: f32,
    m: f32,
    s: f32,
    c_rep: f32,
    h: f32,
) -> Array1<f32> {
    let mut grad = Array1::zeros(D);

    for dim in 0..D {
        let mut x_plus = xi.clone();
        x_plus[dim] += h;
        let f_plus = energy(X, &x_plus, kernel_sum, r, w, m, s, c_rep);

        let mut x_minus = xi.clone();
        x_minus[dim] -= h;
        let f_minus = energy(X, &x_minus, kernel_sum, r, w, m, s, c_rep);

        grad[dim] = (f_plus - f_minus) / (2.0 * h);
    }

    grad
}
