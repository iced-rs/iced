//! This example shows how to use touch events in `Canvas` to draw
//! a circle around each fingertip. This only works on touch-enabled
//! computers like Microsoft Surface.
use iced::mouse;
use iced::touch;
use iced::widget::canvas::event;
use iced::widget::canvas::stroke::{self, Stroke};
use iced::widget::canvas::{self, Canvas, Geometry};
use iced::{Color, Element, Fill, Point, Rectangle, Renderer, Theme};

use std::collections::HashMap;

pub fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application("Multitouch - Iced", Multitouch::update, Multitouch::view)
        .antialiasing(true)
        .centered()
        .run()
}

#[derive(Default)]
struct Multitouch {
    cache: canvas::Cache,
    fingers: HashMap<touch::Finger, Point>,
}

#[derive(Debug)]
enum Message {
    FingerPressed { id: touch::Finger, position: Point },
    FingerLifted { id: touch::Finger },
}

impl Multitouch {
    fn update(&mut self, message: Message) {
        match message {
            Message::FingerPressed { id, position } => {
                self.fingers.insert(id, position);
                self.cache.clear();
            }
            Message::FingerLifted { id } => {
                self.fingers.remove(&id);
                self.cache.clear();
            }
        }
    }

    fn view(&self) -> Element<Message> {
        Canvas::new(self).width(Fill).height(Fill).into()
    }
}

impl canvas::Program<Message> for Multitouch {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: event::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> (event::Status, Option<Message>) {
        match event {
            event::Event::Touch(touch_event) => match touch_event {
                touch::Event::FingerPressed { id, position }
                | touch::Event::FingerMoved { id, position } => (
                    event::Status::Captured,
                    Some(Message::FingerPressed { id, position }),
                ),
                touch::Event::FingerLifted { id, .. }
                | touch::Event::FingerLost { id, .. } => (
                    event::Status::Captured,
                    Some(Message::FingerLifted { id }),
                ),
            },
            _ => (event::Status::Ignored, None),
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let fingerweb = self.cache.draw(renderer, bounds.size(), |frame| {
            if self.fingers.len() < 2 {
                return;
            }

            // Collect tuples of (id, point);
            let mut zones: Vec<(u64, Point)> =
                self.fingers.iter().map(|(id, pt)| (id.0, *pt)).collect();

            // Sort by ID
            zones.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            // Generate sorted list of points
            let vpoints: Vec<(f64, f64)> = zones
                .iter()
                .map(|(_, p)| (f64::from(p.x), f64::from(p.y)))
                .collect();

            let diagram: voronator::VoronoiDiagram<
                voronator::delaunator::Point,
            > = voronator::VoronoiDiagram::from_tuple(
                &(0.0, 0.0),
                &(700.0, 700.0),
                &vpoints,
            )
            .expect("Generate Voronoi diagram");

            for (cell, zone) in diagram.cells().iter().zip(zones) {
                let mut builder = canvas::path::Builder::new();

                for (index, p) in cell.points().iter().enumerate() {
                    let p = Point::new(p.x as f32, p.y as f32);

                    match index {
                        0 => builder.move_to(p),
                        _ => builder.line_to(p),
                    }
                }

                let path = builder.build();

                let color_r = (10 % (zone.0 + 1)) as f32 / 20.0;
                let color_g = (10 % (zone.0 + 8)) as f32 / 20.0;
                let color_b = (10 % (zone.0 + 3)) as f32 / 20.0;

                frame.fill(
                    &path,
                    Color {
                        r: color_r,
                        g: color_g,
                        b: color_b,
                        a: 1.0,
                    },
                );

                frame.stroke(
                    &path,
                    Stroke {
                        style: stroke::Style::Solid(Color::BLACK),
                        width: 3.0,
                        ..Stroke::default()
                    },
                );
            }
        });

        vec![fingerweb]
    }
}
