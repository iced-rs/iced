use iced::mouse;
use iced::time::{self, milliseconds};
use iced::widget::canvas;
use iced::{Color, Fill, Font, Point, Rectangle, Renderer, Subscription, Theme, Widget};

use std::cell::RefCell;

pub fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application(TheMatrix::default, TheMatrix::update, TheMatrix::view)
        .subscription(TheMatrix::subscription)
        .run()
}

#[derive(Default)]
struct TheMatrix {
    tick: usize,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick,
}

impl TheMatrix {
    fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                self.tick += 1;
            }
        }
    }

    fn view(&self) -> impl Widget<Message> {
        canvas(self).width(Fill).height(Fill)
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(milliseconds(50)).map(|_| Message::Tick)
    }
}

impl<Message> canvas::Program<Message> for TheMatrix {
    type State = RefCell<Vec<canvas::Cache>>;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        use rand::RngExt;
        use rand::distr::Distribution;

        const CELL_SIZE: f32 = 10.0;

        let mut caches = state.borrow_mut();

        if caches.is_empty() {
            let group = canvas::Group::unique();

            caches.resize_with(30, || canvas::Cache::with_group(group));
        }

        vec![
            caches[self.tick % caches.len()].draw(renderer, bounds.size(), |frame| {
                frame.fill_rectangle(Point::ORIGIN, frame.size(), Color::BLACK);

                let mut rng = rand::rng();
                let rows = (frame.height() / CELL_SIZE).ceil() as usize;
                let columns = (frame.width() / CELL_SIZE).ceil() as usize;

                for row in 0..rows {
                    for column in 0..columns {
                        let position =
                            Point::new(column as f32 * CELL_SIZE, row as f32 * CELL_SIZE);

                        let alphas = [0.05, 0.1, 0.2, 0.5];
                        let weights = [10, 4, 2, 1];
                        let distribution = rand::distr::weighted::WeightedIndex::new(weights)
                            .expect("Create distribution");

                        frame.fill_text(canvas::Text {
                            content: rng.random_range('!'..'z').to_string(),
                            position,
                            color: Color {
                                a: alphas[distribution.sample(&mut rng)],
                                g: 1.0,
                                ..Color::BLACK
                            },
                            size: CELL_SIZE.into(),
                            font: Font::MONOSPACE,
                            ..canvas::Text::default()
                        });
                    }
                }
            }),
        ]
    }
}
