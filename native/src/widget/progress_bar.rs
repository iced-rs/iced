//! Provide progress feedback to your users.
use crate::{
    layout, Element, Hasher, Layout, Length, Point, Rectangle, Size, Widget,
};

use std::{hash::Hash, ops::RangeInclusive};

/// A bar that displays progress.
///
/// # Example
/// ```
/// # use iced_native::renderer::Null;
/// #
/// # pub type ProgressBar = iced_native::ProgressBar<Null>;
/// let value = 50.0;
///
/// ProgressBar::new(0.0..=100.0, value);
/// ```
///
/// ![Progress bar drawn with `iced_wgpu`](https://user-images.githubusercontent.com/18618951/71662391-a316c200-2d51-11ea-9cef-52758cab85e3.png)
#[allow(missing_debug_implementations)]
pub struct ProgressBar<Renderer: self::Renderer> {
    range: RangeInclusive<f32>,
    value: f32,
    width: Length,
    height: Option<Length>,
    style: Renderer::Style,
}

impl<Renderer: self::Renderer> ProgressBar<Renderer> {
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
            height: None,
            style: Renderer::Style::default(),
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
        self.height = Some(height);
        self
    }

    /// Sets the style of the [`ProgressBar`].
    ///
    /// [`ProgressBar`]: struct.ProgressBar.html
    pub fn style(mut self, style: impl Into<Renderer::Style>) -> Self {
        self.style = style.into();
        self
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for ProgressBar<Renderer>
where
    Renderer: self::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
            .unwrap_or(Length::Units(Renderer::DEFAULT_HEIGHT))
    }

    fn layout(
        &self,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(
            self.height
                .unwrap_or(Length::Units(Renderer::DEFAULT_HEIGHT)),
        );

        let size = limits.resolve(Size::ZERO);

        layout::Node::new(size)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) -> Renderer::Output {
        renderer.draw(
            layout.bounds(),
            self.range.clone(),
            self.value,
            &self.style,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
        self.height.hash(state);
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
    /// The style supported by this renderer.
    type Style: Default;

    /// The default height of a [`ProgressBar`].
    ///
    /// [`ProgressBar`]: struct.ProgressBar.html
    const DEFAULT_HEIGHT: u16;

    /// Draws a [`ProgressBar`].
    ///
    /// It receives:
    ///   * the bounds of the [`ProgressBar`]
    ///   * the range of values of the [`ProgressBar`]
    ///   * the current value of the [`ProgressBar`]
    ///   * maybe a specific background of the [`ProgressBar`]
    ///   * maybe a specific active color of the [`ProgressBar`]
    ///
    /// [`ProgressBar`]: struct.ProgressBar.html
    fn draw(
        &self,
        bounds: Rectangle,
        range: RangeInclusive<f32>,
        value: f32,
        style: &Self::Style,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<ProgressBar<Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer,
    Message: 'a,
{
    fn from(
        progress_bar: ProgressBar<Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(progress_bar)
    }
}
