use iced::border::Radius;
use iced::widget::canvas::{self, Cache, Canvas, Geometry, Path, Text};
use iced::{mouse, Color, Point, Size};
use iced::{Element, Length, Rectangle, Renderer, Theme};

pub fn main() -> iced::Result {
    iced::application(
        "Graphics - minimal drawing example",
        Graphics::update,
        Graphics::view,
    )
    .theme(|_| Theme::Dark)
    .antialiasing(true)
    .run()
}

#[derive(Default)]
struct Graphics {
    cache: Cache,
}

#[derive(Debug, Clone, Copy)]
enum Message {}

impl Graphics {
    fn update(&mut self, _: Message) {
        self.cache.clear();
    }

    fn view(&self) -> Element<Message> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl<Message> canvas::Program<Message> for Graphics {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let rect = Path::rounded_rectangle(
                Point::new(10.0, 10.0),
                Size::new(300.0, 300.0),
                20.0.into(),
            );
            frame.fill(&rect, Color::from_rgb8(0x12, 0x93, 0xD8));

            let rect_bigger_radius = Path::rounded_rectangle(
                Point::new(320.0, 10.0),
                Size::new(300.0, 200.0),
                180.0.into(),
            );
            frame.fill(&rect_bigger_radius, Color::from_rgb8(0x12, 0x93, 0xD8));
            frame.fill_text(Text {
                content: "Rounded rectangle".to_string(),
                position: Point::new(400.0, 90.0),
                ..canvas::Text::default()
            });
            frame.fill_text(Text {
                content: "when radius is bigger than one of the sides".to_string(),
                position: Point::new(330.0, 110.0),
                ..canvas::Text::default()
            });

            let rect_different_radius = Path::rounded_rectangle(
                Point::new(640.0, 10.0),
                Size::new(300.0, 200.0),
                Radius([10.0, 20.0, 30.0, 40.0]),
            );
            frame.fill(&rect_different_radius, Color::from_rgb8(0x12, 0x93, 0xD8));
            frame.fill_text(Text {
                content: "Different radius for each corner".to_string(),
                position: Point::new(690.0, 100.0),
                ..canvas::Text::default()
            });
        });

        vec![geometry]
    }
}
