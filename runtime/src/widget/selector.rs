//! Find and query widgets in your applications.
pub use iced_selector::Selector;

pub use iced_selector::target::{Bounded, Match, Target, Text};

use crate::Task;
use crate::core::widget;
use crate::task;

/// Finds a widget by the given [`widget::Id`].
pub fn find_by_id(id: impl Into<widget::Id>) -> Task<Option<Match>> {
    task::widget(id.into().find())
}

/// Finds a widget that contains the given text.
pub fn find_by_text(text: impl Into<String>) -> Task<Option<Text>> {
    task::widget(Selector::find(text.into()))
}
