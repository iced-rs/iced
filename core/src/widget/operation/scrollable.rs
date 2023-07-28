//! Operate on widgets that can be scrolled.
use crate::widget::{Id, Operation};
use crate::{Rectangle, Vector};

/// The internal state of a widget that can be scrolled.
pub trait Scrollable {
    /// Snaps the scroll of the widget to the given `percentage` along the horizontal & vertical axis.
    fn snap_to(&mut self, offset: RelativeOffset);

    /// Scroll the widget to the given [`AbsoluteOffset`] along the horizontal & vertical axis.
    fn scroll_to(&mut self, offset: AbsoluteOffset);
}

/// Produces an [`Operation`] that snaps the widget with the given [`Id`] to
/// the provided `percentage`.
pub fn snap_to<T>(target: Id, offset: RelativeOffset) -> impl Operation<T> {
    struct SnapTo {
        target: Id,
        offset: RelativeOffset,
    }

    impl<T> Operation<T> for SnapTo {
        fn container(
            &mut self,
            _id: Option<&Id>,
            _bounds: Rectangle,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<T>),
        ) {
            operate_on_children(self)
        }

        fn scrollable(
            &mut self,
            state: &mut dyn Scrollable,
            id: Option<&Id>,
            _bounds: Rectangle,
            _translation: Vector,
        ) {
            if Some(&self.target) == id {
                state.snap_to(self.offset);
            }
        }
    }

    SnapTo { target, offset }
}

/// Produces an [`Operation`] that scrolls the widget with the given [`Id`] to
/// the provided [`AbsoluteOffset`].
pub fn scroll_to<T>(target: Id, offset: AbsoluteOffset) -> impl Operation<T> {
    struct ScrollTo {
        target: Id,
        offset: AbsoluteOffset,
    }

    impl<T> Operation<T> for ScrollTo {
        fn container(
            &mut self,
            _id: Option<&Id>,
            _bounds: Rectangle,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<T>),
        ) {
            operate_on_children(self)
        }

        fn scrollable(
            &mut self,
            state: &mut dyn Scrollable,
            id: Option<&Id>,
            _bounds: Rectangle,
            _translation: Vector,
        ) {
            if Some(&self.target) == id {
                state.scroll_to(self.offset);
            }
        }
    }

    ScrollTo { target, offset }
}

/// The amount of absolute offset in each direction of a [`Scrollable`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AbsoluteOffset {
    /// The amount of horizontal offset
    pub x: f32,
    /// The amount of vertical offset
    pub y: f32,
}

/// The amount of relative offset in each direction of a [`Scrollable`].
///
/// A value of `0.0` means start, while `1.0` means end.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RelativeOffset {
    /// The amount of horizontal offset
    pub x: f32,
    /// The amount of vertical offset
    pub y: f32,
}

impl RelativeOffset {
    /// A relative offset that points to the top-left of a [`Scrollable`].
    pub const START: Self = Self { x: 0.0, y: 0.0 };

    /// A relative offset that points to the bottom-right of a [`Scrollable`].
    pub const END: Self = Self { x: 1.0, y: 1.0 };
}
