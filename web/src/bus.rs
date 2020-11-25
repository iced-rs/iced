use iced_futures::futures::channel::mpsc;
use std::rc::Rc;

/// A publisher of messages.
///
/// It can be used to route messages back to the [`Application`].
///
/// [`Application`]: crate::Application
#[allow(missing_debug_implementations)]
pub struct Bus<Message> {
    publish: Rc<Box<dyn Fn(Message) -> ()>>,
}

impl<Message> Clone for Bus<Message> {
    fn clone(&self) -> Self {
        Bus {
            publish: self.publish.clone(),
        }
    }
}

impl<Message> Bus<Message>
where
    Message: 'static,
{
    pub(crate) fn new(publish: mpsc::UnboundedSender<Message>) -> Self {
        Self {
            publish: Rc::new(Box::new(move |message| {
                publish.unbounded_send(message).expect("Send message");
            })),
        }
    }

    /// Publishes a new message for the [`Application`].
    ///
    /// [`Application`]: crate::Application
    pub fn publish(&self, message: Message) {
        (self.publish)(message)
    }

    /// Creates a new [`Bus`] that applies the given function to the messages
    /// before publishing.
    pub fn map<B>(&self, mapper: Rc<Box<dyn Fn(B) -> Message>>) -> Bus<B>
    where
        B: 'static,
    {
        let publish = self.publish.clone();

        Bus {
            publish: Rc::new(Box::new(move |message| publish(mapper(message)))),
        }
    }
}
