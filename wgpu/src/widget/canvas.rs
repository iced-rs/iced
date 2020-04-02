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
pub use text::Text;

/// A widget capable of drawing 2D graphics.
///
/// A [`Canvas`] may contain multiple layers. A [`Layer`] is drawn using the
/// painter's algorithm. In other words, layers will be drawn on top of each
/// other in the same order they are pushed into the [`Canvas`].
///
/// [`Canvas`]: struct.Canvas.html
/// [`Layer`]: layer/trait.Layer.html
///
/// # Examples
/// The repository has a couple of [examples] showcasing how to use a
/// [`Canvas`]:
///
/// - [`clock`], an application that uses the [`Canvas`] widget to draw a clock
/// and its hands to display the current time.
/// - [`solar_system`], an animated solar system drawn using the [`Canvas`] widget
/// and showcasing how to compose different transforms.
///
/// [examples]: https://github.com/hecrj/iced/tree/0.1/examples
/// [`clock`]: https://github.com/hecrj/iced/tree/0.1/examples/clock
/// [`solar_system`]: https://github.com/hecrj/iced/tree/0.1/examples/solar_system
///
/// ## Drawing a simple circle
/// If you want to get a quick overview, here's how we can draw a simple circle:
///
/// ```no_run
/// # mod iced {
/// #     pub use iced_wgpu::canvas;
/// #     pub use iced_native::Color;
/// # }
/// use iced::canvas::{self, layer, Canvas, Drawable, Fill, Frame, Path};
/// use iced::Color;
///
/// // First, we define the data we need for drawing
/// #[derive(Debug)]
/// struct Circle {
///     radius: f32,
/// }
///
/// // Then, we implement the `Drawable` trait
/// impl Drawable for Circle {
///     fn draw(&self, frame: &mut Frame) {
///         // We create a `Path` representing a simple circle
///         let circle = Path::new(|p| p.circle(frame.center(), self.radius));
///
///         // And fill it with some color
///         frame.fill(&circle, Fill::Color(Color::BLACK));
///     }
/// }
///
/// // We can use a `Cache` to avoid unnecessary re-tessellation
/// let cache: layer::Cache<Circle> = layer::Cache::new();
///
/// // Finally, we simply provide the data to our `Cache` and push the resulting
/// // layer into a `Canvas`
/// let canvas = Canvas::new()
///     .push(cache.with(&Circle { radius: 50.0 }));
/// ```
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
                    .map(|layer| Primitive::Cached {
                        origin,
                        cache: layer.draw(size),
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
