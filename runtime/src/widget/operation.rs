//! Change internal widget state.
use crate::core::widget::Id;
use crate::core::widget::operation;
use crate::task;
use crate::{Action, Task};

pub use crate::core::widget::operation::scrollable::{AbsoluteOffset, RelativeOffset};

/// Snaps the scrollable with the given [`Id`] to the provided [`RelativeOffset`].
pub fn snap_to<T, Custom>(
    id: impl Into<Id>,
    offset: impl Into<RelativeOffset<Option<f32>>>,
) -> Task<T, Custom>
where
    Custom: Send + 'static,
{
    task::effect(Action::widget(operation::scrollable::snap_to(
        id.into(),
        offset.into(),
    )))
}

/// Snaps the scrollable with the given [`Id`] to the [`RelativeOffset::END`].
pub fn snap_to_end<T, Custom>(id: impl Into<Id>) -> Task<T, Custom>
where
    Custom: Send + 'static,
{
    task::effect(Action::widget(operation::scrollable::snap_to(
        id.into(),
        RelativeOffset::END.into(),
    )))
}

/// Scrolls the scrollable with the given [`Id`] to the provided [`AbsoluteOffset`].
pub fn scroll_to<T, Custom>(
    id: impl Into<Id>,
    offset: impl Into<AbsoluteOffset<Option<f32>>>,
) -> Task<T, Custom>
where
    Custom: Send + 'static,
{
    task::effect(Action::widget(operation::scrollable::scroll_to(
        id.into(),
        offset.into(),
    )))
}

/// Scrolls the scrollable with the given [`Id`] by the provided [`AbsoluteOffset`].
pub fn scroll_by<T, Custom>(id: impl Into<Id>, offset: AbsoluteOffset) -> Task<T, Custom>
where
    Custom: Send + 'static,
{
    task::effect(Action::widget(operation::scrollable::scroll_by(
        id.into(),
        offset,
    )))
}

/// Focuses the previous focusable widget.
pub fn focus_previous<T, Custom>() -> Task<T, Custom>
where
    Custom: Send + 'static,
{
    task::effect(Action::widget(operation::focusable::focus_previous()))
}

/// Focuses the next focusable widget.
pub fn focus_next<T, Custom>() -> Task<T, Custom>
where
    Custom: Send + 'static,
{
    task::effect(Action::widget(operation::focusable::focus_next()))
}

/// Returns whether the widget with the given [`Id`] is focused or not.
pub fn is_focused<Custom>(id: impl Into<Id>) -> Task<bool, Custom>
where
    Custom: Send + 'static,
{
    task::widget(operation::focusable::is_focused(id.into()))
}

/// Focuses the widget with the given [`Id`].
pub fn focus<T, Custom>(id: impl Into<Id>) -> Task<T, Custom>
where
    Custom: Send + 'static,
{
    task::effect(Action::widget(operation::focusable::focus(id.into())))
}

/// Moves the cursor of the widget with the given [`Id`] to the end.
pub fn move_cursor_to_end<T, Custom>(id: impl Into<Id>) -> Task<T, Custom>
where
    Custom: Send + 'static,
{
    task::effect(Action::widget(operation::text_input::move_cursor_to_end(
        id.into(),
    )))
}

/// Moves the cursor of the widget with the given [`Id`] to the front.
pub fn move_cursor_to_front<T, Custom>(id: impl Into<Id>) -> Task<T, Custom>
where
    Custom: Send + 'static,
{
    task::effect(Action::widget(operation::text_input::move_cursor_to_front(
        id.into(),
    )))
}

/// Moves the cursor of the widget with the given [`Id`] to the provided position.
pub fn move_cursor_to<T, Custom>(id: impl Into<Id>, position: usize) -> Task<T, Custom>
where
    Custom: Send + 'static,
{
    task::effect(Action::widget(operation::text_input::move_cursor_to(
        id.into(),
        position,
    )))
}

/// Selects all the content of the widget with the given [`Id`].
pub fn select_all<T, Custom>(id: impl Into<Id>) -> Task<T, Custom>
where
    Custom: Send + 'static,
{
    task::effect(Action::widget(operation::text_input::select_all(id.into())))
}

/// Selects the given content range of the widget with the given [`Id`].
pub fn select_range<T, Custom>(id: impl Into<Id>, start: usize, end: usize) -> Task<T, Custom>
where
    Custom: Send + 'static,
{
    task::effect(Action::widget(operation::text_input::select_range(
        id.into(),
        start,
        end,
    )))
}
