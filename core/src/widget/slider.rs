//! Display an interactive selector of a single value from a range of values.
//!
//! A [`Slider`] has some local [`State`].
//!
//! [`Slider`]: struct.Slider.html
//! [`State`]: struct.State.html
use crate::Length;

use std::ops::RangeInclusive;
use std::rc::Rc;

/// An horizontal bar and a handle that selects a single value from a range of
/// values.
///
/// A [`Slider`] will try to fill the horizontal space of its container.
///
/// [`Slider`]: struct.Slider.html
///
/// # Example
/// ```
/// use iced_core::{slider, Slider};
///
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
#[allow(missing_docs)]
pub struct Slider<'a, Message> {
    pub state: &'a mut State,
    pub range: RangeInclusive<f32>,
    pub value: f32,
    pub on_change: Rc<Box<dyn Fn(f32) -> Message>>,
    pub width: Length,
}

impl<'a, Message> std::fmt::Debug for Slider<'a, Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Slider")
            .field("state", &self.state)
            .field("range", &self.range)
            .field("value", &self.value)
            .field("width", &self.width)
            .finish()
    }
}

impl<'a, Message> Slider<'a, Message> {
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
        range: RangeInclusive<f32>,
        value: f32,
        on_change: F,
    ) -> Self
    where
        F: 'static + Fn(f32) -> Message,
    {
        Slider {
            state,
            value: value.max(*range.start()).min(*range.end()),
            range,
            on_change: Rc::new(Box::new(on_change)),
            width: Length::Fill,
        }
    }

    /// Sets the width of the [`Slider`].
    ///
    /// [`Slider`]: struct.Slider.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }
}

/// The local state of a [`Slider`].
///
/// [`Slider`]: struct.Slider.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    /// Whether the [`Slider`] is currently being dragged or not.
    ///
    /// [`Slider`]: struct.Slider.html
    pub is_dragging: bool,
}

impl State {
    /// Creates a new [`State`].
    ///
    /// [`State`]: struct.State.html
    pub fn new() -> State {
        State::default()
    }

    /// Returns whether the associated [`Slider`] is currently being dragged or
    /// not.
    ///
    /// [`Slider`]: struct.Slider.html
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }
}
