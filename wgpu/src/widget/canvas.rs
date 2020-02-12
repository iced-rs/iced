//! Draw freely in 2D.
use crate::{Defaults, Primitive, Renderer};

use iced_native::{
    layout, Color, Element, Hasher, Layout, Length, MouseCursor, Point, Size,
    Widget,
};
use std::hash::Hash;

pub mod layer;
pub mod path;

mod data;
mod frame;

pub use data::Data;
pub use frame::Frame;
pub use layer::Layer;
pub use path::Path;

/// A 2D drawable region.
#[derive(Debug)]
pub struct Canvas<'a> {
    width: Length,
    height: Length,
    layers: Vec<Box<dyn Layer + 'a>>,
}

impl<'a> Canvas<'a> {
    const DEFAULT_SIZE: u16 = 100;

    pub fn new() -> Self {
        Canvas {
            width: Length::Units(Self::DEFAULT_SIZE),
            height: Length::Units(Self::DEFAULT_SIZE),
            layers: Vec::new(),
        }
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    pub fn push(mut self, layer: impl Layer + 'a) -> Self {
        self.layers.push(Box::new(layer));
        self
    }
}

impl<'a, Message> Widget<Message, Renderer> for Canvas<'a> {
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);

        let size = limits.resolve(Size::ZERO);

        layout::Node::new(size)
    }

    fn draw(
        &self,
        _renderer: &mut Renderer,
        _defaults: &Defaults,
        _layout: Layout<'_>,
        _cursor_position: Point,
    ) -> (Primitive, MouseCursor) {
        (Primitive::None, MouseCursor::Idle)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        std::any::TypeId::of::<Canvas<'static>>().hash(state);

        self.width.hash(state);
        self.height.hash(state);
    }
}

impl<'a, Message> From<Canvas<'a>> for Element<'a, Message, Renderer>
where
    Message: 'static,
{
    fn from(canvas: Canvas<'a>) -> Element<'a, Message, Renderer> {
        Element::new(canvas)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Stroke {
    pub color: Color,
    pub width: f32,
    pub line_cap: LineCap,
    pub line_join: LineJoin,
}

impl Default for Stroke {
    fn default() -> Stroke {
        Stroke {
            color: Color::BLACK,
            width: 1.0,
            line_cap: LineCap::default(),
            line_join: LineJoin::default(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LineCap {
    Butt,
    Square,
    Round,
}

impl Default for LineCap {
    fn default() -> LineCap {
        LineCap::Butt
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}

impl Default for LineJoin {
    fn default() -> LineJoin {
        LineJoin::Miter
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Fill {
    Color(Color),
}

impl Default for Fill {
    fn default() -> Fill {
        Fill::Color(Color::BLACK)
    }
}
