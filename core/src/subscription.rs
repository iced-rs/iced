//! Generate events asynchronously for you application.

/// An event subscription.
pub struct Subscription<T> {
    handles: Vec<Box<dyn Handle<Output = T>>>,
}

impl<T> Subscription<T> {
    pub fn none() -> Self {
        Self {
            handles: Vec::new(),
        }
    }

    pub fn batch(subscriptions: impl Iterator<Item = Subscription<T>>) -> Self {
        Self {
            handles: subscriptions
                .flat_map(|subscription| subscription.handles)
                .collect(),
        }
    }

    pub fn handles(self) -> Vec<Box<dyn Handle<Output = T>>> {
        self.handles
    }
}

impl<T, A> From<A> for Subscription<T>
where
    A: Handle<Output = T> + 'static,
{
    fn from(handle: A) -> Self {
        Self {
            handles: vec![Box::new(handle)],
        }
    }
}

/// The handle of an event subscription.
pub trait Handle {
    type Output;

    fn id(&self) -> u64;

    fn stream(&self) -> futures::stream::BoxStream<'static, Self::Output>;
}

impl<T> std::fmt::Debug for Subscription<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subscription").finish()
    }
}
