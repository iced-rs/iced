use iced_native::{
    button, checkbox, image, radio, renderer::Debugger, slider, text, Button,
    Checkbox, Color, Image, Layout, MouseCursor, Node, Point, Radio, Slider,
    Style, Text,
};

pub struct Renderer;

impl text::Renderer for Renderer {
    fn node(&self, _text: &Text) -> Node {
        Node::new(Style::default())
    }

    fn draw(&mut self, _text: &Text, _layout: Layout<'_>) {}
}

impl checkbox::Renderer for Renderer {
    fn node<Message>(&mut self, _checkbox: &Checkbox<Message>) -> Node {
        Node::new(Style::default())
    }

    fn draw<Message>(
        &mut self,
        _checkbox: &Checkbox<Message>,
        _layout: Layout<'_>,
        _cursor_position: Point,
    ) -> MouseCursor {
        MouseCursor::OutOfBounds
    }
}

impl radio::Renderer for Renderer {
    fn node<Message>(&mut self, _checkbox: &Radio<Message>) -> Node {
        Node::new(Style::default())
    }

    fn draw<Message>(
        &mut self,
        _radio: &Radio<Message>,
        _layout: Layout<'_>,
        _cursor_position: Point,
    ) -> MouseCursor {
        MouseCursor::OutOfBounds
    }
}

impl slider::Renderer for Renderer {
    fn node<Message>(&self, _slider: &Slider<Message>) -> Node {
        Node::new(Style::default())
    }

    fn draw<Message>(
        &mut self,
        _slider: &Slider<Message>,
        _layout: Layout<'_>,
        _cursor_position: Point,
    ) -> MouseCursor {
        MouseCursor::OutOfBounds
    }
}

impl image::Renderer<&str> for Renderer {
    fn node(&mut self, _image: &Image<&str>) -> Node {
        Node::new(Style::default())
    }

    fn draw(&mut self, _checkbox: &Image<&str>, _layout: Layout<'_>) {}
}

impl button::Renderer for Renderer {
    fn node<Message>(&self, _button: &Button<Message>) -> Node {
        Node::new(Style::default())
    }

    fn draw<Message>(
        &mut self,
        _button: &Button<Message>,
        _layout: Layout<'_>,
        _cursor_position: Point,
    ) -> MouseCursor {
        MouseCursor::OutOfBounds
    }
}

impl Debugger for Renderer {
    fn explain(&mut self, _layout: &Layout<'_>, _color: Color) {}
}
