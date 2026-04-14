//! Operate on widgets that can be focused.
use std::cell::Cell;

use crate::Rectangle;
use crate::widget::Id;
use crate::widget::operation::{self, Operation, Outcome};

thread_local! {
    /// Center of the widget that was focused before the last directional
    /// navigation. Used as a tiebreaker when multiple in-beam candidates
    /// have equal primary-axis distance — the candidate closest to the
    /// previous origin wins, producing natural "return to where I came
    /// from" behavior.
    static PREV_FOCUS_CENTER: Cell<Option<(f32, f32)>> = const { Cell::new(None) };
}

/// The internal state of a widget that can be focused.
pub trait Focusable {
    /// Returns whether the widget is focused or not.
    fn is_focused(&self) -> bool;

    /// Focuses the widget.
    fn focus(&mut self);

    /// Unfocuses the widget.
    fn unfocus(&mut self);
}

/// Spatial direction for directional focus navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusDirection {
    /// Move focus upward.
    Up,
    /// Move focus downward.
    Down,
    /// Move focus to the left.
    Left,
    /// Move focus to the right.
    Right,
}

/// A summary of the focusable widgets present on a widget tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Count {
    /// The index of the current focused widget, if any.
    pub focused: Option<usize>,

    /// The total amount of focusable widgets.
    pub total: usize,
}

/// Produces an [`Operation`] that focuses the widget with the given [`Id`].
pub fn focus<T>(target: Id) -> impl Operation<T> {
    struct Focus {
        target: Id,
    }

    impl<T> Operation<T> for Focus {
        fn focusable(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            match id {
                Some(id) if id == &self.target => {
                    state.focus();
                }
                _ => {
                    state.unfocus();
                }
            }
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }
    }

    Focus { target }
}

/// Produces an [`Operation`] that unfocuses the focused widget.
pub fn unfocus<T>() -> impl Operation<T> {
    struct Unfocus;

    impl<T> Operation<T> for Unfocus {
        fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            state.unfocus();
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }
    }

    Unfocus
}

/// Produces an [`Operation`] that generates a [`Count`] and chains it with the
/// provided function to build a new [`Operation`].
pub fn count() -> impl Operation<Count> {
    struct CountFocusable {
        count: Count,
    }

    impl Operation<Count> for CountFocusable {
        fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if state.is_focused() {
                self.count.focused = Some(self.count.total);
            }

            self.count.total += 1;
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Count>)) {
            operate(self);
        }

        fn finish(&self) -> Outcome<Count> {
            Outcome::Some(self.count)
        }
    }

    CountFocusable {
        count: Count::default(),
    }
}

/// Produces an [`Operation`] that searches for the current focused widget, and
/// - if found, focuses the previous focusable widget.
/// - if not found, focuses the last focusable widget.
pub fn focus_previous<T>() -> impl Operation<T>
where
    T: Send + 'static,
{
    struct FocusPrevious {
        count: Count,
        current: usize,
    }

    impl<T> Operation<T> for FocusPrevious {
        fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if self.count.total == 0 {
                return;
            }

            match self.count.focused {
                None if self.current == self.count.total - 1 => state.focus(),
                Some(0) if self.current == 0 => state.unfocus(),
                Some(0) => {}
                Some(focused) if focused == self.current => state.unfocus(),
                Some(focused) if focused - 1 == self.current => state.focus(),
                _ => {}
            }

            self.current += 1;
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }
    }

    operation::then(count(), |count| FocusPrevious { count, current: 0 })
}

/// Produces an [`Operation`] that searches for the current focused widget, and
/// - if found, focuses the next focusable widget.
/// - if not found, focuses the first focusable widget.
pub fn focus_next<T>() -> impl Operation<T>
where
    T: Send + 'static,
{
    struct FocusNext {
        count: Count,
        current: usize,
    }

    impl<T> Operation<T> for FocusNext {
        fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            match self.count.focused {
                None if self.current == 0 => state.focus(),
                Some(focused) if focused == self.current => state.unfocus(),
                Some(focused) if focused + 1 == self.current => state.focus(),
                _ => {}
            }

            self.current += 1;
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }
    }

    operation::then(count(), |count| FocusNext { count, current: 0 })
}

/// Produces an [`Operation`] that searches for the current focused widget
/// and stores its ID. This ignores widgets that do not have an ID.
pub fn find_focused() -> impl Operation<Id> {
    struct FindFocused {
        focused: Option<Id>,
    }

    impl Operation<Id> for FindFocused {
        fn focusable(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if state.is_focused() && id.is_some() {
                self.focused = id.cloned();
            }
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Id>)) {
            operate(self);
        }

        fn finish(&self) -> Outcome<Id> {
            if let Some(id) = &self.focused {
                Outcome::Some(id.clone())
            } else {
                Outcome::None
            }
        }
    }

    FindFocused { focused: None }
}

/// Produces an [`Operation`] that searches for the focusable widget
/// and stores whether it is focused or not. This ignores widgets that
/// do not have an ID.
pub fn is_focused(target: Id) -> impl Operation<bool> {
    struct IsFocused {
        target: Id,
        is_focused: Option<bool>,
    }

    impl Operation<bool> for IsFocused {
        fn focusable(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if id.is_some_and(|id| *id == self.target) {
                self.is_focused = Some(state.is_focused());
            }
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<bool>)) {
            if self.is_focused.is_some() {
                return;
            }

            operate(self);
        }

        fn finish(&self) -> Outcome<bool> {
            self.is_focused.map_or(Outcome::None, Outcome::Some)
        }
    }

    IsFocused {
        target,
        is_focused: None,
    }
}

/// A snapshot of all focusable widgets: their indices, bounds, and which is focused.
#[derive(Debug, Clone, Default)]
struct SpatialScan {
    /// `(index, bounds)` for every focusable widget in tree order.
    widgets: Vec<(usize, Rectangle)>,
    /// Index + bounds of the currently focused widget, if any.
    focused: Option<(usize, Rectangle)>,
    /// Running counter while scanning.
    total: usize,
    /// The direction to move focus.
    direction: Option<FocusDirection>,
}

/// Produces an [`Operation`] that collects all focusable widget bounds.
fn spatial_scan(direction: FocusDirection) -> impl Operation<SpatialScan> {
    struct Scan {
        result: SpatialScan,
    }

    impl Operation<SpatialScan> for Scan {
        fn focusable(&mut self, _id: Option<&Id>, bounds: Rectangle, state: &mut dyn Focusable) {
            let idx = self.result.total;
            self.result.widgets.push((idx, bounds));
            if state.is_focused() {
                self.result.focused = Some((idx, bounds));
            }
            self.result.total += 1;
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<SpatialScan>)) {
            operate(self);
        }

        fn finish(&self) -> Outcome<SpatialScan> {
            Outcome::Some(self.result.clone())
        }
    }

    Scan {
        result: SpatialScan {
            direction: Some(direction),
            ..SpatialScan::default()
        },
    }
}

/// Produces an [`Operation`] that moves focus to the nearest focusable widget
/// in the given [`FocusDirection`] based on spatial position.
///
/// If no widget is currently focused, focuses the first widget.
/// If no candidate exists in the requested direction, focus does not move.
pub fn focus_directional<T>(direction: FocusDirection) -> impl Operation<T>
where
    T: Send + 'static,
{
    struct FocusDirectional {
        scan: SpatialScan,
        target_index: Option<usize>,
        current: usize,
    }

    impl<T> Operation<T> for FocusDirectional {
        fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if let Some(target) = self.target_index {
                // Unfocus the previously focused widget.
                if self
                    .scan
                    .focused
                    .is_some_and(|(idx, _)| idx == self.current)
                {
                    state.unfocus();
                }
                // Focus the target widget.
                if self.current == target {
                    state.focus();
                }
            }
            self.current += 1;
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }
    }

    fn apply_directional(scan: SpatialScan) -> FocusDirectional {
        let direction = scan.direction.unwrap_or(FocusDirection::Down);
        let target_index = find_directional_target(&scan, direction);

        // Save the current focused widget's center so the *next* directional
        // navigation can use it as a tiebreaker ("return to origin").
        if target_index.is_some()
            && let Some((_idx, bounds)) = scan.focused
        {
            let c = bounds.center();
            PREV_FOCUS_CENTER.set(Some((c.x, c.y)));
        }

        FocusDirectional {
            scan,
            target_index,
            current: 0,
        }
    }

    operation::then(spatial_scan(direction), apply_directional)
}

/// Finds the best focusable widget in the given direction from the currently
/// focused widget using a beam-based spatial proximity algorithm with
/// navigation history.
///
/// Ranking:
/// 1. Candidates whose cross-axis extent overlaps with the source widget's
///    projection ("in beam") are always preferred over those outside the beam.
/// 2. Among in-beam candidates, the closest by primary-axis edge distance wins.
/// 3. In-beam ties are broken by navigation history: the candidate whose center
///    is closest (cross-axis) to the *previously* focused widget wins, producing
///    natural "return to where I came from" behavior. Without history, more
///    beam overlap wins.
/// 4. Among out-of-beam candidates, a weighted edge distance is used
///    (primary × 1 + cross × 3) to favour roughly-aligned widgets.
///
/// Returns `Some(index)` of the best candidate, or the first widget if
/// nothing is focused, or `None` if there are no candidates.
fn find_directional_target(scan: &SpatialScan, direction: FocusDirection) -> Option<usize> {
    if scan.widgets.is_empty() {
        return None;
    }

    let Some((focused_idx, focused_bounds)) = scan.focused else {
        // Nothing focused → focus the first widget.
        return Some(scan.widgets[0].0);
    };

    let origin = focused_bounds.center();
    let prev_center = PREV_FOCUS_CENTER.get();

    // Best candidate: (index, in_beam, primary_edge_dist, beam_overlap,
    //                  cross_edge_dist, history_dist).
    let mut best: Option<(usize, bool, f32, f32, f32, f32)> = None;

    for &(idx, bounds) in &scan.widgets {
        if idx == focused_idx {
            continue;
        }

        let candidate = bounds.center();

        // Check the candidate is in the correct direction.
        let in_direction = match direction {
            FocusDirection::Up => candidate.y < origin.y,
            FocusDirection::Down => candidate.y > origin.y,
            FocusDirection::Left => candidate.x < origin.x,
            FocusDirection::Right => candidate.x > origin.x,
        };

        if !in_direction {
            continue;
        }

        // Compute edge-to-edge distances and beam overlap.
        let (primary_dist, beam_overlap, cross_dist) = match direction {
            FocusDirection::Up | FocusDirection::Down => {
                let primary = edge_dist(
                    focused_bounds.y,
                    focused_bounds.y + focused_bounds.height,
                    bounds.y,
                    bounds.y + bounds.height,
                );
                let beam = axis_overlap(
                    focused_bounds.x,
                    focused_bounds.x + focused_bounds.width,
                    bounds.x,
                    bounds.x + bounds.width,
                );
                let cross = edge_dist(
                    focused_bounds.x,
                    focused_bounds.x + focused_bounds.width,
                    bounds.x,
                    bounds.x + bounds.width,
                );
                (primary, beam, cross)
            }
            FocusDirection::Left | FocusDirection::Right => {
                let primary = edge_dist(
                    focused_bounds.x,
                    focused_bounds.x + focused_bounds.width,
                    bounds.x,
                    bounds.x + bounds.width,
                );
                let beam = axis_overlap(
                    focused_bounds.y,
                    focused_bounds.y + focused_bounds.height,
                    bounds.y,
                    bounds.y + bounds.height,
                );
                let cross = edge_dist(
                    focused_bounds.y,
                    focused_bounds.y + focused_bounds.height,
                    bounds.y,
                    bounds.y + bounds.height,
                );
                (primary, beam, cross)
            }
        };

        let in_beam = beam_overlap > 0.0;

        // Cross-axis distance from the candidate's center to the previously
        // focused widget's center. Used as a tiebreaker so that reversing
        // direction returns to the original widget.
        let history_dist = prev_center.map_or(f32::MAX, |(px, py)| match direction {
            FocusDirection::Up | FocusDirection::Down => (candidate.x - px).abs(),
            FocusDirection::Left | FocusDirection::Right => (candidate.y - py).abs(),
        });

        let is_better = match best {
            None => true,
            Some((_, best_in_beam, best_primary, best_beam, best_cross, best_hist)) => {
                if in_beam != best_in_beam {
                    // In-beam always wins over out-of-beam.
                    in_beam
                } else if in_beam {
                    // Both in beam: prefer closer primary edge distance.
                    if (primary_dist - best_primary).abs() > 0.5 {
                        primary_dist < best_primary
                    } else if prev_center.is_some() {
                        // Same primary distance with history: prefer
                        // candidate closer to where we came from.
                        history_dist < best_hist
                    } else {
                        // No history: prefer more beam overlap.
                        beam_overlap > best_beam
                    }
                } else {
                    // Both out of beam: weighted edge distance.
                    (primary_dist + cross_dist * 3.0) < (best_primary + best_cross * 3.0)
                }
            }
        };

        if is_better {
            best = Some((
                idx,
                in_beam,
                primary_dist,
                beam_overlap,
                cross_dist,
                history_dist,
            ));
        }
    }

    best.map(|(idx, _, _, _, _, _)| idx)
}

/// Edge-to-edge distance between two 1D intervals. Returns 0 if they overlap.
fn edge_dist(a_min: f32, a_max: f32, b_min: f32, b_max: f32) -> f32 {
    if a_max <= b_min {
        b_min - a_max
    } else if b_max <= a_min {
        a_min - b_max
    } else {
        0.0
    }
}

/// Overlap length between two 1D intervals. Returns 0 if they don't overlap.
fn axis_overlap(a_min: f32, a_max: f32, b_min: f32, b_max: f32) -> f32 {
    (a_max.min(b_max) - a_min.max(b_min)).max(0.0)
}
