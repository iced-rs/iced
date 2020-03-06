//! Display an interactive selector of a single value from a range of values.
//!
//! A [`Slider`] has some local [`State`].
//!
//! [`Slider`]: struct.Slider.html
//! [`State`]: struct.State.html
use crate::{style, Bus, Element, Length, Widget};

use dodrio::bumpalo;
use std::{ops::RangeInclusive, rc::Rc};

/// An horizontal bar and a handle that selects a single value from a range of
/// values.
///
/// A [`Slider`] will try to fill the horizontal space of its container.
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
pub struct Slider<'a, Message> {
    _state: &'a mut State,
    range: RangeInclusive<f32>,
    value: f32,
    on_change: Rc<Box<dyn Fn(f32) -> Message>>,
    width: Length,
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
            _state: state,
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

impl<'a, Message> Widget<Message> for Slider<'a, Message>
where
    Message: 'static + Clone,
{
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
        _style_sheet: &mut style::Sheet<'b>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::*;
        use wasm_bindgen::JsCast;

        let (start, end) = self.range.clone().into_inner();

        let min = bumpalo::format!(in bump, "{}", start);
        let max = bumpalo::format!(in bump, "{}", end);
        let value = bumpalo::format!(in bump, "{}", self.value);

        let on_change = self.on_change.clone();
        let event_bus = bus.clone();

        // TODO: Make `step` configurable
        // TODO: Complete styling
        input(bump)
            .attr("type", "range")
            .attr("step", "0.01")
            .attr("min", min.into_bump_str())
            .attr("max", max.into_bump_str())
            .attr("value", value.into_bump_str())
            .attr("style", "width: 100%")
            .on("input", move |root, vdom, event| {
                let slider = match event.target().and_then(|t| {
                    t.dyn_into::<web_sys::HtmlInputElement>().ok()
                }) {
                    None => return,
                    Some(slider) => slider,
                };

                if let Ok(value) = slider.value().parse::<f32>() {
                    event_bus.publish(on_change(value), root);
                    vdom.schedule_render();
                }
            })
            .finish()
    }
}

impl<'a, Message> From<Slider<'a, Message>> for Element<'a, Message>
where
    Message: 'static + Clone,
{
    fn from(slider: Slider<'a, Message>) -> Element<'a, Message> {
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
