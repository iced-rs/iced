//! Provide progress feedback to your users.
use crate::layout;
use crate::renderer;
use crate::{Color, Element, Layout, Length, Point, Rectangle, Size, Widget};

use std::ops::RangeInclusive;

pub use iced_style::progress_bar::{Style, StyleSheet};

/// A bar that displays progress.
///
/// # Example
/// ```
/// # use iced_native::widget::ProgressBar;
/// let value = 50.0;
///
/// ProgressBar::new(0.0..=100.0, value);
/// ```
///
/// ![Progress bar drawn with `iced_wgpu`](https://user-images.githubusercontent.com/18618951/71662391-a316c200-2d51-11ea-9cef-52758cab85e3.png)
#[allow(missing_debug_implementations)]
pub struct ProgressBar<'a> {
    range: RangeInclusive<f32>,
    value: f32,
    width: Length,
    height: Option<Length>,
    custom_style_sheet: Option<Box<dyn StyleSheet + 'a>>,
}

impl<'a> ProgressBar<'a> {
    /// The default height of a [`ProgressBar`].
    pub const DEFAULT_HEIGHT: u16 = 30;

    /// Creates a new [`ProgressBar`].
    ///
    /// It expects:
    ///   * an inclusive range of possible values
    ///   * the current value of the [`ProgressBar`]
    pub fn new(range: RangeInclusive<f32>, value: f32) -> Self {
        ProgressBar {
            value: value.max(*range.start()).min(*range.end()),
            range,
            width: Length::Fill,
            height: None,
            custom_style_sheet: None,
        }
    }

    /// Sets the width of the [`ProgressBar`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`ProgressBar`].
    pub fn height(mut self, height: Length) -> Self {
        self.height = Some(height);
        self
    }

    /// Sets the style of the [`ProgressBar`].
    pub fn style(
        mut self,
        style_sheet: impl Into<Box<dyn StyleSheet + 'a>>,
    ) -> Self {
        self.custom_style_sheet = Some(style_sheet.into());
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for ProgressBar<'a>
where
    Renderer: crate::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height.unwrap_or(Length::Units(Self::DEFAULT_HEIGHT))
    }

    fn layout(
        &self,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits
            .width(self.width)
            .height(self.height.unwrap_or(Length::Units(Self::DEFAULT_HEIGHT)));

        let size = limits.resolve(Size::ZERO);

        layout::Node::new(size)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let (range_start, range_end) = self.range.clone().into_inner();

        let active_progress_width = if range_start >= range_end {
            0.0
        } else {
            bounds.width * (self.value - range_start)
                / (range_end - range_start)
        };

        let style_sheet = match &self.custom_style_sheet {
            Some(style_sheet) => style_sheet,
            None => &renderer_style.progress_bar_style_sheet,
        };
        let style = style_sheet.style();

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle { ..bounds },
                border_radius: style.border_radius,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            style.background,
        );

        if active_progress_width > 0.0 {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        width: active_progress_width,
                        ..bounds
                    },
                    border_radius: style.border_radius,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
                style.bar,
            );
        }
    }
}

impl<'a, Message, Renderer> From<ProgressBar<'a>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + crate::Renderer,
    Message: 'a,
{
    fn from(progress_bar: ProgressBar<'a>) -> Element<'a, Message, Renderer> {
        Element::new(progress_bar)
    }
}
