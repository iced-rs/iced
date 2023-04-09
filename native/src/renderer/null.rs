use crate::{
    button, checkbox, column, container, pane_grid, progress_bar, radio, row,
    scrollable, slider, text, text_input, toggler, Color, Element, Font,
    HorizontalAlignment, Layout, Padding, Point, Rectangle, Renderer, Size,
    VerticalAlignment,
};

/// A renderer that does nothing.
///
/// It can be useful if you are writing tests!
#[derive(Debug, Clone, Copy)]
pub struct Null;

impl Null {
    /// Creates a new [`Null`] renderer.
    pub fn new() -> Null {
        Null
    }
}

impl Renderer for Null {
    type Output = ();
    type Defaults = ();

    fn overlay(&mut self, _base: (), _overlay: (), _overlay_bounds: Rectangle) {
    }
}

impl column::Renderer for Null {
    fn draw<Message>(
        &mut self,
        _defaults: &Self::Defaults,
        _content: &[Element<'_, Message, Self>],
        _layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
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
        _viewport: &Rectangle,
    ) {
    }
}

impl text::Renderer for Null {
    type Font = Font;

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
    type Style = ();

    fn scrollbar(
        &self,
        _bounds: Rectangle,
        _content_bounds: Rectangle,
        _offset: u32,
        _scrollbar_width: u16,
        _scrollbar_margin: u16,
        _scroller_width: u16,
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
        _style: &Self::Style,
        _content: Self::Output,
    ) {
    }
}

impl text_input::Renderer for Null {
    type Style = ();

    fn measure_value(&self, _value: &str, _size: u16, _font: Font) -> f32 {
        0.0
    }

    fn offset(
        &self,
        _text_bounds: Rectangle,
        _font: Font,
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
        _font: Font,
        _size: u16,
        _placeholder: &str,
        _value: &text_input::Value,
        _state: &text_input::State,
        _style: &Self::Style,
    ) -> Self::Output {
    }
}

impl button::Renderer for Null {
    const DEFAULT_PADDING: Padding = Padding::ZERO;

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
    type Style = ();

    const DEFAULT_SIZE: u16 = 20;
    const DEFAULT_SPACING: u16 = 15;

    fn draw(
        &mut self,
        _bounds: Rectangle,
        _is_selected: bool,
        _is_mouse_over: bool,
        _label: Self::Output,
        _style: &Self::Style,
    ) {
    }
}

impl checkbox::Renderer for Null {
    type Style = ();

    const DEFAULT_SIZE: u16 = 20;
    const DEFAULT_SPACING: u16 = 15;

    fn draw(
        &mut self,
        _bounds: Rectangle,
        _is_checked: bool,
        _is_mouse_over: bool,
        _label: Self::Output,
        _style: &Self::Style,
    ) {
    }
}

impl slider::Renderer for Null {
    type Style = ();

    const DEFAULT_HEIGHT: u16 = 30;

    fn draw(
        &mut self,
        _bounds: Rectangle,
        _cursor_position: Point,
        _range: std::ops::RangeInclusive<f32>,
        _value: f32,
        _is_dragging: bool,
        _style_sheet: &Self::Style,
    ) {
    }
}

impl progress_bar::Renderer for Null {
    type Style = ();

    const DEFAULT_HEIGHT: u16 = 30;

    fn draw(
        &self,
        _bounds: Rectangle,
        _range: std::ops::RangeInclusive<f32>,
        _value: f32,
        _style: &Self::Style,
    ) {
    }
}

impl container::Renderer for Null {
    type Style = ();

    fn draw<Message>(
        &mut self,
        _defaults: &Self::Defaults,
        _bounds: Rectangle,
        _cursor_position: Point,
        _viewport: &Rectangle,
        _style: &Self::Style,
        _content: &Element<'_, Message, Self>,
        _content_layout: Layout<'_>,
    ) {
    }
}

impl pane_grid::Renderer for Null {
    type Style = ();

    fn draw<Message>(
        &mut self,
        _defaults: &Self::Defaults,
        _content: &[(pane_grid::Pane, pane_grid::Content<'_, Message, Self>)],
        _dragging: Option<(pane_grid::Pane, Point)>,
        _resizing: Option<(pane_grid::Axis, Rectangle, bool)>,
        _layout: Layout<'_>,
        _style: &<Self as pane_grid::Renderer>::Style,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) {
    }

    fn draw_pane<Message>(
        &mut self,
        _defaults: &Self::Defaults,
        _bounds: Rectangle,
        _style: &<Self as container::Renderer>::Style,
        _title_bar: Option<(
            &pane_grid::TitleBar<'_, Message, Self>,
            Layout<'_>,
        )>,
        _body: (&Element<'_, Message, Self>, Layout<'_>),
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) {
    }

    fn draw_title_bar<Message>(
        &mut self,
        _defaults: &Self::Defaults,
        _bounds: Rectangle,
        _style: &<Self as container::Renderer>::Style,
        _content: (&Element<'_, Message, Self>, Layout<'_>),
        _controls: Option<(&Element<'_, Message, Self>, Layout<'_>)>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) {
    }
}

impl toggler::Renderer for Null {
    type Style = ();

    const DEFAULT_SIZE: u16 = 20;

    fn draw(
        &mut self,
        _bounds: Rectangle,
        _is_checked: bool,
        _is_mouse_over: bool,
        _label: Option<Self::Output>,
        _style: &Self::Style,
    ) {
    }
}
