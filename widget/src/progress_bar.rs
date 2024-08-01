//! Provide progress feedback to your users.
use crate::core::border::{self, Border};
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::widget::Tree;
use crate::core::{
    self, Background, Element, Layout, Length, Rectangle, Size, Theme, Widget,
};

use std::ops::RangeInclusive;

/// A bar that displays progress.
///
/// # Example
/// ```no_run
/// # type ProgressBar<'a> = iced_widget::ProgressBar<'a>;
/// #
/// let value = 50.0;
///
/// ProgressBar::new(0.0..=100.0, value);
/// ```
///
/// ![Progress bar drawn with `iced_wgpu`](https://user-images.githubusercontent.com/18618951/71662391-a316c200-2d51-11ea-9cef-52758cab85e3.png)
#[allow(missing_debug_implementations)]
pub struct ProgressBar<'a, Theme = crate::Theme>
where
    Theme: Catalog,
{
    range: RangeInclusive<f32>,
    value: f32,
    width: Length,
    height: Option<Length>,
    class: Theme::Class<'a>,
}

impl<'a, Theme> ProgressBar<'a, Theme>
where
    Theme: Catalog,
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
            range,
            width: Length::Fill,
            height: None,
            class: Theme::default(),
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
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`ProgressBar`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for ProgressBar<'a, Theme>
where
    Theme: Catalog,
    Renderer: core::Renderer,
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

        let style = theme.style(&self.class);

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle { ..bounds },
                border: style.border,
                ..renderer::Quad::default()
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
                    border: border::rounded(style.border.radius),
                    ..renderer::Quad::default()
                },
                style.bar,
            );
        }
    }
}

impl<'a, Message, Theme, Renderer> From<ProgressBar<'a, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a + Catalog,
    Renderer: 'a + core::Renderer,
{
    fn from(
        progress_bar: ProgressBar<'a, Theme>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(progress_bar)
    }
}

/// The appearance of a progress bar.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// The [`Background`] of the progress bar.
    pub background: Background,
    /// The [`Background`] of the bar of the progress bar.
    pub bar: Background,
    /// The [`Border`] of the progress bar.
    pub border: Border,
}

/// The theme catalog of a [`ProgressBar`].
pub trait Catalog: Sized {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// A styling function for a [`ProgressBar`].
///
/// This is just a boxed closure: `Fn(&Theme, Status) -> Style`.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(primary)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

/// The primary style of a [`ProgressBar`].
pub fn primary(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    styled(
        palette.background.strong.color,
        palette.primary.strong.color,
    )
}

/// The secondary style of a [`ProgressBar`].
pub fn secondary(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    styled(
        palette.background.strong.color,
        palette.secondary.base.color,
    )
}

/// The success style of a [`ProgressBar`].
pub fn success(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    styled(palette.background.strong.color, palette.success.base.color)
}

/// The danger style of a [`ProgressBar`].
pub fn danger(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    styled(palette.background.strong.color, palette.danger.base.color)
}

fn styled(
    background: impl Into<Background>,
    bar: impl Into<Background>,
) -> Style {
    Style {
        background: background.into(),
        bar: bar.into(),
        border: border::rounded(2),
    }
}
