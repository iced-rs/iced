//! Find and query widgets in your applications.
pub use iced_selector::{Bounded, Candidate, Selector, Target, Text, id, is_focused};

use crate::Task;
use crate::task;

/// Finds a widget matching the given [`Selector`].
pub fn find<S, Custom>(selector: S) -> Task<Option<S::Output>, Custom>
where
    S: Selector + Send + 'static,
    S::Output: Send + Clone + 'static,
    Custom: Send + 'static,
{
    task::widget(selector.find())
}

/// Finds all widgets matching the given [`Selector`].
pub fn find_all<S, Custom>(selector: S) -> Task<Vec<S::Output>, Custom>
where
    S: Selector + Send + 'static,
    S::Output: Send + Clone + 'static,
    Custom: Send + 'static,
{
    task::widget(selector.find_all())
}
