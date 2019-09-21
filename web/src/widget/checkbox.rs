use crate::{Bus, Element, Widget};

use dodrio::bumpalo;

pub use iced_core::Checkbox;

impl<Message> Widget<Message> for Checkbox<Message>
where
    Message: 'static + Copy,
{
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::*;

        let checkbox_label = bumpalo::format!(in bump, "{}", self.label);

        let event_bus = bus.clone();
        let msg = (self.on_toggle)(!self.is_checked);

        // TODO: Complete styling
        label(bump)
            .children(vec![
                input(bump)
                    .attr("type", "checkbox")
                    .bool_attr("checked", self.is_checked)
                    .on("click", move |root, vdom, _event| {
                        event_bus.publish(msg, root);

                        vdom.schedule_render();
                    })
                    .finish(),
                text(checkbox_label.into_bump_str()),
            ])
            .finish()
    }
}

impl<'a, Message> From<Checkbox<Message>> for Element<'a, Message>
where
    Message: 'static + Copy,
{
    fn from(checkbox: Checkbox<Message>) -> Element<'a, Message> {
        Element::new(checkbox)
    }
}
