mod combo_box {
    use iced_native::{
        layout, mouse, Background, Color, Element, Hasher, Layer, Layout,
        Length, Overlay, Point, Size, Vector, Widget,
    };
    use iced_wgpu::{Defaults, Primitive, Renderer};

    pub struct ComboBox;

    impl ComboBox {
        pub fn new() -> Self {
            Self
        }
    }

    impl<'a, Message> Widget<'a, Message, Renderer> for ComboBox {
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
            layout::Node::new(Size::new(50.0, 50.0))
        }

        fn hash_layout(&self, _state: &mut Hasher) {}

        fn draw(
            &self,
            _renderer: &mut Renderer,
            _defaults: &Defaults,
            layout: Layout<'_>,
            _cursor_position: Point,
        ) -> (Primitive, mouse::Interaction) {
            let primitive = Primitive::Quad {
                bounds: layout.bounds(),
                background: Background::Color(Color::BLACK),
                border_width: 0,
                border_radius: 0,
                border_color: Color::TRANSPARENT,
            };

            (primitive, mouse::Interaction::default())
        }

        fn overlay(
            &mut self,
            layout: Layout<'_>,
        ) -> Option<Overlay<'a, Message, Renderer>> {
            Some(Overlay::new(layout.position(), Box::new(Menu)))
        }
    }

    impl<'a, Message> Into<Element<'a, Message, Renderer>> for ComboBox {
        fn into(self) -> Element<'a, Message, Renderer> {
            Element::new(self)
        }
    }

    pub struct Menu;

    impl<Message> Layer<Message, Renderer> for Menu {
        fn layout(
            &self,
            _renderer: &Renderer,
            _bounds: Size,
            position: Point,
        ) -> layout::Node {
            let mut node = layout::Node::new(Size::new(100.0, 100.0));

            node.move_to(position + Vector::new(25.0, 25.0));

            node
        }

        fn hash_layout(&self, state: &mut Hasher, position: Point) {
            use std::hash::Hash;

            (position.x as u32).hash(state);
            (position.y as u32).hash(state);
        }

        fn draw(
            &self,
            _renderer: &mut Renderer,
            _defaults: &Defaults,
            layout: Layout<'_>,
            _cursor_position: Point,
        ) -> (Primitive, mouse::Interaction) {
            let primitive = Primitive::Quad {
                bounds: layout.bounds(),
                background: Background::Color(Color {
                    r: 0.0,
                    g: 0.0,
                    b: 1.0,
                    a: 0.5,
                }),
                border_width: 0,
                border_radius: 0,
                border_color: Color::TRANSPARENT,
            };

            (primitive, mouse::Interaction::default())
        }
    }
}

pub use combo_box::ComboBox;

use iced::{
    button, Button, Column, Container, Element, Length, Sandbox, Settings, Text,
};

pub fn main() {
    Example::run(Settings::default())
}

#[derive(Default)]
struct Example {
    button: button::State,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    ButtonPressed,
}

impl Sandbox for Example {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("Combo box - Iced")
    }

    fn update(&mut self, _message: Message) {}

    fn view(&mut self) -> Element<Message> {
        let combo_box = ComboBox::new();

        let button = Button::new(&mut self.button, Text::new("Press me!"))
            .on_press(Message::ButtonPressed);

        let content = Column::new().spacing(10).push(combo_box).push(button);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
