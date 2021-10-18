use crate::button;
use crate::checkbox;
use crate::container;
use crate::pane_grid;
use crate::progress_bar;
use crate::radio;
use crate::renderer::{self, Renderer};
use crate::slider;
use crate::text;
use crate::text_input;
use crate::toggler;
use crate::{Font, Padding, Point, Rectangle, Size, Vector};

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
    fn with_layer(
        &mut self,
        _bounds: Rectangle,
        _offset: Vector<u32>,
        _f: impl FnOnce(&mut Self),
    ) {
    }

    fn clear(&mut self) {}

    fn fill_rectangle(&mut self, _quad: renderer::Quad) {}
}

impl renderer::Text for Null {
    type Font = Font;

    fn fill_text(&mut self, _text: renderer::text::Section<'_, Self::Font>) {}
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

    fn hit_test(
        &self,
        _contents: &str,
        _size: f32,
        _font: Self::Font,
        _bounds: Size,
        _point: Point,
        _nearest_only: bool,
    ) -> Option<text::Hit> {
        None
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
}

impl button::Renderer for Null {
    const DEFAULT_PADDING: Padding = Padding::ZERO;

    type Style = ();
}

impl radio::Renderer for Null {
    type Style = ();

    const DEFAULT_SIZE: u16 = 20;
    const DEFAULT_SPACING: u16 = 15;
}

impl checkbox::Renderer for Null {
    type Style = ();

    const DEFAULT_SIZE: u16 = 20;
    const DEFAULT_SPACING: u16 = 15;
}

impl slider::Renderer for Null {
    type Style = ();

    const DEFAULT_HEIGHT: u16 = 30;
}

impl progress_bar::Renderer for Null {
    type Style = ();

    const DEFAULT_HEIGHT: u16 = 30;
}

impl container::Renderer for Null {
    type Style = ();
}

impl pane_grid::Renderer for Null {
    type Style = ();
}

impl toggler::Renderer for Null {
    type Style = ();

    const DEFAULT_SIZE: u16 = 20;
}
