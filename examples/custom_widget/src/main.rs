//! This example showcases a simple native custom widget that draws a circle.
mod circle {
    // For now, to implement a custom native widget you will need to add
    // `iced_native` and `iced_wgpu` to your dependencies.
    //
    // Then, you simply need to define your widget type and implement the
    // `iced_native::Widget` trait with the `iced_wgpu::Renderer`.
    //
    // Of course, you can choose to make the implementation renderer-agnostic,
    // if you wish to, by creating your own `Renderer` trait, which could be
    // implemented by `iced_wgpu` and other renderers.
    use iced_native::layout::{self, Layout};
    use iced_native::renderer;
    use iced_native::{
        Color, Element, Hasher, Length, Point, Rectangle, Size, Widget,
    };

    pub struct Circle {
        radius: f32,
    }

    impl Circle {
        pub fn new(radius: f32) -> Self {
            Self { radius }
        }
    }

    impl<Message, Renderer> Widget<Message, Renderer> for Circle
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
            layout::Node::new(Size::new(self.radius * 2.0, self.radius * 2.0))
        }

        fn hash_layout(&self, state: &mut Hasher) {
            use std::hash::Hash;

            self.radius.to_bits().hash(state);
        }

        fn draw(
            &self,
            renderer: &mut Renderer,
            _style: &renderer::Style,
            layout: Layout<'_>,
            _cursor_position: Point,
            _viewport: &Rectangle,
        ) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border_radius: self.radius,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
                Color::BLACK,
            );
        }
    }

    impl<'a, Message, Renderer> Into<Element<'a, Message, Renderer>> for Circle
    where
        Renderer: renderer::Renderer,
    {
        fn into(self) -> Element<'a, Message, Renderer> {
            Element::new(self)
        }
    }
}

use circle::Circle;
use iced::{
    slider, Alignment, Column, Container, Element, Length, Sandbox, Settings,
    Slider, Text,
};

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

struct Example {
    radius: f32,
    slider: slider::State,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    RadiusChanged(f32),
}

impl Sandbox for Example {
    type Message = Message;

    fn new() -> Self {
        Example {
            radius: 50.0,
            slider: slider::State::new(),
        }
    }

    fn title(&self) -> String {
        String::from("Custom widget - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::RadiusChanged(radius) => {
                self.radius = radius;
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let content = Column::new()
            .padding(20)
            .spacing(20)
            .max_width(500)
            .align_items(Alignment::Center)
            .push(Circle::new(self.radius))
            .push(Text::new(format!("Radius: {:.2}", self.radius)))
            .push(
                Slider::new(
                    &mut self.slider,
                    1.0..=100.0,
                    self.radius,
                    Message::RadiusChanged,
                )
                .step(0.01),
            );

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
