//! Query or update internal widget state.
pub mod focusable;
pub mod scrollable;
pub mod text_input;

pub use focusable::Focusable;
pub use scrollable::Scrollable;
pub use text_input::TextInput;

use crate::widget::Id;
use crate::{Rectangle, Vector};

use std::any::Any;
use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

/// A piece of logic that can traverse the widget tree of an application in
/// order to query or update some widget state.
pub trait Operation<T = ()>: Send {
    /// Operates on a widget that contains other widgets.
    ///
    /// The `operate_on_children` function can be called to return control to
    /// the widget tree and keep traversing it.
    fn container(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        operate_on_children: &mut dyn FnMut(&mut dyn Operation<T>),
    );

    /// Operates on a widget that can be focused.
    fn focusable(
        &mut self,
        _id: Option<&Id>,
        _bounds: Rectangle,
        _state: &mut dyn Focusable,
    ) {
    }

    /// Operates on a widget that can be scrolled.
    fn scrollable(
        &mut self,
        _id: Option<&Id>,
        _bounds: Rectangle,
        _content_bounds: Rectangle,
        _translation: Vector,
        _state: &mut dyn Scrollable,
    ) {
    }

    /// Operates on a widget that has text input.
    fn text_input(
        &mut self,
        _id: Option<&Id>,
        _bounds: Rectangle,
        _state: &mut dyn TextInput,
    ) {
    }

    /// Operates on a widget that contains some text.
    fn text(&mut self, _id: Option<&Id>, _bounds: Rectangle, _text: &str) {}

    /// Operates on a custom widget with some state.
    fn custom(
        &mut self,
        _id: Option<&Id>,
        _bounds: Rectangle,
        _state: &mut dyn Any,
    ) {
    }

    /// Finishes the [`Operation`] and returns its [`Outcome`].
    fn finish(&self) -> Outcome<T> {
        Outcome::None
    }
}

impl<T, O> Operation<O> for Box<T>
where
    T: Operation<O> + ?Sized,
{
    fn container(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        operate_on_children: &mut dyn FnMut(&mut dyn Operation<O>),
    ) {
        self.as_mut().container(id, bounds, operate_on_children);
    }

    fn focusable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        state: &mut dyn Focusable,
    ) {
        self.as_mut().focusable(id, bounds, state);
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        translation: Vector,
        state: &mut dyn Scrollable,
    ) {
        self.as_mut().scrollable(
            id,
            bounds,
            content_bounds,
            translation,
            state,
        );
    }

    fn text_input(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        state: &mut dyn TextInput,
    ) {
        self.as_mut().text_input(id, bounds, state);
    }

    fn text(&mut self, id: Option<&Id>, bounds: Rectangle, text: &str) {
        self.as_mut().text(id, bounds, text);
    }

    fn custom(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        state: &mut dyn Any,
    ) {
        self.as_mut().custom(id, bounds, state);
    }

    fn finish(&self) -> Outcome<O> {
        self.as_ref().finish()
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
            Self::Some(output) => write!(f, "Outcome::Some({output:?})"),
            Self::Chain(_) => write!(f, "Outcome::Chain(...)"),
        }
    }
}

/// Wraps the [`Operation`] in a black box, erasing its returning type.
pub fn black_box<'a, T, O>(
    operation: &'a mut dyn Operation<T>,
) -> impl Operation<O> + 'a
where
    T: 'a,
{
    struct BlackBox<'a, T> {
        operation: &'a mut dyn Operation<T>,
    }

    impl<T, O> Operation<O> for BlackBox<'_, T> {
        fn container(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<O>),
        ) {
            self.operation.container(id, bounds, &mut |operation| {
                operate_on_children(&mut BlackBox { operation });
            });
        }

        fn focusable(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            state: &mut dyn Focusable,
        ) {
            self.operation.focusable(id, bounds, state);
        }

        fn scrollable(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            content_bounds: Rectangle,
            translation: Vector,
            state: &mut dyn Scrollable,
        ) {
            self.operation.scrollable(
                id,
                bounds,
                content_bounds,
                translation,
                state,
            );
        }

        fn text_input(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            state: &mut dyn TextInput,
        ) {
            self.operation.text_input(id, bounds, state);
        }

        fn text(&mut self, id: Option<&Id>, bounds: Rectangle, text: &str) {
            self.operation.text(id, bounds, text);
        }

        fn custom(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            state: &mut dyn Any,
        ) {
            self.operation.custom(id, bounds, state);
        }

        fn finish(&self) -> Outcome<O> {
            Outcome::None
        }
    }

    BlackBox { operation }
}

/// Maps the output of an [`Operation`] using the given function.
pub fn map<A, B>(
    operation: impl Operation<A>,
    f: impl Fn(A) -> B + Send + Sync + 'static,
) -> impl Operation<B>
where
    A: 'static,
    B: 'static,
{
    #[allow(missing_debug_implementations)]
    struct Map<O, A, B> {
        operation: O,
        f: Arc<dyn Fn(A) -> B + Send + Sync>,
    }

    impl<O, A, B> Operation<B> for Map<O, A, B>
    where
        O: Operation<A>,
        A: 'static,
        B: 'static,
    {
        fn container(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<B>),
        ) {
            struct MapRef<'a, A> {
                operation: &'a mut dyn Operation<A>,
            }

            impl<A, B> Operation<B> for MapRef<'_, A> {
                fn container(
                    &mut self,
                    id: Option<&Id>,
                    bounds: Rectangle,
                    operate_on_children: &mut dyn FnMut(&mut dyn Operation<B>),
                ) {
                    let Self { operation, .. } = self;

                    operation.container(id, bounds, &mut |operation| {
                        operate_on_children(&mut MapRef { operation });
                    });
                }

                fn scrollable(
                    &mut self,
                    id: Option<&Id>,
                    bounds: Rectangle,
                    content_bounds: Rectangle,
                    translation: Vector,
                    state: &mut dyn Scrollable,
                ) {
                    self.operation.scrollable(
                        id,
                        bounds,
                        content_bounds,
                        translation,
                        state,
                    );
                }

                fn focusable(
                    &mut self,
                    id: Option<&Id>,
                    bounds: Rectangle,
                    state: &mut dyn Focusable,
                ) {
                    self.operation.focusable(id, bounds, state);
                }

                fn text_input(
                    &mut self,
                    id: Option<&Id>,
                    bounds: Rectangle,
                    state: &mut dyn TextInput,
                ) {
                    self.operation.text_input(id, bounds, state);
                }

                fn text(
                    &mut self,
                    id: Option<&Id>,
                    bounds: Rectangle,
                    text: &str,
                ) {
                    self.operation.text(id, bounds, text);
                }

                fn custom(
                    &mut self,
                    id: Option<&Id>,
                    bounds: Rectangle,
                    state: &mut dyn Any,
                ) {
                    self.operation.custom(id, bounds, state);
                }
            }

            let Self { operation, .. } = self;

            MapRef { operation }.container(id, bounds, operate_on_children);
        }

        fn focusable(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            state: &mut dyn Focusable,
        ) {
            self.operation.focusable(id, bounds, state);
        }

        fn scrollable(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            content_bounds: Rectangle,
            translation: Vector,
            state: &mut dyn Scrollable,
        ) {
            self.operation.scrollable(
                id,
                bounds,
                content_bounds,
                translation,
                state,
            );
        }

        fn text_input(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            state: &mut dyn TextInput,
        ) {
            self.operation.text_input(id, bounds, state);
        }

        fn text(&mut self, id: Option<&Id>, bounds: Rectangle, text: &str) {
            self.operation.text(id, bounds, text);
        }

        fn custom(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            state: &mut dyn Any,
        ) {
            self.operation.custom(id, bounds, state);
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
        f: Arc::new(f),
    }
}

/// Chains the output of an [`Operation`] with the provided function to
/// build a new [`Operation`].
pub fn then<A, B, O>(
    operation: impl Operation<A> + 'static,
    f: fn(A) -> O,
) -> impl Operation<B>
where
    A: 'static,
    B: Send + 'static,
    O: Operation<B> + 'static,
{
    struct Chain<T, O, A, B>
    where
        T: Operation<A>,
        O: Operation<B>,
    {
        operation: T,
        next: fn(A) -> O,
        _result: PhantomData<B>,
    }

    impl<T, O, A, B> Operation<B> for Chain<T, O, A, B>
    where
        T: Operation<A> + 'static,
        O: Operation<B> + 'static,
        A: 'static,
        B: Send + 'static,
    {
        fn container(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<B>),
        ) {
            self.operation.container(id, bounds, &mut |operation| {
                operate_on_children(&mut black_box(operation));
            });
        }

        fn focusable(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            state: &mut dyn Focusable,
        ) {
            self.operation.focusable(id, bounds, state);
        }

        fn scrollable(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            content_bounds: Rectangle,
            translation: crate::Vector,
            state: &mut dyn Scrollable,
        ) {
            self.operation.scrollable(
                id,
                bounds,
                content_bounds,
                translation,
                state,
            );
        }

        fn text_input(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            state: &mut dyn TextInput,
        ) {
            self.operation.text_input(id, bounds, state);
        }

        fn text(&mut self, id: Option<&Id>, bounds: Rectangle, text: &str) {
            self.operation.text(id, bounds, text);
        }

        fn custom(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            state: &mut dyn Any,
        ) {
            self.operation.custom(id, bounds, state);
        }

        fn finish(&self) -> Outcome<B> {
            match self.operation.finish() {
                Outcome::None => Outcome::None,
                Outcome::Some(value) => {
                    Outcome::Chain(Box::new((self.next)(value)))
                }
                Outcome::Chain(operation) => {
                    Outcome::Chain(Box::new(then(operation, self.next)))
                }
            }
        }
    }

    Chain {
        operation,
        next: f,
        _result: PhantomData,
    }
}

/// Produces an [`Operation`] that applies the given [`Operation`] to the
/// children of a container with the given [`Id`].
pub fn scope<T: 'static>(
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
            _bounds: Rectangle,
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
