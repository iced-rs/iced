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
    use iced_graphics::{Backend, Defaults, Primitive, Renderer};
    use iced_native::{
        layout, mouse, Background, Color, Element, Hasher, Layout, Length,
        Point, Rectangle, Size, Widget,
    };

    pub struct Circle {
        radius: u16,
    }

    impl Circle {
        pub fn new(radius: u16) -> Self {
            Self { radius }
        }
    }

    impl<Message, B> Widget<Message, Renderer<B>> for Circle
    where
        B: Backend,
    {
        fn width(&self) -> Length {
            Length::Shrink
        }

        fn height(&self) -> Length {
            Length::Shrink
        }

        fn layout(
            &self,
            _renderer: &Renderer<B>,
            _limits: &layout::Limits,
        ) -> layout::Node {
            layout::Node::new(Size::new(
                f32::from(self.radius) * 2.0,
                f32::from(self.radius) * 2.0,
            ))
        }

        fn hash_layout(&self, state: &mut Hasher) {
            use std::hash::Hash;

            self.radius.hash(state);
        }

        fn draw(
            &self,
            _renderer: &mut Renderer<B>,
            _defaults: &Defaults,
            layout: Layout<'_>,
            _cursor_position: Point,
            _viewport: &Rectangle,
        ) -> (Primitive, mouse::Interaction) {
            (
                Primitive::Quad {
                    bounds: layout.bounds(),
                    background: Background::Color(Color::BLACK),
                    border_radius: self.radius,
                    border_width: 0,
                    border_color: Color::TRANSPARENT,
                },
                mouse::Interaction::default(),
            )
        }
    }

    impl<'a, Message, B> Into<Element<'a, Message, Renderer<B>>> for Circle
    where
        B: Backend,
    {
        fn into(self) -> Element<'a, Message, Renderer<B>> {
            Element::new(self)
        }
    }
}

use circle::Circle;
use iced::{
    slider, Align, Column, Container, Element, Length, Sandbox, Settings,
    Slider, Text,
};

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

struct Example {
    radius: u16,
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
            radius: 50,
            slider: slider::State::new(),
        }
    }

    fn title(&self) -> String {
        String::from("Custom widget - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::RadiusChanged(radius) => {
                self.radius = radius.round() as u16;
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let content = Column::new()
            .padding(20)
            .spacing(20)
            .max_width(500)
            .align_items(Align::Center)
            .push(Circle::new(self.radius))
            .push(Text::new(format!("Radius: {}", self.radius.to_string())))
            .push(Slider::new(
                &mut self.slider,
                1.0..=100.0,
                f32::from(self.radius),
                Message::RadiusChanged,
            ));

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
