//! Provide progress feedback to your users.
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::widget::Tree;
use crate::core::{
    Background, Border, Element, Layout, Length, Rectangle, Size, Widget,
};
use crate::style::Theme;

use std::ops::RangeInclusive;

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
pub struct ProgressBar<Theme = crate::Theme> {
    range: RangeInclusive<f32>,
    value: f32,
    width: Length,
    height: Option<Length>,
    style: Style<Theme>,
}

impl<Theme> ProgressBar<Theme> {
    /// The default height of a [`ProgressBar`].
    pub const DEFAULT_HEIGHT: f32 = 30.0;

    /// Creates a new [`ProgressBar`].
    ///
    /// It expects:
    ///   * an inclusive range of possible values
    ///   * the current value of the [`ProgressBar`]
    pub fn new(range: RangeInclusive<f32>, value: f32) -> Self
    where
        Style<Theme>: Default,
    {
        ProgressBar {
            value: value.clamp(*range.start(), *range.end()),
            range,
            width: Length::Fill,
            height: None,
            style: Style::default(),
        }
    }

    /// Sets the width of the [`ProgressBar`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`ProgressBar`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = Some(height.into());
        self
    }

    /// Sets the style of the [`ProgressBar`].
    pub fn style(mut self, style: fn(&Theme) -> Appearance) -> Self {
        self.style = style.into();
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for ProgressBar<Theme>
where
    Renderer: crate::core::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height.unwrap_or(Length::Fixed(Self::DEFAULT_HEIGHT)),
        }
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(
            limits,
            self.width,
            self.height.unwrap_or(Length::Fixed(Self::DEFAULT_HEIGHT)),
        )
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
        let (range_start, range_end) = self.range.clone().into_inner();

        let active_progress_width = if range_start >= range_end {
            0.0
        } else {
            bounds.width * (self.value - range_start)
                / (range_end - range_start)
        };

        let appearance = (self.style.0)(theme);

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle { ..bounds },
                border: appearance.border,
                ..renderer::Quad::default()
            },
            appearance.background,
        );

        if active_progress_width > 0.0 {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        width: active_progress_width,
                        ..bounds
                    },
                    border: Border::with_radius(appearance.border.radius),
                    ..renderer::Quad::default()
                },
                appearance.bar,
            );
        }
    }
}

impl<'a, Message, Theme, Renderer> From<ProgressBar<Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: 'a + crate::core::Renderer,
{
    fn from(
        progress_bar: ProgressBar<Theme>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(progress_bar)
    }
}

/// The appearance of a progress bar.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The [`Background`] of the progress bar.
    pub background: Background,
    /// The [`Background`] of the bar of the progress bar.
    pub bar: Background,
    /// The [`Border`] of the progress bar.
    pub border: Border,
}

/// The style of a [`ProgressBar`].
#[derive(Debug, PartialEq, Eq)]
pub struct Style<Theme>(fn(&Theme) -> Appearance);

impl<Theme> Clone for Style<Theme> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Theme> Copy for Style<Theme> {}

impl Default for Style<Theme> {
    fn default() -> Self {
        Style(primary)
    }
}

impl<Theme> From<fn(&Theme) -> Appearance> for Style<Theme> {
    fn from(f: fn(&Theme) -> Appearance) -> Self {
        Style(f)
    }
}

/// The primary style of a [`ProgressBar`].
pub fn primary(theme: &Theme) -> Appearance {
    let palette = theme.extended_palette();

    styled(
        palette.background.strong.color,
        palette.primary.strong.color,
    )
}

/// The secondary style of a [`ProgressBar`].
pub fn secondary(theme: &Theme) -> Appearance {
    let palette = theme.extended_palette();

    styled(
        palette.background.strong.color,
        palette.secondary.base.color,
    )
}

/// The success style of a [`ProgressBar`].
pub fn success(theme: &Theme) -> Appearance {
    let palette = theme.extended_palette();

    styled(palette.background.strong.color, palette.success.base.color)
}

/// The danger style of a [`ProgressBar`].
pub fn danger(theme: &Theme) -> Appearance {
    let palette = theme.extended_palette();

    styled(palette.background.strong.color, palette.danger.base.color)
}

fn styled(
    background: impl Into<Background>,
    bar: impl Into<Background>,
) -> Appearance {
    Appearance {
        background: background.into(),
        bar: bar.into(),
        border: Border::with_radius(2),
    }
}
