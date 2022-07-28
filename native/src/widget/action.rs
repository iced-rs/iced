use crate::widget::state;
use crate::widget::{Id, Operation};

use iced_futures::MaybeSend;

pub struct Action<T>(Box<dyn Operation<T>>);

impl<T> Action<T> {
    pub fn new(operation: impl Operation<T> + 'static) -> Self {
        Self(Box::new(operation))
    }

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

    pub fn into_operation(self) -> Box<dyn Operation<T>> {
        self.0
    }
}

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

    fn focusable(&mut self, state: &mut dyn state::Focusable, id: Option<&Id>) {
        self.operation.focusable(state, id);
    }
}
