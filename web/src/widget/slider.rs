use crate::{Bus, Element, Length, Widget};

use dodrio::bumpalo;
use std::{ops::RangeInclusive, rc::Rc};

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
    Message: 'static + Copy,
{
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
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
        label(bump)
            .children(vec![input(bump)
                .attr("type", "range")
                .attr("step", "0.01")
                .attr("min", min.into_bump_str())
                .attr("max", max.into_bump_str())
                .attr("value", value.into_bump_str())
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
                .finish()])
            .finish()
    }
}

impl<'a, Message> From<Slider<'a, Message>> for Element<'a, Message>
where
    Message: 'static + Copy,
{
    fn from(slider: Slider<'a, Message>) -> Element<'a, Message> {
        Element::new(slider)
    }
}

pub struct State;

impl State {
    pub fn new() -> Self {
        Self
    }
}
