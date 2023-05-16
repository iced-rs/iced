//! Query or update internal widget state.
pub mod focusable;
pub mod scrollable;
pub mod text_input;

pub use focusable::Focusable;
pub use scrollable::Scrollable;
pub use text_input::TextInput;

use crate::widget::Id;

use std::{any::Any, fmt, rc::Rc};

#[allow(missing_debug_implementations)]
/// A wrapper around an [`Operation`] that can be used for Application Messages and internally in Iced.
pub enum OperationWrapper<M> {
    /// Application Message
    Message(Box<dyn Operation<M>>),
    /// Widget Id
    Id(Box<dyn Operation<crate::widget::Id>>),
    /// Wrapper
    Wrapper(Box<dyn Operation<OperationOutputWrapper<M>>>),
}

#[allow(missing_debug_implementations)]
/// A wrapper around an [`Operation`] output that can be used for Application Messages and internally in Iced.
pub enum OperationOutputWrapper<M> {
    /// Application Message
    Message(M),
    /// Widget Id
    Id(crate::widget::Id),
}

impl<M: 'static> Operation<OperationOutputWrapper<M>> for OperationWrapper<M> {
    fn container(
        &mut self,
        id: Option<&Id>,
        operate_on_children: &mut dyn FnMut(
            &mut dyn Operation<OperationOutputWrapper<M>>,
        ),
    ) {
        match self {
            OperationWrapper::Message(operation) => {
                operation.container(id, &mut |operation| {
                    operate_on_children(&mut MapOperation { operation });
                });
            }
            OperationWrapper::Id(operation) => {
                operation.container(id, &mut |operation| {
                    operate_on_children(&mut MapOperation { operation });
                });
            }
            OperationWrapper::Wrapper(operation) => {
                operation.container(id, operate_on_children);
            }
        }
    }

    fn focusable(&mut self, state: &mut dyn Focusable, id: Option<&Id>) {
        match self {
            OperationWrapper::Message(operation) => {
                operation.focusable(state, id);
            }
            OperationWrapper::Id(operation) => {
                operation.focusable(state, id);
            }
            OperationWrapper::Wrapper(operation) => {
                operation.focusable(state, id);
            }
        }
    }

    fn scrollable(&mut self, state: &mut dyn Scrollable, id: Option<&Id>) {
        match self {
            OperationWrapper::Message(operation) => {
                operation.scrollable(state, id);
            }
            OperationWrapper::Id(operation) => {
                operation.scrollable(state, id);
            }
            OperationWrapper::Wrapper(operation) => {
                operation.scrollable(state, id);
            }
        }
    }

    fn text_input(&mut self, state: &mut dyn TextInput, id: Option<&Id>) {
        match self {
            OperationWrapper::Message(operation) => {
                operation.text_input(state, id);
            }
            OperationWrapper::Id(operation) => {
                operation.text_input(state, id);
            }
            OperationWrapper::Wrapper(operation) => {
                operation.text_input(state, id);
            }
        }
    }

    fn finish(&self) -> Outcome<OperationOutputWrapper<M>> {
        match self {
            OperationWrapper::Message(operation) => match operation.finish() {
                Outcome::None => Outcome::None,
                Outcome::Some(o) => {
                    Outcome::Some(OperationOutputWrapper::Message(o))
                }
                Outcome::Chain(c) => {
                    Outcome::Chain(Box::new(OperationWrapper::Message(c)))
                }
            },
            OperationWrapper::Id(operation) => match operation.finish() {
                Outcome::None => Outcome::None,
                Outcome::Some(id) => {
                    Outcome::Some(OperationOutputWrapper::Id(id))
                }
                Outcome::Chain(c) => {
                    Outcome::Chain(Box::new(OperationWrapper::Id(c)))
                }
            },
            OperationWrapper::Wrapper(c) => c.as_ref().finish(),
        }
    }
}

#[allow(missing_debug_implementations)]
/// Map Operation
pub struct MapOperation<'a, B> {
    /// inner operation
    pub(crate) operation: &'a mut dyn Operation<B>,
}

impl<'a, B> MapOperation<'a, B> {
    /// Creates a new [`MapOperation`].
    pub fn new(operation: &'a mut dyn Operation<B>) -> MapOperation<'a, B> {
        MapOperation { operation }
    }
}

impl<'a, T, B> Operation<T> for MapOperation<'a, B> {
    fn container(
        &mut self,
        id: Option<&Id>,
        operate_on_children: &mut dyn FnMut(&mut dyn Operation<T>),
    ) {
        self.operation.container(id, &mut |operation| {
            operate_on_children(&mut MapOperation { operation });
        });
    }

    fn focusable(&mut self, state: &mut dyn Focusable, id: Option<&Id>) {
        self.operation.focusable(state, id);
    }

    fn scrollable(&mut self, state: &mut dyn Scrollable, id: Option<&Id>) {
        self.operation.scrollable(state, id);
    }

    fn text_input(&mut self, state: &mut dyn TextInput, id: Option<&Id>) {
        self.operation.text_input(state, id)
    }
}

/// A piece of logic that can traverse the widget tree of an application in
/// order to query or update some widget state.
pub trait Operation<T> {
    /// Operates on a widget that contains other widgets.
    ///
    /// The `operate_on_children` function can be called to return control to
    /// the widget tree and keep traversing it.
    fn container(
        &mut self,
        id: Option<&Id>,
        operate_on_children: &mut dyn FnMut(&mut dyn Operation<T>),
    );

    /// Operates on a widget that can be focused.
    fn focusable(&mut self, _state: &mut dyn Focusable, _id: Option<&Id>) {}

    /// Operates on a widget that can be scrolled.
    fn scrollable(&mut self, _state: &mut dyn Scrollable, _id: Option<&Id>) {}

    /// Operates on a widget that has text input.
    fn text_input(&mut self, _state: &mut dyn TextInput, _id: Option<&Id>) {}

    /// Operates on a custom widget.
    fn custom(&mut self, _state: &mut dyn Any, _id: Option<&Id>) {}

    /// Finishes the [`Operation`] and returns its [`Outcome`].
    fn finish(&self) -> Outcome<T> {
        Outcome::None
    }
}

/// Maps the output of an [`Operation`] using the given function.
pub fn map<A, B>(
    operation: Box<dyn Operation<A>>,
    f: impl Fn(A) -> B + 'static,
) -> impl Operation<B>
where
    A: 'static,
    B: 'static,
{
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

                fn text_input(
                    &mut self,
                    state: &mut dyn TextInput,
                    id: Option<&Id>,
                ) {
                    self.operation.text_input(state, id);
                }

                fn custom(&mut self, state: &mut dyn Any, id: Option<&Id>) {
                    self.operation.custom(state, id);
                }
            }

            let Self { operation, .. } = self;

            MapRef {
                operation: operation.as_mut(),
            }
            .container(id, operate_on_children);
        }

        fn focusable(&mut self, state: &mut dyn Focusable, id: Option<&Id>) {
            self.operation.focusable(state, id);
        }

        fn scrollable(&mut self, state: &mut dyn Scrollable, id: Option<&Id>) {
            self.operation.scrollable(state, id);
        }

        fn text_input(&mut self, state: &mut dyn TextInput, id: Option<&Id>) {
            self.operation.text_input(state, id);
        }

        fn custom(&mut self, state: &mut dyn Any, id: Option<&Id>) {
            self.operation.custom(state, id);
        }

        fn finish(&self) -> Outcome<B> {
            match self.operation.finish() {
                Outcome::None => Outcome::None,
                Outcome::Some(output) => Outcome::Some((self.f)(output)),
                Outcome::Chain(next) => Outcome::Chain(Box::new(Map {
                    operation: next,
                    f: self.f.clone(),
                })),
            }
        }
    }

    Map {
        operation,
        f: Rc::new(f),
    }
}

/// The result of an [`Operation`].
pub enum Outcome<T> {
    /// The [`Operation`] produced no result.
    None,

    /// The [`Operation`] produced some result.
    Some(T),

    /// The [`Operation`] needs to be followed by another [`Operation`].
    Chain(Box<dyn Operation<T>>),
}

impl<T> fmt::Debug for Outcome<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "Outcome::None"),
            Self::Some(output) => write!(f, "Outcome::Some({:?})", output),
            Self::Chain(_) => write!(f, "Outcome::Chain(...)"),
        }
    }
}

/// Produces an [`Operation`] that applies the given [`Operation`] to the
/// children of a container with the given [`Id`].
pub fn scoped<T: 'static>(
    target: Id,
    operation: impl Operation<T> + 'static,
) -> impl Operation<T> {
    struct ScopedOperation<Message> {
        target: Id,
        operation: Box<dyn Operation<Message>>,
    }

    impl<Message: 'static> Operation<Message> for ScopedOperation<Message> {
        fn container(
            &mut self,
            id: Option<&Id>,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<Message>),
        ) {
            if id == Some(&self.target) {
                operate_on_children(self.operation.as_mut());
            } else {
                operate_on_children(self);
            }
        }

        fn finish(&self) -> Outcome<Message> {
            match self.operation.finish() {
                Outcome::Chain(next) => {
                    Outcome::Chain(Box::new(ScopedOperation {
                        target: self.target.clone(),
                        operation: next,
                    }))
                }
                outcome => outcome,
            }
        }
    }

    ScopedOperation {
        target,
        operation: Box::new(operation),
    }
}
