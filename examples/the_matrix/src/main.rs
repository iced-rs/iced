use iced::mouse;
use iced::time::{self, Instant};
use iced::widget::canvas;
use iced::widget::canvas::{Cache, Geometry};
use iced::{
    Color, Element, Font, Length, Point, Rectangle, Renderer, Subscription,
    Theme,
};

pub fn main() -> iced::Result {
    iced::program("The Matrix - Iced", TheMatrix::update, TheMatrix::view)
        .subscription(TheMatrix::subscription)
        .antialiasing(true)
        .run()
}

struct TheMatrix {
    ticks: usize,
    backgrounds: Vec<Cache>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick(Instant),
}

impl TheMatrix {
    fn update(&mut self, message: Message) {
        match message {
            Message::Tick(_now) => {
                self.ticks += 1;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        canvas(self as &Self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(std::time::Duration::from_millis(50)).map(Message::Tick)
    }
}

impl Default for TheMatrix {
    fn default() -> Self {
        let mut backgrounds = Vec::with_capacity(30);
        backgrounds.resize_with(30, Cache::default);

        Self {
            ticks: 0,
            backgrounds,
        }
    }
}

impl<Message> canvas::Program<Message> for TheMatrix {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        use rand::distributions::Distribution;
        use rand::Rng;

        const CELL_SIZE: f32 = 10.0;

        vec![self.backgrounds[self.ticks % self.backgrounds.len()].draw(
            renderer,
            bounds.size(),
            |frame| {
                frame.fill_rectangle(Point::ORIGIN, frame.size(), Color::BLACK);

                let mut rng = rand::thread_rng();
                let rows = (frame.height() / CELL_SIZE).ceil() as usize;
                let columns = (frame.width() / CELL_SIZE).ceil() as usize;

                for row in 0..rows {
                    for column in 0..columns {
                        let position = Point::new(
                            column as f32 * CELL_SIZE,
                            row as f32 * CELL_SIZE,
                        );

                        let alphas = [0.05, 0.1, 0.2, 0.5];
                        let weights = [10, 4, 2, 1];
                        let distribution =
                            rand::distributions::WeightedIndex::new(weights)
                                .expect("Create distribution");

                        frame.fill_text(canvas::Text {
                            content: rng.gen_range('!'..'z').to_string(),
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
            },
        )]
    }
}
