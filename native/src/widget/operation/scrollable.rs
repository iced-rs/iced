//! Operate on widgets that can be scrolled.
use crate::widget::{Id, Operation};
use iced_core::Point;

/// The internal state of a widget that can be scrolled.
pub trait Scrollable {
    /// Snaps the scroll of the widget to the given `percentage` along the horizontal & vertical axis.
    fn snap_to(&mut self, percentage: Point);
}

/// Produces an [`Operation`] that snaps the widget with the given [`Id`] to
/// the provided `percentage`.
pub fn snap_to<T>(target: Id, percentage: Point) -> impl Operation<T> {
    struct SnapTo {
        target: Id,
        percentage: Point,
    }

    impl<T> Operation<T> for SnapTo {
        fn container(
            &mut self,
            _id: Option<&Id>,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<T>),
        ) {
            operate_on_children(self)
        }

        fn scrollable(&mut self, state: &mut dyn Scrollable, id: Option<&Id>) {
            if Some(&self.target) == id {
                state.snap_to(self.percentage);
            }
        }
    }

    SnapTo { target, percentage }
}
