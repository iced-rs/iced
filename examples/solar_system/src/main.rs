//! An animated solar system.
//!
//! This example showcases how to use a `Canvas` widget with transforms to draw
//! using different coordinate systems.
//!
//! Inspired by the example found in the MDN docs[1].
//!
//! [1]: https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API/Tutorial/Basic_animations#An_animated_solar_system
use iced::mouse;
use iced::widget::canvas::stroke::{self, Stroke};
use iced::widget::canvas::{Geometry, Path};
use iced::widget::{canvas, image};
use iced::window;
use iced::{
    Color, Element, Fill, Point, Rectangle, Renderer, Size, Subscription,
    Theme, Vector,
};

use std::time::Instant;

pub fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application(
        "Solar System - Iced",
        SolarSystem::update,
        SolarSystem::view,
    )
    .subscription(SolarSystem::subscription)
    .theme(SolarSystem::theme)
    .run()
}

#[derive(Default)]
struct SolarSystem {
    state: State,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick(Instant),
}

impl SolarSystem {
    fn update(&mut self, message: Message) {
        match message {
            Message::Tick(instant) => {
                self.state.update(instant);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        canvas(&self.state).width(Fill).height(Fill).into()
    }

    fn theme(&self) -> Theme {
        Theme::Moonfly
    }

    fn subscription(&self) -> Subscription<Message> {
        window::frames().map(Message::Tick)
    }
}

#[derive(Debug)]
struct State {
    sun: image::Handle,
    earth: image::Handle,
    moon: image::Handle,
    space_cache: canvas::Cache,
    system_cache: canvas::Cache,
    start: Instant,
    now: Instant,
    stars: Vec<(Point, f32)>,
}

impl State {
    const SUN_RADIUS: f32 = 70.0;
    const ORBIT_RADIUS: f32 = 150.0;
    const EARTH_RADIUS: f32 = 12.0;
    const MOON_RADIUS: f32 = 4.0;
    const MOON_DISTANCE: f32 = 28.0;

    pub fn new() -> State {
        let now = Instant::now();
        let size = window::Settings::default().size;

        State {
            sun: image::Handle::from_bytes(
                include_bytes!("../assets/sun.png").as_slice(),
            ),
            earth: image::Handle::from_bytes(
                include_bytes!("../assets/earth.png").as_slice(),
            ),
            moon: image::Handle::from_bytes(
                include_bytes!("../assets/moon.png").as_slice(),
            ),
            space_cache: canvas::Cache::default(),
            system_cache: canvas::Cache::default(),
            start: now,
            now,
            stars: Self::generate_stars(size.width, size.height),
        }
    }

    pub fn update(&mut self, now: Instant) {
        self.now = now;
        self.system_cache.clear();
    }

    fn generate_stars(width: f32, height: f32) -> Vec<(Point, f32)> {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        (0..100)
            .map(|_| {
                (
                    Point::new(
                        rng.gen_range((-width / 2.0)..(width / 2.0)),
                        rng.gen_range((-height / 2.0)..(height / 2.0)),
                    ),
                    rng.gen_range(0.5..1.0),
                )
            })
            .collect()
    }
}

impl<Message> canvas::Program<Message> for State {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        use std::f32::consts::PI;

        let background =
            self.space_cache.draw(renderer, bounds.size(), |frame| {
                frame.fill_rectangle(Point::ORIGIN, frame.size(), Color::BLACK);

                let stars = Path::new(|path| {
                    for (p, size) in &self.stars {
                        path.rectangle(*p, Size::new(*size, *size));
                    }
                });

                frame.translate(frame.center() - Point::ORIGIN);
                frame.fill(&stars, Color::WHITE);
            });

        let system = self.system_cache.draw(renderer, bounds.size(), |frame| {
            let center = frame.center();
            frame.translate(Vector::new(center.x, center.y));

            frame.draw_image(
                Rectangle::with_radius(Self::SUN_RADIUS),
                &self.sun,
            );

            let orbit = Path::circle(Point::ORIGIN, Self::ORBIT_RADIUS);
            frame.stroke(
                &orbit,
                Stroke {
                    style: stroke::Style::Solid(Color::WHITE.scale_alpha(0.1)),
                    width: 1.0,
                    line_dash: canvas::LineDash {
                        offset: 0,
                        segments: &[3.0, 6.0],
                    },
                    ..Stroke::default()
                },
            );

            let elapsed = self.now - self.start;
            let rotation = (2.0 * PI / 60.0) * elapsed.as_secs() as f32
                + (2.0 * PI / 60_000.0) * elapsed.subsec_millis() as f32;

            frame.rotate(rotation);
            frame.translate(Vector::new(Self::ORBIT_RADIUS, 0.0));

            frame.draw_image(
                Rectangle::with_radius(Self::EARTH_RADIUS),
                canvas::Image::new(&self.earth).rotation(-rotation * 20.0),
            );

            frame.rotate(rotation * 10.0);
            frame.translate(Vector::new(0.0, Self::MOON_DISTANCE));

            frame.draw_image(
                Rectangle::with_radius(Self::MOON_RADIUS),
                &self.moon,
            );
        });

        vec![background, system]
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
