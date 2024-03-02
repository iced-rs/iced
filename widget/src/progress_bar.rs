//! Provide progress feedback to your users.
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::widget::Tree;
use crate::core::{Border, Element, Layout, Length, Rectangle, Size, Widget};

use std::ops::RangeInclusive;

use iced_style::core::renderer::Quad;
use iced_style::core::Background;
use iced_style::core::Color;
pub use iced_style::progress_bar::{Appearance, StyleSheet};

use self::progress::get_progress_rect;

mod progress;

/// A bar that displays progress.
///
/// # Example
/// ```no_run
/// # type ProgressBar = iced_widget::ProgressBar<iced_widget::style::Theme>;
/// #
/// let value = 50.0;
///
/// ProgressBar::new(0.0..=100.0, value);
/// ```
///
/// ![Progress bar drawn with `iced_wgpu`](https://user-images.githubusercontent.com/18618951/71662391-a316c200-2d51-11ea-9cef-52758cab85e3.png)
#[allow(missing_debug_implementations)]
pub struct ProgressBar<Theme = crate::Theme>
where
    Theme: StyleSheet,
{
    range: RangeInclusive<f32>,
    value: f32,
    buffer: f32,
    vertical: bool,
    reverse: bool,
    length: Length,
    size: Option<Length>,
    style: Theme::Style,
}

impl<Theme> ProgressBar<Theme>
where
    Theme: StyleSheet,
{
    /// The default height of a [`ProgressBar`].
    pub const DEFAULT_HEIGHT: f32 = 30.0;

    /// Creates a new [`ProgressBar`].
    ///
    /// It expects:
    ///   * an inclusive range of possible values
    ///   * the current value of the [`ProgressBar`]
    pub fn new(range: RangeInclusive<f32>, value: f32) -> Self {
        ProgressBar {
            value: value.clamp(*range.start(), *range.end()),
            buffer: *range.start(),
            vertical: false,
            reverse: false,
            range,
            length: Length::Fill,
            size: None,
            style: Default::default(),
        }
    }

    /// Sets the buffer of the [`ProgressBar`].
    pub fn buffer(mut self, buffer: f32) -> Self {
        self.buffer = buffer;
        self
    }

    /// Sets the vertical orientation of the [`ProgressBar`]. With [`bool`] value
    pub fn vertical_f(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }

    /// Sets the vertical orientation of the [`ProgressBar`].
    pub fn vertical(self) -> Self {
        self.vertical_f(true)
    }

    /// Sets the reverse value rendering of the [`ProgressBar`]. With [`bool`] value
    pub fn reverse_f(mut self, reverse: bool) -> Self {
        self.reverse = reverse;
        self
    }

    /// Sets the reverse value rendering of the [`ProgressBar`].
    pub fn reverse(self) -> Self {
        self.reverse_f(true)
    }

    /// Sets the width of the [`ProgressBar`].
    pub fn length(mut self, width: impl Into<Length>) -> Self {
        self.length = width.into();
        self
    }

    /// Sets the height of the [`ProgressBar`].
    pub fn size(mut self, height: impl Into<Length>) -> Self {
        self.size = Some(height.into());
        self
    }

    /// Sets the style of the [`ProgressBar`].
    pub fn style(mut self, style: impl Into<Theme::Style>) -> Self {
        self.style = style.into();
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for ProgressBar<Theme>
where
    Renderer: crate::core::Renderer,
    Theme: StyleSheet,
{
    fn size(&self) -> Size<Length> {
        let ln = self.length;
        let sz = self.size.unwrap_or(Length::Fixed(Self::DEFAULT_HEIGHT));

        if self.vertical {
            Size {
                width: sz,
                height: ln,
            }
        } else {
            Size {
                width: ln,
                height: sz,
            }
        }
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = self.size.unwrap_or(Length::Fixed(Self::DEFAULT_HEIGHT));

        if self.vertical {
            layout::atomic(limits, size, self.length)
        } else {
            layout::atomic(limits, self.length, size)
        }
    }

    fn draw(
        &self,
        _state: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let style = theme.appearance(&self.style);

        let mut render_bar = |bounds, bkg: Background| {
            renderer.fill_quad(
                Quad {
                    bounds,
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.,
                        radius: style.border_radius,
                    },
                    ..Default::default()
                },
                bkg,
            )
        };

        //фон
        render_bar(bounds, style.background);

        //буфер
        if self.buffer > *self.range.start() {
            render_bar(
                get_progress_rect(
                    bounds,
                    self.buffer,
                    self.range.clone(),
                    self.vertical,
                    self.reverse,
                ),
                style.buffer,
            );
        }

        //значение
        if self.value > *self.range.start() {
            render_bar(
                get_progress_rect(
                    bounds,
                    self.value,
                    self.range.clone(),
                    self.vertical,
                    self.reverse,
                ),
                style.bar,
            );
        }
    }
}

impl<'a, Message, Theme, Renderer> From<ProgressBar<Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: StyleSheet + 'a,
    Renderer: 'a + crate::core::Renderer,
{
    fn from(
        progress_bar: ProgressBar<Theme>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(progress_bar)
    }
}
