//! Find and query widgets in your applications.
pub use iced_selector::{Bounded, Candidate, Selector, Target, Text};

use crate::core::Rectangle;

use crate::Task;
use crate::core::widget;
use crate::task;

/// Finds a widget by the given [`widget::Id`].
pub fn find_by_id(id: impl Into<widget::Id>) -> Task<Option<Target>> {
    task::widget(id.into().find())
}

/// Finds a widget that contains the given text.
pub fn find_by_text(text: impl Into<String>) -> Task<Option<Text>> {
    task::widget(Selector::find(text.into()))
}

/// Finds the visible bounds of the first [`Selector`] target.
pub fn delineate<S>(selector: S) -> Task<Option<Rectangle>>
where
    S: Selector + Send + 'static,
    S::Output: Bounded + Clone + Send + 'static,
{
    task::widget(selector.find())
        .map(|target| target.as_ref().and_then(Bounded::visible_bounds))
}
