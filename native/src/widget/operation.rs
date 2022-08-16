//! Query or update internal widget state.
pub mod focusable;
pub mod scrollable;

pub use focusable::Focusable;
pub use scrollable::Scrollable;

use crate::widget::Id;

use std::fmt;

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

    /// Finishes the [`Operation`] and returns its [`Outcome`].
    fn finish(&self) -> Outcome<T> {
        Outcome::None
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
