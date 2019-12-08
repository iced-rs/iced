//! Generate events asynchronously for you application.

/// An event subscription.
pub struct Subscription<I, O> {
    connections: Vec<Box<dyn Connection<Input = I, Output = O>>>,
}

impl<I, O> Subscription<I, O> {
    pub fn none() -> Self {
        Self {
            connections: Vec::new(),
        }
    }

    pub fn batch(
        subscriptions: impl Iterator<Item = Subscription<I, O>>,
    ) -> Self {
        Self {
            connections: subscriptions
                .flat_map(|subscription| subscription.connections)
                .collect(),
        }
    }

    pub fn connections(
        self,
    ) -> Vec<Box<dyn Connection<Input = I, Output = O>>> {
        self.connections
    }
}

impl<I, O, T> From<T> for Subscription<I, O>
where
    T: Connection<Input = I, Output = O> + 'static,
{
    fn from(handle: T) -> Self {
        Self {
            connections: vec![Box::new(handle)],
        }
    }
}

/// The connection of an event subscription.
pub trait Connection {
    type Input;
    type Output;

    fn id(&self) -> u64;

    fn stream(
        &self,
        input: Self::Input,
    ) -> futures::stream::BoxStream<'static, Self::Output>;
}

impl<I, O> std::fmt::Debug for Subscription<I, O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subscription").finish()
    }
}
