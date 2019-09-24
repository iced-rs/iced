use crate::{Bus, Element, Widget};

use dodrio::bumpalo;

pub use iced_core::Radio;

impl<Message> Widget<Message> for Radio<Message>
where
    Message: 'static + Copy,
{
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::*;

        let radio_label = bumpalo::format!(in bump, "{}", self.label);

        let event_bus = bus.clone();
        let on_click = self.on_click;

        // TODO: Complete styling
        label(bump)
            .attr("style", "display: block")
            .children(vec![
                input(bump)
                    .attr("type", "radio")
                    .bool_attr("checked", self.is_selected)
                    .on("click", move |root, vdom, _event| {
                        event_bus.publish(on_click, root);

                        vdom.schedule_render();
                    })
                    .finish(),
                text(radio_label.into_bump_str()),
            ])
            .finish()
    }
}

impl<'a, Message> From<Radio<Message>> for Element<'a, Message>
where
    Message: 'static + Copy,
{
    fn from(radio: Radio<Message>) -> Element<'a, Message> {
        Element::new(radio)
    }
}
