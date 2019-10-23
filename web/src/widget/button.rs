use crate::{Bus, Element, Widget};

use dodrio::bumpalo;

pub use iced_core::button::State;

pub type Button<'a, Message> =
    iced_core::Button<'a, Message, Element<'a, Message>>;

impl<'a, Message> Widget<Message> for Button<'a, Message>
where
    Message: 'static + Copy,
{
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::*;

        let mut node =
            button(bump).children(vec![self.content.node(bump, bus)]);

        if let Some(on_press) = self.on_press {
            let event_bus = bus.clone();

            node = node.on("click", move |root, vdom, _event| {
                event_bus.publish(on_press, root);

                vdom.schedule_render();
            });
        }

        // TODO: Complete styling

        node.finish()
    }
}

impl<'a, Message> From<Button<'a, Message>> for Element<'a, Message>
where
    Message: 'static + Copy,
{
    fn from(button: Button<'a, Message>) -> Element<'a, Message> {
        Element::new(button)
    }
}
