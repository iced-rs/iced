//! Display an interactive selector of a single value from a range of values.
//!
//! A [`Slider`] has some local [`State`].
//!
//! [`Slider`]: struct.Slider.html
//! [`State`]: struct.State.html
use crate::{Element, Length, Widget, Hasher, layout};

pub use iced_style::slider::{Handle, HandleShape, Style, StyleSheet};

use std::{ops::RangeInclusive, hash::Hash, rc::Rc};

/// An horizontal bar and a handle that selects a single value from a range of
/// values.
///
/// A [`Slider`] will try to fill the horizontal space of its container.
///
/// The [`Slider`] range of numeric values is generic and its step size defaults
/// to 1 unit.
///
/// [`Slider`]: struct.Slider.html
///
/// # Example
/// ```
/// # use iced_web::{slider, Slider};
/// #
/// pub enum Message {
///     SliderChanged(f32),
/// }
///
/// let state = &mut slider::State::new();
/// let value = 50.0;
///
/// Slider::new(state, 0.0..=100.0, value, Message::SliderChanged);
/// ```
///
/// ![Slider drawn by Coffee's renderer](https://github.com/hecrj/coffee/blob/bda9818f823dfcb8a7ad0ff4940b4d4b387b5208/images/ui/slider.png?raw=true)
#[allow(missing_debug_implementations)]
pub struct Slider<'a, T, Message> {
    _state: &'a mut State,
    range: RangeInclusive<T>,
    step: T,
    value: T,
    on_change: Rc<Box<dyn Fn(T) -> Message>>,
    width: Length,
    style: Box<dyn StyleSheet>,
}

impl<'a, T, Message> Slider<'a, T, Message>
where
    T: Copy + From<u8> + std::cmp::PartialOrd,
{
    /// Creates a new [`Slider`].
    ///
    /// It expects:
    ///   * the local [`State`] of the [`Slider`]
    ///   * an inclusive range of possible values
    ///   * the current value of the [`Slider`]
    ///   * a function that will be called when the [`Slider`] is dragged.
    ///   It receives the new value of the [`Slider`] and must produce a
    ///   `Message`.
    ///
    /// [`Slider`]: struct.Slider.html
    /// [`State`]: struct.State.html
    pub fn new<F>(
        state: &'a mut State,
        range: RangeInclusive<T>,
        value: T,
        on_change: F,
    ) -> Self
    where
        F: 'static + Fn(T) -> Message,
    {
        let value = if value >= *range.start() {
            value
        } else {
            *range.start()
        };

        let value = if value <= *range.end() {
            value
        } else {
            *range.end()
        };

        Slider {
            _state: state,
            value,
            range,
            step: T::from(1),
            on_change: Rc::new(Box::new(on_change)),
            width: Length::Fill,
            style: Default::default(),
        }
    }

    /// Sets the width of the [`Slider`].
    ///
    /// [`Slider`]: struct.Slider.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the style of the [`Slider`].
    ///
    /// [`Slider`]: struct.Slider.html
    pub fn style(mut self, style: impl Into<Box<dyn StyleSheet>>) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the step size of the [`Slider`].
    ///
    /// [`Slider`]: struct.Slider.html
    pub fn step(mut self, step: T) -> Self {
        self.step = step;
        self
    }
}

impl<'a, T, Message> Widget<Message> for Slider<'a, T, Message>
where
    T: 'static + Copy + Into<f64> + num_traits::FromPrimitive,
    Message: 'static,
{
    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
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

impl<'a, T, Message> From<Slider<'a, T, Message>> for Element<'a, Message>
where
    T: 'static + Copy + Into<f64> + num_traits::FromPrimitive,
    Message: 'static,
{
    fn from(slider: Slider<'a, T, Message>) -> Element<'a, Message> {
        Element::new(slider)
    }
}

/// The local state of a [`Slider`].
///
/// [`Slider`]: struct.Slider.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State;

impl State {
    /// Creates a new [`State`].
    ///
    /// [`State`]: struct.State.html
    pub fn new() -> Self {
        Self
    }
}
