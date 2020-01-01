//! Display a progressbar
//!
//!
//! [`Progressbar`]: struct.Progressbar.html
use crate::{
    layout, Element, Hasher, Layout, Length, Point, Rectangle, Size, Widget,
};

use std::{hash::Hash, ops::RangeInclusive};

/// A progressbar
///
/// # Example
///
/// ```
/// # use iced_native::Progressbar;
///
/// Progressbar::new(0.0..=100.0, value);
/// ```
///
/// TODO: Get a progressbar render
/// ![Slider drawn by Coffee's renderer](https://github.com/hecrj/coffee/blob/bda9818f823dfcb8a7ad0ff4940b4d4b387b5208/images/ui/slider.png?raw=true)
#[allow(missing_debug_implementations)]
pub struct Progressbar {
    range: RangeInclusive<f32>,
    value: f32,
    width: Length,
}

impl Progressbar {
    /// Creates a new [`Progressbar`].
    ///
    /// It expects:
    ///   * an inclusive range of possible values
    ///   * the current value of the [`Progressbar`]
    ///
    /// [`Progressbar`]: struct.Progressbar.html
    pub fn new(range: RangeInclusive<f32>, value: f32) -> Self {
        Progressbar {
            value: value.max(*range.start()).min(*range.end()),
            range,
            width: Length::Fill,
        }
    }

    /// Sets the width of the [`Progressbar`].
    ///
    /// [`Progressbar`]: struct.Progressbar.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for Progressbar
where
    Renderer: self::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits
            .width(self.width)
            .height(Length::Units(renderer.height() as u16));

        let size = limits.resolve(Size::ZERO);

        layout::Node::new(size)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        _cursor_position: Point,
    ) -> Renderer::Output {
        renderer.draw(layout.bounds(), self.range.clone(), self.value)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.width.hash(state);
    }
}

/// The renderer of a [`Progressbar`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`Progressbar`] in your user interface.
///
/// [`Progressbar`]: struct.Progressbar.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: crate::Renderer {
    /// Returns the height of the [`Progressbar`].
    ///
    /// [`Progressbar`]: struct.Progressbar.html
    fn height(&self) -> u32;

    /// Draws a [`Progressbar`].
    ///
    /// It receives:
    ///   * the local state of the [`Progressbar`]
    ///   * the bounds of the [`Progressbar`]
    ///   * the range of values of the [`Progressbar`]
    ///   * the current value of the [`Progressbar`]
    ///
    /// [`Progressbar`]: struct.Progressbar.html
    /// [`State`]: struct.State.html
    /// [`Class`]: enum.Class.html
    fn draw(
        &self,
        bounds: Rectangle,
        range: RangeInclusive<f32>,
        value: f32,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<Progressbar> for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer,
    Message: 'static,
{
    fn from(progressbar: Progressbar) -> Element<'a, Message, Renderer> {
        Element::new(progressbar)
    }
}
