use iced::mouse;
use iced::widget::canvas::event::{self, Event};
use iced::widget::canvas::{self, Canvas, Geometry};
use iced::widget::{column, row, slider, text};
use iced::{Center, Color, Fill, Point, Rectangle, Renderer, Size, Theme};

use rand::Rng;
use std::fmt::Debug;

fn main() -> iced::Result {
    iced::application(
        "Sierpinski Triangle - Iced",
        SierpinskiEmulator::update,
        SierpinskiEmulator::view,
    )
    .antialiasing(true)
    .run()
}

#[derive(Debug, Default)]
struct SierpinskiEmulator {
    graph: SierpinskiGraph,
}

#[derive(Debug, Clone)]
pub enum Message {
    IterationSet(i32),
    PointAdded(Point),
    PointRemoved,
}

impl SierpinskiEmulator {
    fn update(&mut self, message: Message) {
        match message {
            Message::IterationSet(cur_iter) => {
                self.graph.iteration = cur_iter;
            }
            Message::PointAdded(point) => {
                self.graph.fix_points.push(point);
                self.graph.random_points.clear();
            }
            Message::PointRemoved => {
                self.graph.fix_points.pop();
                self.graph.random_points.clear();
            }
        }

        self.graph.redraw();
    }

    fn view(&self) -> iced::Element<'_, Message> {
        column![
            Canvas::new(&self.graph).width(Fill).height(Fill),
            row![
                text!("Iteration: {:?}", self.graph.iteration),
                slider(0..=10000, self.graph.iteration, Message::IterationSet)
            ]
            .padding(10)
            .spacing(20),
        ]
        .align_x(Center)
        .into()
    }
}

#[derive(Default, Debug)]
struct SierpinskiGraph {
    iteration: i32,
    fix_points: Vec<Point>,
    random_points: Vec<Point>,
    cache: canvas::Cache,
}

impl canvas::Program<Message> for SierpinskiGraph {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (event::Status, Option<Message>) {
        let Some(cursor_position) = cursor.position_in(bounds) else {
            return (event::Status::Ignored, None);
        };

        match event {
            Event::Mouse(mouse_event) => {
                let message = match mouse_event {
                    iced::mouse::Event::ButtonPressed(
                        iced::mouse::Button::Left,
                    ) => Some(Message::PointAdded(cursor_position)),
                    iced::mouse::Event::ButtonPressed(
                        iced::mouse::Button::Right,
                    ) => Some(Message::PointRemoved),
                    _ => None,
                };
                (event::Status::Captured, message)
            }
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
        let geom = self.cache.draw(renderer, bounds.size(), |frame| {
            frame.stroke(
                &canvas::Path::rectangle(Point::ORIGIN, frame.size()),
                canvas::Stroke::default(),
            );

            if self.fix_points.is_empty() {
                return;
            }

            let mut last = None;

            for _ in 0..self.iteration {
                let p = self.gen_rand_point(last);
                let path = canvas::Path::rectangle(p, Size::new(1_f32, 1_f32));

                frame.stroke(&path, canvas::Stroke::default());

                last = Some(p);
            }

            self.fix_points.iter().for_each(|p| {
                let path = canvas::Path::circle(*p, 5.0);
                frame.fill(&path, Color::from_rgb8(0x12, 0x93, 0xD8));
            });
        });

        vec![geom]
    }
}

impl SierpinskiGraph {
    fn redraw(&mut self) {
        self.cache.clear();
    }

    fn gen_rand_point(&self, last: Option<Point>) -> Point {
        let dest_point_idx =
            rand::thread_rng().gen_range(0..self.fix_points.len());

        let dest_point = self.fix_points[dest_point_idx];
        let cur_point = last.or_else(|| Some(self.fix_points[0])).unwrap();

        Point::new(
            (dest_point.x + cur_point.x) / 2_f32,
            (dest_point.y + cur_point.y) / 2_f32,
        )
    }
}
