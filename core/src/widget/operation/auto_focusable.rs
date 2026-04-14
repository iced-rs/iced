//! Operate on widgets marked for automatic focus.
//!
//! When a page or view first appears, the framework can call
//! [`focus_auto`] to focus the first widget wrapped in an
//! [`auto_focus`](crate::widget::auto_focus) wrapper. If no auto-focus
//! widget exists, the first focusable widget receives focus instead.
use crate::widget::operation::{self, Focusable, Operation, Outcome};
use crate::widget::Id;
use crate::Rectangle;

/// Result of the first tree-walk phase.
#[derive(Debug, Clone, Copy, Default)]
struct ScanResult {
    /// Index (in focusable order) of the first auto-focus widget, if any.
    auto_focus_index: Option<usize>,
    /// Total number of focusable widgets encountered.
    total: usize,
    /// Index of the currently focused widget, if any.
    focused_index: Option<usize>,
}

/// Produces an [`Operation`] that focuses the first auto-focus widget.
///
/// If the auto-focus target is already focused, the operation is a no-op.
/// If no widget is marked for auto-focus, the first focusable widget
/// receives focus instead (matching [`focus_next`](super::focusable::focus_next)
/// fallback behaviour).
pub fn focus_auto<T>() -> impl Operation<T>
where
    T: Send + 'static,
{
    operation::then(scan(), apply)
}

// -- Phase 1: scan the tree ------------------------------------------------

fn scan() -> impl Operation<ScanResult> {
    struct Scan {
        result: ScanResult,
        /// When `true`, the *next* `focusable()` call records the target.
        mark_next: bool,
    }

    impl Operation<ScanResult> for Scan {
        fn auto_focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle) {
            self.mark_next = true;
        }

        fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if state.is_focused() {
                self.result.focused_index = Some(self.result.total);
            }

            if self.mark_next && self.result.auto_focus_index.is_none() {
                self.result.auto_focus_index = Some(self.result.total);
                self.mark_next = false;
            }
            self.result.total += 1;
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<ScanResult>)) {
            operate(self);
        }

        fn finish(&self) -> Outcome<ScanResult> {
            // If ANY widget is already focused, don't steal focus.
            // This matches Flutter's behaviour: autofocus only fires
            // when nothing in the scope has primary focus.
            if let Some(idx) = self.result.focused_index {
                log::debug!(
                    "[focus_auto] widget at index {} is already focused — not stealing focus",
                    idx,
                );
                return Outcome::None;
            }

            log::debug!(
                "[focus_auto] scan done — target={:?}, focused={:?}, total={}",
                self.result.auto_focus_index,
                self.result.focused_index,
                self.result.total
            );
            Outcome::Some(self.result)
        }
    }

    Scan {
        result: ScanResult::default(),
        mark_next: false,
    }
}

// -- Phase 2: apply focus ---------------------------------------------------

fn apply<T>(result: ScanResult) -> impl Operation<T> {
    struct Apply {
        /// Index of the widget to focus.
        target: usize,
        /// Whether there is any focusable widget at all.
        has_target: bool,
        current: usize,
    }

    impl<T> Operation<T> for Apply {
        fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if self.has_target && self.current == self.target {
                log::debug!("[focus_auto] focusing widget at index {}", self.current);
                state.focus();
            } else {
                state.unfocus();
            }
            self.current += 1;
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }
    }

    Apply {
        target: result.auto_focus_index.unwrap_or(0),
        has_target: result.total > 0,
        current: 0,
    }
}
