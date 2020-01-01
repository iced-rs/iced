use crate::{
    button, checkbox, column, radio, row, scrollable, text, text_input, Color,
    Element, Font, HorizontalAlignment, Layout, Point, Rectangle, Renderer,
    Size, VerticalAlignment,
};

/// A renderer that does nothing.
///
/// It can be useful if you are writing tests!
#[derive(Debug, Clone, Copy)]
pub struct Null;

impl Null {
    pub fn new() -> Null {
        Null
    }
}

impl Renderer for Null {
    type Output = ();
    type Defaults = ();
}

impl column::Renderer for Null {
    fn draw<Message>(
        &mut self,
        _defaults: &Self::Defaults,
        _content: &[Element<'_, Message, Self>],
        _layout: Layout<'_>,
        _cursor_position: Point,
    ) {
    }
}

impl row::Renderer for Null {
    fn draw<Message>(
        &mut self,
        _defaults: &Self::Defaults,
        _content: &[Element<'_, Message, Self>],
        _layout: Layout<'_>,
        _cursor_position: Point,
    ) {
    }
}

impl text::Renderer for Null {
    fn default_size(&self) -> u16 {
        20
    }

    fn measure(
        &self,
        _content: &str,
        _size: u16,
        _font: Font,
        _bounds: Size,
    ) -> (f32, f32) {
        (0.0, 20.0)
    }

    fn draw(
        &mut self,
        _defaults: &Self::Defaults,
        _bounds: Rectangle,
        _content: &str,
        _size: u16,
        _font: Font,
        _color: Option<Color>,
        _horizontal_alignment: HorizontalAlignment,
        _vertical_alignment: VerticalAlignment,
    ) {
    }
}

impl scrollable::Renderer for Null {
    fn scrollbar(
        &self,
        _bounds: Rectangle,
        _content_bounds: Rectangle,
        _offset: u32,
    ) -> Option<scrollable::Scrollbar> {
        None
    }

    fn draw(
        &mut self,
        _scrollable: &scrollable::State,
        _bounds: Rectangle,
        _content_bounds: Rectangle,
        _is_mouse_over: bool,
        _is_mouse_over_scrollbar: bool,
        _scrollbar: Option<scrollable::Scrollbar>,
        _offset: u32,
        _content: Self::Output,
    ) {
    }
}

impl text_input::Renderer for Null {
    type Style = ();

    fn default_size(&self) -> u16 {
        20
    }

    fn measure_value(&self, _value: &str, _size: u16) -> f32 {
        0.0
    }

    fn offset(
        &self,
        _text_bounds: Rectangle,
        _size: u16,
        _value: &text_input::Value,
        _state: &text_input::State,
    ) -> f32 {
        0.0
    }

    fn draw(
        &mut self,
        _bounds: Rectangle,
        _text_bounds: Rectangle,
        _cursor_position: Point,
        _size: u16,
        _placeholder: &str,
        _value: &text_input::Value,
        _state: &text_input::State,
        _style: &Self::Style,
    ) -> Self::Output {
    }
}

impl button::Renderer for Null {
    type Style = ();

    fn draw<Message>(
        &mut self,
        _defaults: &Self::Defaults,
        _bounds: Rectangle,
        _cursor_position: Point,
        _is_disabled: bool,
        _is_pressed: bool,
        _style: &Self::Style,
        _content: &Element<'_, Message, Self>,
        _content_layout: Layout<'_>,
    ) -> Self::Output {
    }
}

impl radio::Renderer for Null {
    fn default_size(&self) -> u32 {
        20
    }

    fn draw(
        &mut self,
        _bounds: Rectangle,
        _is_selected: bool,
        _is_mouse_over: bool,
        _label: Self::Output,
    ) {
    }
}

impl checkbox::Renderer for Null {
    fn default_size(&self) -> u32 {
        20
    }

    fn draw(
        &mut self,
        _bounds: Rectangle,
        _is_checked: bool,
        _is_mouse_over: bool,
        _label: Self::Output,
    ) {
    }
}
