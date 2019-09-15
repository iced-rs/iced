use crate::{Bus, Element, Widget};

use dodrio::bumpalo;

pub type Slider<'a, Message> = iced::Slider<'a, Message>;

pub use iced::slider::State;

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
