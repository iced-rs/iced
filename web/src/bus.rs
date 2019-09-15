use crate::Application;

use std::rc::Rc;

#[derive(Clone)]
pub struct Bus<Message> {
    publish: Rc<Box<dyn Fn(Message, &mut dyn dodrio::RootRender)>>,
}

impl<Message> Bus<Message>
where
    Message: 'static,
{
    pub fn new() -> Self {
        Self {
            publish: Rc::new(Box::new(|message, root| {
                let app = root.unwrap_mut::<Application<Message>>();

                app.update(message)
            })),
        }
    }

    pub fn publish(&self, message: Message, root: &mut dyn dodrio::RootRender) {
        (self.publish)(message, root);
    }

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
