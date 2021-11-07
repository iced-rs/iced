//! Provide progress feedback to your users.
use crate::{bumpalo, css, Bus, Css, Element, Length, Widget};

pub use iced_style::progress_bar::{Style, StyleSheet};

use std::ops::RangeInclusive;

/// A bar that displays progress.
///
/// # Example
/// ```
/// use iced_web::ProgressBar;
///
/// let value = 50.0;
///
/// ProgressBar::new(0.0..=100.0, value);
/// ```
///
/// ![Progress bar](https://user-images.githubusercontent.com/18618951/71662391-a316c200-2d51-11ea-9cef-52758cab85e3.png)
#[allow(missing_debug_implementations)]
pub struct ProgressBar<'a> {
    range: RangeInclusive<f32>,
    value: f32,
    width: Length,
    height: Option<Length>,
    style: Box<dyn StyleSheet + 'a>,
}

impl<'a> ProgressBar<'a> {
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
            style: Default::default(),
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
    pub fn style(mut self, style: impl Into<Box<dyn StyleSheet>>) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, Message> Widget<Message> for ProgressBar<'a> {
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        _bus: &Bus<Message>,
        _style_sheet: &mut Css<'b>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::*;

        let (range_start, range_end) = self.range.clone().into_inner();
        let amount_filled =
            (self.value - range_start) / (range_end - range_start).max(1.0);

        let style = self.style.style();

        let bar = div(bump)
            .attr(
                "style",
                bumpalo::format!(
                    in bump,
                    "width: {}%; height: 100%; background: {}",
                    amount_filled * 100.0,
                    css::background(style.bar)
                )
                .into_bump_str(),
            )
            .finish();

        let node = div(bump).attr(
            "style",
            bumpalo::format!(
                in bump,
                "width: {}; height: {}; background: {}; border-radius: {}px; overflow: hidden;",
                css::length(self.width),
                css::length(self.height.unwrap_or(Length::Units(30))),
                css::background(style.background),
                style.border_radius
            )
            .into_bump_str(),
        ).children(vec![bar]);

        node.finish()
    }
}

impl<'a, Message> From<ProgressBar<'a>> for Element<'a, Message>
where
    Message: 'static,
{
    fn from(container: ProgressBar<'a>) -> Element<'a, Message> {
        Element::new(container)
    }
}
