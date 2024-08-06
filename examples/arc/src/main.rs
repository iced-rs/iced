use std::{f32::consts::PI, time::Instant};

use iced::mouse;
use iced::widget::canvas::{
    self, stroke, Cache, Canvas, Geometry, Path, Stroke,
};
use iced::{Element, Fill, Point, Rectangle, Renderer, Subscription, Theme};

pub fn main() -> iced::Result {
    iced::application("Arc - Iced", Arc::update, Arc::view)
        .subscription(Arc::subscription)
        .theme(|_| Theme::Dark)
        .antialiasing(true)
        .run()
}

struct Arc {
    start: Instant,
    cache: Cache,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick,
}

impl Arc {
    fn update(&mut self, _: Message) {
        self.cache.clear();
    }

    fn view(&self) -> Element<Message> {
        Canvas::new(self).width(Fill).height(Fill).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(10))
            .map(|_| Message::Tick)
    }
}

impl Default for Arc {
    fn default() -> Self {
        Arc {
            start: Instant::now(),
            cache: Cache::default(),
        }
    }
}

impl<Message> canvas::Program<Message> for Arc {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let palette = theme.palette();

            let center = frame.center();
            let radius = frame.width().min(frame.height()) / 5.0;

            let start = Point::new(center.x, center.y - radius);

            let angle = (self.start.elapsed().as_millis() % 10_000) as f32
                / 10_000.0
                * 2.0
                * PI;

            let end = Point::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            );

            let circles = Path::new(|b| {
                b.circle(start, 10.0);
                b.move_to(end);
                b.circle(end, 10.0);
            });

            frame.fill(&circles, palette.text);

            let path = Path::new(|b| {
                b.move_to(start);
                b.arc_to(center, end, 50.0);
                b.line_to(end);
            });

            frame.stroke(
                &path,
                Stroke {
                    style: stroke::Style::Solid(palette.text),
                    width: 10.0,
                    ..Stroke::default()
                },
            );
        });

        vec![geometry]
    }
}
