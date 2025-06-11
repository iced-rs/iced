//! Select widgets of a user interface.
use crate::core::text;
use crate::core::widget;

/// A selector describes a strategy to find a certain widget in a user interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector {
    /// Find the widget with the given [`widget::Id`].
    Id(widget::Id),
    /// Find the widget containing the given [`text::Fragment`].
    Text(text::Fragment<'static>),
}

impl From<widget::Id> for Selector {
    fn from(id: widget::Id) -> Self {
        Self::Id(id)
    }
}

impl From<&'static str> for Selector {
    fn from(text: &'static str) -> Self {
        Self::Text(text.into())
    }
}

/// Creates a [`Selector`] that finds the widget with the given [`widget::Id`].
pub fn id(id: impl Into<widget::Id>) -> Selector {
    Selector::Id(id.into())
}

/// Creates a [`Selector`] that finds the widget containing the given text fragment.
pub fn text(fragment: impl text::IntoFragment<'static>) -> Selector {
    Selector::Text(fragment.into_fragment())
}
