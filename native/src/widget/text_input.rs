use crate::{
    Element, Event, Hasher, Layout, Length, Node, Point, Rectangle, Style,
    Widget,
};

pub use iced_core::{text_input::State, TextInput};

impl<'a, Message, Renderer> Widget<Message, Renderer> for TextInput<'a, Message>
where
    Renderer: self::Renderer,
    Message: Clone + std::fmt::Debug,
{
    fn node(&self, renderer: &Renderer) -> Node {
        let text_bounds =
            Node::new(Style::default().width(Length::Fill).height(
                Length::Units(self.size.unwrap_or(renderer.default_size())),
            ));

        Node::with_children(
            Style::default()
                .width(self.width)
                .max_width(self.width)
                .padding(self.padding),
            vec![text_bounds],
        )
    }

    fn on_event(
        &mut self,
        _event: Event,
        _layout: Layout<'_>,
        _cursor_position: Point,
        _messages: &mut Vec<Message>,
        _renderer: &Renderer,
    ) {
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        let bounds = layout.bounds();
        let text_bounds = layout.children().next().unwrap().bounds();

        renderer.draw(&self, bounds, text_bounds, cursor_position)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::any::TypeId;
        use std::hash::Hash;

        TypeId::of::<TextInput<'static, ()>>().hash(state);

        self.width.hash(state);
        self.max_width.hash(state);
        self.padding.hash(state);
        self.size.hash(state);
    }
}

pub trait Renderer: crate::Renderer + Sized {
    fn default_size(&self) -> u16;

    fn draw<Message>(
        &mut self,
        text_input: &TextInput<'_, Message>,
        bounds: Rectangle,
        text_bounds: Rectangle,
        cursor_position: Point,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<TextInput<'a, Message>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'static + self::Renderer,
    Message: 'static + Clone + std::fmt::Debug,
{
    fn from(button: TextInput<'a, Message>) -> Element<'a, Message, Renderer> {
        Element::new(button)
    }
}
