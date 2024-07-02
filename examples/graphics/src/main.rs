use iced::widget::canvas::{self, Cache, Canvas, Geometry, Path};
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
                Size::new(100.0, 100.0),
                20.0,
            );
            frame.fill(&rect, Color::from_rgb8(0x12, 0x93, 0xD8));
        });

        vec![geometry]
    }
}
