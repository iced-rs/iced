use rand::{Rng, thread_rng};
use crate::canvas::{Cursor, FillStyle, Geometry, Gradient};
use iced::widget::canvas::{Cache, Fill, Frame};
use iced::widget::{canvas, Canvas};
use iced::Settings;
use iced::{
    executor, Application, Color, Command, Element, Length, Point, Rectangle,
    Renderer, Size, Theme,
};

fn main() -> iced::Result {
    ModernArt::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

#[derive(Debug, Clone, Copy)]
enum Message {}

struct ModernArt {
    cache: Cache,
}

impl Application for ModernArt {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            ModernArt {
                cache: Default::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Modern Art")
    }

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl<Message> canvas::Program<Message> for ModernArt {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw(bounds.size(), |frame| {
            let num_squares = thread_rng().gen_range(0..1200);

            let mut i = 0;
            while i <= num_squares {
                generate_box(frame, bounds.size());
                i += 1;
            }
        });

        vec![geometry]
    }
}

fn generate_box(frame: &mut Frame, bounds: Size) -> bool {
    let solid = rand::random::<bool>();

    let random_color = || -> Color {
        Color::from_rgb(
            thread_rng().gen_range(0.0..1.0),
            thread_rng().gen_range(0.0..1.0),
            thread_rng().gen_range(0.0..1.0),
        )
    };

    let gradient = |top_left: Point, bottom_right: Point| -> Gradient {
        let mut builder = Gradient::linear(top_left, bottom_right);
        let stops = thread_rng().gen_range(1..10u32);

        let mut i = 0;
        while i <= stops {
            builder = builder.add_stop(
                i as f32 / stops as f32,
                random_color()
            );
            i += 1;
        }

        builder.build().unwrap()
    };

    let top_left = Point::new(
        thread_rng().gen_range(0.0..bounds.width),
        thread_rng().gen_range(0.0..bounds.height)
    );

    let size = Size::new(
        thread_rng().gen_range(50.0..200.0),
        thread_rng().gen_range(50.0..200.0),
    );

    if solid {
        frame.fill_rectangle(
            top_left,
            size,
            Fill {
                style: FillStyle::Solid(random_color()),
                .. Default::default()
            }
        );
    } else {
        frame.fill_rectangle(
            top_left,
            size,
            Fill {
                style: FillStyle::Gradient(&gradient(
                    top_left,
                    Point::new(top_left.x + size.width, top_left.y + size.height)
                )),
                .. Default::default()
            }
        );
    };

    solid
}