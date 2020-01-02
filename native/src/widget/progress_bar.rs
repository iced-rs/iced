//! Display a ProgressBar
//!
//!
//! [`ProgressBar`]: struct.ProgressBar.html
use crate::{
    layout, Background, Color, Element, Hasher, Layout, Length, Point,
    Rectangle, Size, Widget,
};

use std::{hash::Hash, ops::RangeInclusive};

const DEFAULT_HEIGHT: Length = Length::Units(30);

/// A ProgressBar
///
/// # Example
///
/// ```
/// # use iced_native::ProgressBar;
///
/// let value = 50.0;
/// ProgressBar::new(0.0..=100.0, value);
/// ```
///
/// ![Default ProgressBar](https://user-images.githubusercontent.com/18618951/71662391-a316c200-2d51-11ea-9cef-52758cab85e3.png)
#[allow(missing_debug_implementations)]
pub struct ProgressBar {
    range: RangeInclusive<f32>,
    value: f32,
    width: Length,
    height: Length,
    background: Option<Background>,
    active_color: Option<Color>,
}

impl ProgressBar {
    /// Creates a new [`ProgressBar`].
    ///
    /// It expects:
    ///   * an inclusive range of possible values
    ///   * the current value of the [`ProgressBar`]
    ///
    /// [`ProgressBar`]: struct.ProgressBar.html
    pub fn new(range: RangeInclusive<f32>, value: f32) -> Self {
        ProgressBar {
            value: value.max(*range.start()).min(*range.end()),
            range,
            width: Length::Fill,
            height: DEFAULT_HEIGHT,
            background: None,
            active_color: None,
        }
    }

    /// Sets the width of the [`ProgressBar`].
    ///
    /// [`ProgressBar`]: struct.ProgressBar.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`ProgressBar`].
    ///
    /// [`ProgressBar`]: struct.ProgressBar.html
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the background of the [`ProgressBar`].
    ///
    /// [`ProgressBar`]: struct.ProgressBar.html
    pub fn background(mut self, background: Background) -> Self {
        self.background = Some(background);
        self
    }

    /// Sets the active color of the [`ProgressBar`].
    ///
    /// [`ProgressBar`]: struct.ProgressBar.html
    pub fn active_color(mut self, active_color: Color) -> Self {
        self.active_color = Some(active_color);
        self
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for ProgressBar
where
    Renderer: self::Renderer,
{
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
        renderer: &mut Renderer,
        layout: Layout<'_>,
        _cursor_position: Point,
    ) -> Renderer::Output {
        renderer.draw(
            layout.bounds(),
            self.range.clone(),
            self.value,
            self.background,
            self.active_color,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.width.hash(state);
    }
}

/// The renderer of a [`ProgressBar`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`ProgressBar`] in your user interface.
///
/// [`ProgressBar`]: struct.ProgressBar.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: crate::Renderer {
    /// Draws a [`ProgressBar`].
    ///
    /// It receives:
    ///   * the local state of the [`ProgressBar`]
    ///   * the bounds of the [`ProgressBar`]
    ///   * the range of values of the [`ProgressBar`]
    ///   * the current value of the [`ProgressBar`]
    ///   * maybe a specific background of the [`ProgressBar`]
    ///   * maybe a specific active color of the [`ProgressBar`]
    ///
    /// [`ProgressBar`]: struct.ProgressBar.html
    /// [`State`]: struct.State.html
    /// [`Class`]: enum.Class.html
    fn draw(
        &self,
        bounds: Rectangle,
        range: RangeInclusive<f32>,
        value: f32,
        background: Option<Background>,
        active_color: Option<Color>,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<ProgressBar> for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer,
    Message: 'static,
{
    fn from(progressbar: ProgressBar) -> Element<'a, Message, Renderer> {
        Element::new(progressbar)
    }
}
