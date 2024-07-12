//! This example showcases a drawing a quad.
mod quad {
    use iced::advanced::layout::{self, Layout};
    use iced::advanced::renderer;
    use iced::advanced::widget::{self, Widget};
    use iced::border;
    use iced::mouse;
    use iced::{Border, Color, Element, Length, Rectangle, Shadow, Size};

    pub struct CustomQuad {
        size: f32,
        radius: border::Radius,
        border_width: f32,
        shadow: Shadow,
    }

    impl CustomQuad {
        pub fn new(
            size: f32,
            radius: border::Radius,
            border_width: f32,
            shadow: Shadow,
        ) -> Self {
            Self {
                size,
                radius,
                border_width,
                shadow,
            }
        }
    }

    impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for CustomQuad
    where
        Renderer: renderer::Renderer,
    {
        fn size(&self) -> Size<Length> {
            Size {
                width: Length::Shrink,
                height: Length::Shrink,
            }
        }

        fn layout(
            &self,
            _tree: &mut widget::Tree,
            _renderer: &Renderer,
            _limits: &layout::Limits,
        ) -> layout::Node {
            layout::Node::new(Size::new(self.size, self.size))
        }

        fn draw(
            &self,
            _state: &widget::Tree,
            renderer: &mut Renderer,
            _theme: &Theme,
            _style: &renderer::Style,
            layout: Layout<'_>,
            _cursor: mouse::Cursor,
            _viewport: &Rectangle,
        ) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border: Border {
                        radius: self.radius,
                        width: self.border_width,
                        color: Color::from_rgb(1.0, 0.0, 0.0),
                    },
                    shadow: self.shadow,
                },
                Color::BLACK,
            );
        }
    }

    impl<'a, Message> From<CustomQuad> for Element<'a, Message> {
        fn from(circle: CustomQuad) -> Self {
            Self::new(circle)
        }
    }
}

use iced::border;
use iced::widget::{center, column, slider, text};
use iced::{Center, Color, Element, Shadow, Vector};

pub fn main() -> iced::Result {
    iced::run("Custom Quad - Iced", Example::update, Example::view)
}

struct Example {
    radius: border::Radius,
    border_width: f32,
    shadow: Shadow,
}

#[derive(Debug, Clone, Copy)]
#[allow(clippy::enum_variant_names)]
enum Message {
    RadiusTopLeftChanged(f32),
    RadiusTopRightChanged(f32),
    RadiusBottomRightChanged(f32),
    RadiusBottomLeftChanged(f32),
    BorderWidthChanged(f32),
    ShadowXOffsetChanged(f32),
    ShadowYOffsetChanged(f32),
    ShadowBlurRadiusChanged(f32),
}

impl Example {
    fn new() -> Self {
        Self {
            radius: border::radius(50),
            border_width: 0.0,
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.8),
                offset: Vector::new(0.0, 8.0),
                blur_radius: 16.0,
            },
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::RadiusTopLeftChanged(radius) => {
                self.radius = self.radius.top_left(radius);
            }
            Message::RadiusTopRightChanged(radius) => {
                self.radius = self.radius.top_right(radius);
            }
            Message::RadiusBottomRightChanged(radius) => {
                self.radius = self.radius.bottom_right(radius);
            }
            Message::RadiusBottomLeftChanged(radius) => {
                self.radius = self.radius.bottom_left(radius);
            }
            Message::BorderWidthChanged(width) => {
                self.border_width = width;
            }
            Message::ShadowXOffsetChanged(x) => {
                self.shadow.offset.x = x;
            }
            Message::ShadowYOffsetChanged(y) => {
                self.shadow.offset.y = y;
            }
            Message::ShadowBlurRadiusChanged(s) => {
                self.shadow.blur_radius = s;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let border::Radius {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        } = self.radius;

        let Shadow {
            offset: Vector { x: sx, y: sy },
            blur_radius: sr,
            ..
        } = self.shadow;

        let content = column![
            quad::CustomQuad::new(
                200.0,
                self.radius,
                self.border_width,
                self.shadow
            ),
            text!("Radius: {top_left:.2}/{top_right:.2}/{bottom_right:.2}/{bottom_left:.2}"),
            slider(1.0..=100.0, top_left, Message::RadiusTopLeftChanged).step(0.01),
            slider(1.0..=100.0, top_right, Message::RadiusTopRightChanged).step(0.01),
            slider(1.0..=100.0, bottom_right, Message::RadiusBottomRightChanged)
                .step(0.01),
            slider(1.0..=100.0, bottom_left, Message::RadiusBottomLeftChanged)
                .step(0.01),
            slider(1.0..=10.0, self.border_width, Message::BorderWidthChanged)
                .step(0.01),
            text!("Shadow: {sx:.2}x{sy:.2}, {sr:.2}"),
            slider(-100.0..=100.0, sx, Message::ShadowXOffsetChanged)
                .step(0.01),
            slider(-100.0..=100.0, sy, Message::ShadowYOffsetChanged)
                .step(0.01),
            slider(0.0..=100.0, sr, Message::ShadowBlurRadiusChanged)
                .step(0.01),
        ]
        .padding(20)
        .spacing(20)
        .max_width(500)
        .align_x(Center);

        center(content).into()
    }
}

impl Default for Example {
    fn default() -> Self {
        Self::new()
    }
}
