//! Display an interactive selector of a single value from a range of values.
//!
//! A [`Slider`] has some local [`State`].
use crate::{Bus, Css, Element, Length, Widget};

pub use iced_style::slider::{Handle, HandleShape, Style, StyleSheet};

use dodrio::bumpalo;
use std::{ops::RangeInclusive, rc::Rc};

/// An horizontal bar and a handle that selects a single value from a range of
/// values.
///
/// A [`Slider`] will try to fill the horizontal space of its container.
///
/// The [`Slider`] range of numeric values is generic and its step size defaults
/// to 1 unit.
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
    #[allow(dead_code)]
    width: Length,
    #[allow(dead_code)]
    style_sheet: Box<dyn StyleSheet + 'a>,
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
            style_sheet: Default::default(),
        }
    }

    /// Sets the width of the [`Slider`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the style of the [`Slider`].
    pub fn style(
        mut self,
        style_sheet: impl Into<Box<dyn StyleSheet + 'a>>,
    ) -> Self {
        self.style_sheet = style_sheet.into();
        self
    }

    /// Sets the step size of the [`Slider`].
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
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
        _style_sheet: &mut Css<'b>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::*;
        use wasm_bindgen::JsCast;

        let (start, end) = self.range.clone().into_inner();

        let min = bumpalo::format!(in bump, "{}", start.into());
        let max = bumpalo::format!(in bump, "{}", end.into());
        let value = bumpalo::format!(in bump, "{}", self.value.into());
        let step = bumpalo::format!(in bump, "{}", self.step.into());

        let on_change = self.on_change.clone();
        let event_bus = bus.clone();

        // TODO: Styling
        input(bump)
            .attr("type", "range")
            .attr("step", step.into_bump_str())
            .attr("min", min.into_bump_str())
            .attr("max", max.into_bump_str())
            .attr("value", value.into_bump_str())
            .attr("style", "width: 100%")
            .on("input", move |_root, _vdom, event| {
                let slider = match event.target().and_then(|t| {
                    t.dyn_into::<web_sys::HtmlInputElement>().ok()
                }) {
                    None => return,
                    Some(slider) => slider,
                };

                if let Ok(value) = slider.value().parse::<f64>() {
                    if let Some(value) = T::from_f64(value) {
                        event_bus.publish(on_change(value));
                    }
                }
            })
            .finish()
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State;

impl State {
    /// Creates a new [`State`].
    pub fn new() -> Self {
        Self
    }
}
