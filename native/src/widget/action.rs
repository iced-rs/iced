use crate::widget::operation::{self, Focusable, Operation, Scrollable};
use crate::widget::Id;

use iced_futures::MaybeSend;

use std::rc::Rc;

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
            f: Rc::new(f),
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
    f: Rc<dyn Fn(A) -> B>,
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
        struct MapRef<'a, A> {
            operation: &'a mut dyn Operation<A>,
        }

        impl<'a, A, B> Operation<B> for MapRef<'a, A> {
            fn container(
                &mut self,
                id: Option<&Id>,
                operate_on_children: &mut dyn FnMut(&mut dyn Operation<B>),
            ) {
                let Self { operation, .. } = self;

                operation.container(id, &mut |operation| {
                    operate_on_children(&mut MapRef { operation });
                });
            }

            fn scrollable(
                &mut self,
                state: &mut dyn Scrollable,
                id: Option<&Id>,
            ) {
                self.operation.scrollable(state, id);
            }

            fn focusable(
                &mut self,
                state: &mut dyn Focusable,
                id: Option<&Id>,
            ) {
                self.operation.focusable(state, id);
            }
        }

        let Self { operation, .. } = self;

        MapRef {
            operation: operation.as_mut(),
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

    fn text_input(
        &mut self,
        state: &mut dyn operation::TextInput,
        id: Option<&Id>,
    ) {
        self.operation.text_input(state, id);
    }

    fn finish(&self) -> operation::Outcome<B> {
        match self.operation.finish() {
            operation::Outcome::None => operation::Outcome::None,
            operation::Outcome::Some(output) => {
                operation::Outcome::Some((self.f)(output))
            }
            operation::Outcome::Chain(next) => {
                operation::Outcome::Chain(Box::new(Map {
                    operation: next,
                    f: self.f.clone(),
                }))
            }
        }
    }
}
