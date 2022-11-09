use crate::widget::operation::{self, Operation};
use crate::widget::Id;

use iced_futures::MaybeSend;

/// An operation to be performed on the widget tree.
#[allow(missing_debug_implementations)]
pub struct Action<T>(Box<dyn Operation<T>>);

impl<T> Action<T> {
    /// Creates a new [`Action`] with the given [`Operation`].
    pub fn new(operation: impl Operation<T> + 'static) -> Self {
        Self(Box::new(operation))
    }

    /// Maps the output of an [`Action`] using the given function.
    pub fn map<A>(
        self,
        f: impl Fn(T) -> A + 'static + MaybeSend + Sync,
    ) -> Action<A>
    where
        T: 'static,
        A: 'static,
    {
        Action(Box::new(Map {
            operation: self.0,
            f: Box::new(f),
        }))
    }

    /// Consumes the [`Action`] and returns the internal [`Operation`].
    pub fn into_operation(self) -> Box<dyn Operation<T>> {
        self.0
    }
}

#[allow(missing_debug_implementations)]
struct Map<A, B> {
    operation: Box<dyn Operation<A>>,
    f: Box<dyn Fn(A) -> B>,
}

impl<A, B> Operation<B> for Map<A, B>
where
    A: 'static,
    B: 'static,
{
    fn container(
        &mut self,
        id: Option<&Id>,
        operate_on_children: &mut dyn FnMut(&mut dyn Operation<B>),
    ) {
        struct MapRef<'a, A, B> {
            operation: &'a mut dyn Operation<A>,
            f: &'a dyn Fn(A) -> B,
        }

        impl<'a, A, B> Operation<B> for MapRef<'a, A, B> {
            fn container(
                &mut self,
                id: Option<&Id>,
                operate_on_children: &mut dyn FnMut(&mut dyn Operation<B>),
            ) {
                let Self { operation, f } = self;

                operation.container(id, &mut |operation| {
                    operate_on_children(&mut MapRef { operation, f });
                });
            }
        }

        let Self { operation, f } = self;

        MapRef {
            operation: operation.as_mut(),
            f,
        }
        .container(id, operate_on_children);
    }

    fn focusable(
        &mut self,
        state: &mut dyn operation::Focusable,
        id: Option<&Id>,
    ) {
        self.operation.focusable(state, id);
    }

    fn scrollable(
        &mut self,
        state: &mut dyn operation::Scrollable,
        id: Option<&Id>,
    ) {
        self.operation.scrollable(state, id);
    }
}
