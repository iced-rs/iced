//! Write your own renderer.
//!
//! You will need to implement the `Renderer` trait first. It simply contains
//! an `Output` associated type.
//!
//! There is no common trait to draw all the widgets. Instead, every [`Widget`]
//! constrains its generic `Renderer` type as necessary.
//!
//! This approach is flexible and composable. For instance, the
//! [`Text`] widget only needs a [`text::Renderer`] while a [`Checkbox`] widget
//! needs both a [`text::Renderer`] and a [`checkbox::Renderer`], reusing logic.
//!
//! In the end, a __renderer__ satisfying all the constraints is
//! needed to build a [`UserInterface`].
//!
//! [`Widget`]: crate::Widget
//! [`UserInterface`]: crate::UserInterface
//! [`Text`]: crate::widget::Text
//! [`text::Renderer`]: crate::widget::text::Renderer
//! [`Checkbox`]: crate::widget::Checkbox
//! [`checkbox::Renderer`]: crate::widget::checkbox::Renderer
#[cfg(debug_assertions)]
mod null;
#[cfg(debug_assertions)]
pub use null::Null;

use crate::layout;
use crate::{Background, Color, Element, Rectangle, Vector};

/// A component that can take the state of a user interface and produce an
/// output for its users.
pub trait Renderer: Sized {
    /// Lays out the elements of a user interface.
    ///
    /// You should override this if you need to perform any operations before or
    /// after layouting. For instance, trimming the measurements cache.
    fn layout<'a, Message>(
        &mut self,
        element: &Element<'a, Message, Self>,
        limits: &layout::Limits,
    ) -> layout::Node {
        element.layout(self, limits)
    }

    /// Draws the primitives recorded in the given closure in a new layer.
    ///
    /// The layer will clip its contents to the provided `bounds`.
    fn with_layer(&mut self, bounds: Rectangle, f: impl FnOnce(&mut Self));

    /// Applies a `translation` to the primitives recorded in the given closure.
    fn with_translation(
        &mut self,
        translation: Vector,
        f: impl FnOnce(&mut Self),
    );

    /// Clears all of the recorded primitives in the [`Renderer`].
    fn clear(&mut self);

    /// Fills a [`Quad`] with the provided [`Background`].
    fn fill_quad(&mut self, quad: Quad, background: impl Into<Background>);
}

/// A polygon with four sides.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    /// The bounds of the [`Quad`].
    pub bounds: Rectangle,

    /// The border radius of the [`Quad`].
    pub border_radius: f32,

    /// The border width of the [`Quad`].
    pub border_width: f32,

    /// The border color of the [`Quad`].
    pub border_color: Color,
}

/// The styling attributes of a [`Renderer`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The text color
    pub text_color: Color,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            text_color: Color::BLACK,
        }
    }
}
