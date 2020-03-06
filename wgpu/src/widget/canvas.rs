//! Draw 2D graphics for your users.
//!
//! A [`Canvas`] widget can be used to draw different kinds of 2D shapes in a
//! [`Frame`]. It can be used for animation, data visualization, game graphics,
//! and more!
//!
//! [`Canvas`]: struct.Canvas.html
//! [`Frame`]: struct.Frame.html
use crate::{Defaults, Primitive, Renderer};

use iced_native::{
    layout, Element, Hasher, Layout, Length, MouseCursor, Point, Size, Widget,
};
use std::hash::Hash;

pub mod layer;
pub mod path;

mod drawable;
mod fill;
mod frame;
mod stroke;
mod text;

pub use drawable::Drawable;
pub use fill::Fill;
pub use frame::Frame;
pub use layer::Layer;
pub use path::Path;
pub use stroke::{LineCap, LineJoin, Stroke};
pub use text::TextNode;

/// A widget capable of drawing 2D graphics.
///
/// A [`Canvas`] may contain multiple layers. A [`Layer`] is drawn using the
/// painter's algorithm. In other words, layers will be drawn on top of each in
/// the same order they are pushed into the [`Canvas`].
///
/// [`Canvas`]: struct.Canvas.html
/// [`Layer`]: layer/trait.Layer.html
#[derive(Debug)]
pub struct Canvas<'a> {
    width: Length,
    height: Length,
    layers: Vec<Box<dyn Layer + 'a>>,
}

impl<'a> Canvas<'a> {
    const DEFAULT_SIZE: u16 = 100;

    /// Creates a new [`Canvas`] with no layers.
    ///
    /// [`Canvas`]: struct.Canvas.html
    pub fn new() -> Self {
        Canvas {
            width: Length::Units(Self::DEFAULT_SIZE),
            height: Length::Units(Self::DEFAULT_SIZE),
            layers: Vec::new(),
        }
    }

    /// Sets the width of the [`Canvas`].
    ///
    /// [`Canvas`]: struct.Canvas.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Canvas`].
    ///
    /// [`Canvas`]: struct.Canvas.html
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Adds a [`Layer`] to the [`Canvas`].
    ///
    /// It will be drawn on top of previous layers.
    ///
    /// [`Layer`]: layer/trait.Layer.html
    /// [`Canvas`]: struct.Canvas.html
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
                    .map(|layer| layer.draw(origin, size))
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
