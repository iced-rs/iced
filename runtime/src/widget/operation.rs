//! Change internal widget state.
use crate::core::widget::Id;
use crate::core::widget::operation;
use crate::task;
use crate::{Action, Task};

pub use crate::core::widget::operation::Animation;

/// Focuses the previous focusable widget.
pub fn focus_previous<T>() -> Task<T> {
    task::effect(Action::widget(operation::focusable::focus_previous()))
}

/// Focuses the next focusable widget.
pub fn focus_next<T>() -> Task<T> {
    task::effect(Action::widget(operation::focusable::focus_next()))
}

/// Returns whether the widget with the given [`Id`] is focused or not.
pub fn is_focused(id: impl Into<Id>) -> Task<bool> {
    task::widget(operation::focusable::is_focused(id.into()))
}

/// Focuses the widget with the given [`Id`].
pub fn focus<T>(id: impl Into<Id>) -> Task<T> {
    task::effect(Action::widget(operation::focusable::focus(id.into())))
}

/// Operations for widgets that can scroll.
pub mod scrollable {
    use super::*;

    pub use crate::core::widget::operation::scrollable::{AbsoluteOffset, RelativeOffset};

    /// Snaps the scrollable with the given [`Id`] to the provided [`RelativeOffset`].
    pub fn snap_to<T>(
        id: impl Into<Id>,
        offset: impl Into<RelativeOffset<Option<f32>>>,
        animation: Animation,
    ) -> Task<T> {
        task::effect(Action::widget(operation::scrollable::snap_to(
            id.into(),
            offset.into(),
            animation,
        )))
    }

    /// Snaps the scrollable with the given [`Id`] to the [`RelativeOffset::END`].
    pub fn snap_to_end<T>(id: impl Into<Id>, animation: Animation) -> Task<T> {
        task::effect(Action::widget(operation::scrollable::snap_to(
            id.into(),
            RelativeOffset::END.into(),
            animation,
        )))
    }

    /// Scrolls the scrollable with the given [`Id`] to the provided [`AbsoluteOffset`].
    pub fn scroll_to<T>(
        id: impl Into<Id>,
        offset: impl Into<AbsoluteOffset<Option<f32>>>,
        animation: Animation,
    ) -> Task<T> {
        task::effect(Action::widget(operation::scrollable::scroll_to(
            id.into(),
            offset.into(),
            animation,
        )))
    }

    /// Scrolls the scrollable with the given [`Id`] by the provided [`AbsoluteOffset`].
    pub fn scroll_by<T>(
        id: impl Into<Id>,
        offset: AbsoluteOffset,
        animation: Animation,
    ) -> Task<T> {
        task::effect(Action::widget(operation::scrollable::scroll_by(
            id.into(),
            offset,
            animation,
        )))
    }
}

/// Operations for widgets that contain text.
pub mod text {
    use super::*;
    use crate::core::Point;
    use crate::core::text::Target;

    /// Selects text between the given positions in layout coordinates.
    ///
    /// `start` is the position where the user started selecting from,
    /// while `end` is the final position of the selection.
    pub fn select<T>(start: Point, end: Point, target: Target) -> Task<T> {
        task::effect(Action::widget(operation::text::select(start, end, target)))
    }

    /// Selects all text in the widget with the given [`Id`].
    pub fn select_all<T>(id: impl Into<Id>) -> Task<T> {
        task::effect(Action::widget(operation::scope(
            id.into(),
            operation::text::select_all(),
        )))
    }

    /// Deselects any selected text.
    pub fn deselect<T>() -> Task<T> {
        task::effect(Action::widget(operation::text::deselect()))
    }
}

/// Operations for widgets that can be used for text input.
pub mod text_input {
    use super::*;
    use crate::core::text::Position;

    /// Selects all the content of the widget with the given [`Id`].
    pub fn select_all<T>(id: impl Into<Id>) -> Task<T> {
        task::effect(Action::widget(operation::text_input::select_all(id.into())))
    }

    /// Selects the given content range of the widget with the given [`Id`].
    pub fn select_range<T>(id: impl Into<Id>, start: Position, end: Position) -> Task<T> {
        task::effect(Action::widget(operation::text_input::select_range(
            id.into(),
            start,
            end,
        )))
    }

    /// Moves the cursor of the widget with the given [`Id`] to the end.
    pub fn move_cursor_to_end<T>(id: impl Into<Id>) -> Task<T> {
        task::effect(Action::widget(operation::text_input::move_cursor_to_end(
            id.into(),
        )))
    }

    /// Moves the cursor of the widget with the given [`Id`] to the front.
    pub fn move_cursor_to_front<T>(id: impl Into<Id>) -> Task<T> {
        task::effect(Action::widget(operation::text_input::move_cursor_to_front(
            id.into(),
        )))
    }

    /// Moves the cursor of the widget with the given [`Id`] to the provided position.
    pub fn move_cursor_to<T>(id: impl Into<Id>, position: Position) -> Task<T> {
        task::effect(Action::widget(operation::text_input::move_cursor_to(
            id.into(),
            position,
        )))
    }
}
