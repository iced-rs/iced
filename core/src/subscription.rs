//! Generate events asynchronously for you application.

/// An event subscription.
pub struct Subscription<T> {
    definitions: Vec<Box<dyn Definition<Message = T>>>,
}

impl<T> Subscription<T> {
    pub fn none() -> Self {
        Self {
            definitions: Vec::new(),
        }
    }

    pub fn batch(subscriptions: impl Iterator<Item = Subscription<T>>) -> Self {
        Self {
            definitions: subscriptions
                .flat_map(|subscription| subscription.definitions)
                .collect(),
        }
    }

    pub fn definitions(self) -> Vec<Box<dyn Definition<Message = T>>> {
        self.definitions
    }
}

impl<T, A> From<A> for Subscription<T>
where
    A: Definition<Message = T> + 'static,
{
    fn from(definition: A) -> Self {
        Self {
            definitions: vec![Box::new(definition)],
        }
    }
}

/// The definition of an event subscription.
pub trait Definition {
    type Message;

    fn id(&self) -> u64;

    fn stream(
        &self,
    ) -> (
        futures::stream::BoxStream<'static, Self::Message>,
        Box<dyn Handle>,
    );
}

pub trait Handle {
    fn cancel(&mut self);
}

impl<T> std::fmt::Debug for Subscription<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command").finish()
    }
}
