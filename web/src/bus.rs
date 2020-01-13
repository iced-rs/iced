use crate::Instance;

use std::rc::Rc;

/// A publisher of messages.
///
/// It can be used to route messages back to the [`Application`].
///
/// [`Application`]: trait.Application.html
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct Bus<Message> {
    publish: Rc<Box<dyn Fn(Message, &mut dyn dodrio::RootRender)>>,
}

impl<Message> Bus<Message>
where
    Message: 'static,
{
    pub(crate) fn new() -> Self {
        Self {
            publish: Rc::new(Box::new(|message, root| {
                let app = root.unwrap_mut::<Instance<Message>>();

                app.update(message)
            })),
        }
    }

    /// Publishes a new message for the [`Application`].
    ///
    /// [`Application`]: trait.Application.html
    pub fn publish(&self, message: Message, root: &mut dyn dodrio::RootRender) {
        (self.publish)(message, root);
    }

    /// Creates a new [`Bus`] that applies the given function to the messages
    /// before publishing.
    ///
    /// [`Bus`]: struct.Bus.html
    pub fn map<B>(&self, mapper: Rc<Box<dyn Fn(B) -> Message>>) -> Bus<B>
    where
        B: 'static,
    {
        let publish = self.publish.clone();

        Bus {
            publish: Rc::new(Box::new(move |message, root| {
                publish(mapper(message), root)
            })),
        }
    }
}
