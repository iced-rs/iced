//! An animated solar system.
//!
//! This example showcases how to use a `Canvas` widget with transforms to draw
//! using different coordinate systems.
//!
//! Inspired by the example found in the MDN docs[1].
//!
//! [1]: https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API/Tutorial/Basic_animations#An_animated_solar_system
use iced::application;
use iced::executor;
use iced::theme::{self, Theme};
use iced::time;
use iced::widget::canvas;
use iced::widget::canvas::gradient::{self, Gradient};
use iced::widget::canvas::stroke::{self, Stroke};
use iced::widget::canvas::{Cursor, Path};
use iced::window;
use iced::{
    Application, Color, Command, Element, Length, Point, Rectangle, Settings,
    Size, Subscription, Vector,
};

use std::time::Instant;

pub fn main() -> iced::Result {
    SolarSystem::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

struct SolarSystem {
    state: State,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick(Instant),
}

impl Application for SolarSystem {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            SolarSystem {
                state: State::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Solar system - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick(instant) => {
                self.state.update(instant);
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        canvas(&self.state)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn style(&self) -> theme::Application {
        fn dark_background(_theme: &Theme) -> application::Appearance {
            application::Appearance {
                background_color: Color::BLACK,
                text_color: Color::WHITE,
            }
        }

        theme::Application::from(dark_background as fn(&Theme) -> _)
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(time::Duration::from_millis(10)).map(Message::Tick)
    }
}

#[derive(Debug)]
struct State {
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
        let (width, height) = window::Settings::default().size;

        State {
            space_cache: Default::default(),
            system_cache: Default::default(),
            start: now,
            now,
            stars: Self::generate_stars(width, height),
        }
    }

    pub fn update(&mut self, now: Instant) {
        self.now = now;
        self.system_cache.clear();
    }

    fn generate_stars(width: u32, height: u32) -> Vec<(Point, f32)> {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        (0..100)
            .map(|_| {
                (
                    Point::new(
                        rng.gen_range(
                            (-(width as f32) / 2.0)..(width as f32 / 2.0),
                        ),
                        rng.gen_range(
                            (-(height as f32) / 2.0)..(height as f32 / 2.0),
                        ),
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
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<canvas::Geometry> {
        use std::f32::consts::PI;

        let background = self.space_cache.draw(bounds.size(), |frame| {
            let stars = Path::new(|path| {
                for (p, size) in &self.stars {
                    path.rectangle(*p, Size::new(*size, *size));
                }
            });

            frame.translate(frame.center() - Point::ORIGIN);
            frame.fill(&stars, Color::WHITE);
        });

        let system = self.system_cache.draw(bounds.size(), |frame| {
            let center = frame.center();

            let sun = Path::circle(center, Self::SUN_RADIUS);
            let orbit = Path::circle(center, Self::ORBIT_RADIUS);

            frame.fill(&sun, Color::from_rgb8(0xF9, 0xD7, 0x1C));
            frame.stroke(
                &orbit,
                Stroke {
                    style: stroke::Style::Solid(Color::from_rgba8(
                        0, 153, 255, 0.1,
                    )),
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

            frame.with_save(|frame| {
                frame.translate(Vector::new(center.x, center.y));
                frame.rotate(rotation);
                frame.translate(Vector::new(Self::ORBIT_RADIUS, 0.0));

                let earth = Path::circle(Point::ORIGIN, Self::EARTH_RADIUS);

                let earth_fill =
                    Gradient::linear(gradient::Position::Absolute {
                        start: Point::new(-Self::EARTH_RADIUS, 0.0),
                        end: Point::new(Self::EARTH_RADIUS, 0.0),
                    })
                    .add_stop(0.2, Color::from_rgb(0.15, 0.50, 1.0))
                    .add_stop(0.8, Color::from_rgb(0.0, 0.20, 0.47))
                    .build()
                    .expect("Build Earth fill gradient");

                frame.fill(&earth, earth_fill);

                frame.with_save(|frame| {
                    frame.rotate(rotation * 10.0);
                    frame.translate(Vector::new(0.0, Self::MOON_DISTANCE));

                    let moon = Path::circle(Point::ORIGIN, Self::MOON_RADIUS);
                    frame.fill(&moon, Color::WHITE);
                });
            });
        });

        vec![background, system]
    }
}
