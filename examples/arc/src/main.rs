use std::{f32::consts::PI, time::Instant};

use iced::mouse;
use iced::widget::canvas::{
    self, Cache, Canvas, Geometry, Path, Stroke, stroke,
};
use iced::window;
use iced::{Element, Fill, Point, Rectangle, Renderer, Subscription, Theme};

pub fn main() -> iced::Result {
    iced::application(Arc::new, Arc::update, Arc::view)
        .subscription(Arc::subscription)
        .theme(|_| Theme::Dark)
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
    fn new() -> Self {
        Arc {
            start: Instant::now(),
            cache: Cache::default(),
        }
    }

    fn update(&mut self, _: Message) {
        self.cache.clear();
    }

    fn view(&self) -> Element<Message> {
        Canvas::new(self).width(Fill).height(Fill).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        window::frames().map(|_| Message::Tick)
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
