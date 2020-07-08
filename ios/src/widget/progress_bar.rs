//! Provide progress feedback to your users.
use crate::{Element, Length, Widget, Hasher, layout};
use std::hash::Hash;

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
pub struct ProgressBar {
    range: RangeInclusive<f32>,
    value: f32,
    width: Length,
    height: Option<Length>,
    style: Box<dyn StyleSheet>,
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
            height: None,
            style: Default::default(),
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
    pub fn style(mut self, style: impl Into<Box<dyn StyleSheet>>) -> Self {
        self.style = style.into();
        self
    }
}

impl<Message> Widget<Message> for ProgressBar {
    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);
        self.width.hash(state);
        self.height.hash(state);
    }
    fn layout(
        &self,
        limits: &layout::Limits,
    ) -> layout::Node {
        todo!();
    }

    fn width(&self) -> Length {
        todo!();
    }

    fn height(&self) -> Length {
        todo!();
    }
}

impl<'a, Message> From<ProgressBar> for Element<'a, Message>
where
    Message: 'static,
{
    fn from(container: ProgressBar) -> Element<'a, Message> {
        Element::new(container)
    }
}
