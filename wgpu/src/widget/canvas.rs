//! Draw freely in 2D.
use crate::{Defaults, Primitive, Renderer};

use iced_native::{
    layout, Element, Hasher, Layout, Length, MouseCursor, Point, Size, Widget,
};
use std::hash::Hash;

pub mod layer;
pub mod path;

mod fill;
mod frame;
mod stroke;

pub use fill::Fill;
pub use frame::Frame;
pub use layer::Layer;
pub use path::Path;
pub use stroke::{LineCap, LineJoin, Stroke};

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
        layout: Layout<'_>,
        _cursor_position: Point,
    ) -> (Primitive, MouseCursor) {
        let bounds = layout.bounds();
        let origin = Point::new(bounds.x, bounds.y);
        let size = Size::new(bounds.width, bounds.height);

        (
            Primitive::Group {
                primitives: self
                    .layers
                    .iter()
                    .map(|layer| Primitive::Mesh2D {
                        origin,
                        buffers: layer.draw(size),
                    })
                    .collect(),
            },
            MouseCursor::Idle,
        )
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
