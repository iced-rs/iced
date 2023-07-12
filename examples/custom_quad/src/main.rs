//! This example showcases a drawing a quad.
mod quad {
    use iced::advanced::layout::{self, Layout};
    use iced::advanced::renderer;
    use iced::advanced::widget::{self, Widget};
    use iced::mouse;
    use iced::{Color, Element, Length, Rectangle, Size};

    pub struct CustomQuad {
        size: f32,
        radius: [f32; 4],
        border_width: f32,
    }

    impl CustomQuad {
        pub fn new(size: f32, radius: [f32; 4], border_width: f32) -> Self {
            Self {
                size,
                radius,
                border_width,
            }
        }
    }

    impl<Message, Renderer> Widget<Message, Renderer> for CustomQuad
    where
        Renderer: renderer::Renderer,
    {
        fn width(&self) -> Length {
            Length::Shrink
        }

        fn height(&self) -> Length {
            Length::Shrink
        }

        fn layout(
            &self,
            _renderer: &Renderer,
            _limits: &layout::Limits,
        ) -> layout::Node {
            layout::Node::new(Size::new(self.size, self.size))
        }

        fn draw(
            &self,
            _state: &widget::Tree,
            renderer: &mut Renderer,
            _theme: &Renderer::Theme,
            _style: &renderer::Style,
            layout: Layout<'_>,
            _cursor: mouse::Cursor,
            _viewport: &Rectangle,
        ) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border_radius: self.radius.into(),
                    border_width: self.border_width,
                    border_color: Color::from_rgb(1.0, 0.0, 0.0),
                },
                Color::BLACK,
            );
        }
    }

    impl<'a, Message, Renderer> From<CustomQuad> for Element<'a, Message, Renderer>
    where
        Renderer: renderer::Renderer,
    {
        fn from(circle: CustomQuad) -> Self {
            Self::new(circle)
        }
    }
}

use iced::widget::{column, container, slider, text};
use iced::{Alignment, Element, Length, Sandbox, Settings};

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

struct Example {
    radius: [f32; 4],
    border_width: f32,
}

#[derive(Debug, Clone, Copy)]
#[allow(clippy::enum_variant_names)]
enum Message {
    RadiusTopLeftChanged(f32),
    RadiusTopRightChanged(f32),
    RadiusBottomRightChanged(f32),
    RadiusBottomLeftChanged(f32),
    BorderWidthChanged(f32),
}

impl Sandbox for Example {
    type Message = Message;

    fn new() -> Self {
        Self {
            radius: [50.0; 4],
            border_width: 0.0,
        }
    }

    fn title(&self) -> String {
        String::from("Custom widget - Iced")
    }

    fn update(&mut self, message: Message) {
        let [tl, tr, br, bl] = self.radius;
        match message {
            Message::RadiusTopLeftChanged(radius) => {
                self.radius = [radius, tr, br, bl];
            }
            Message::RadiusTopRightChanged(radius) => {
                self.radius = [tl, radius, br, bl];
            }
            Message::RadiusBottomRightChanged(radius) => {
                self.radius = [tl, tr, radius, bl];
            }
            Message::RadiusBottomLeftChanged(radius) => {
                self.radius = [tl, tr, br, radius];
            }
            Message::BorderWidthChanged(width) => {
                self.border_width = width;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let [tl, tr, br, bl] = self.radius;

        let content = column![
            quad::CustomQuad::new(200.0, self.radius, self.border_width),
            text(format!("Radius: {tl:.2}/{tr:.2}/{br:.2}/{bl:.2}")),
            slider(1.0..=100.0, tl, Message::RadiusTopLeftChanged).step(0.01),
            slider(1.0..=100.0, tr, Message::RadiusTopRightChanged).step(0.01),
            slider(1.0..=100.0, br, Message::RadiusBottomRightChanged)
                .step(0.01),
            slider(1.0..=100.0, bl, Message::RadiusBottomLeftChanged)
                .step(0.01),
            slider(1.0..=10.0, self.border_width, Message::BorderWidthChanged)
                .step(0.01),
        ]
        .padding(20)
        .spacing(20)
        .max_width(500)
        .align_items(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
