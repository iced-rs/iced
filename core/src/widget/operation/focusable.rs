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

    /// Index and bounds of the last widget that was focused during
    /// directional navigation. When nothing is focused and a direction
    /// key is pressed, we try to restore focus to this widget (if it
    /// still exists at the same tree position with matching bounds)
    /// and navigate from there, rather than always jumping to the
    /// first widget.
    static LAST_FOCUSED: Cell<Option<(usize, Rectangle)>> = const { Cell::new(None) };

    /// Set to `true` whenever a focus operation changes which widget is
    /// focused. The runtime checks and clears this flag after processing
    /// widget operations, injecting a `FocusChanged` event when set.
    static FOCUS_DIRTY: Cell<bool> = const { Cell::new(false) };
}

/// Marks that a focus change occurred during an operation.
///
/// Called internally by focus operations when they call
/// [`Focusable::focus()`] or [`Focusable::unfocus()`].
pub fn mark_focus_dirty() {
    FOCUS_DIRTY.set(true);
}

/// Returns `true` if a focus change occurred since the last call,
/// clearing the flag.
///
/// Called by the runtime after processing widget operations.
pub fn take_focus_dirty() -> bool {
    FOCUS_DIRTY.replace(false)
}

/// The internal state of a widget that can be focused.
pub trait Focusable {
    /// Returns whether the widget is focused or not.
    fn is_focused(&self) -> bool;

    /// Focuses the widget.
    fn focus(&mut self);

    /// Unfocuses the widget.
    fn unfocus(&mut self);

    /// Requests activation of the focused widget (e.g. gamepad A button).
    ///
    /// The widget should set an internal flag that is consumed during the
    /// next event-processing cycle to fire its `on_press` handler.
    /// Returns `true` if the widget accepted the press.
    ///
    /// The default implementation does nothing and returns `false`.
    fn press(&mut self) -> bool {
        false
    }

    /// Gives the focused widget a chance to consume a directional input
    /// before focus moves to another widget.
    ///
    /// For example, a focusable scrollable returns `true` (and scrolls
    /// internally) when the direction still has room to scroll, preventing
    /// focus from leaving. At a scroll boundary it returns `false` so
    /// focus proceeds to the next widget.
    ///
    /// The default implementation returns `false` (direction not consumed).
    fn consume_direction(&mut self, _direction: FocusDirection) -> bool {
        false
    }
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
                    mark_focus_dirty();
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
            if state.is_focused() {
                state.unfocus();
                mark_focus_dirty();
            }
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
                None if self.current == self.count.total - 1 => {
                    state.focus();
                    mark_focus_dirty();
                }
                Some(0) if self.current == 0 => state.unfocus(),
                Some(0) => {}
                Some(focused) if focused == self.current => state.unfocus(),
                Some(focused) if focused - 1 == self.current => {
                    state.focus();
                    mark_focus_dirty();
                }
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
                None if self.current == 0 => {
                    state.focus();
                    mark_focus_dirty();
                }
                Some(focused) if focused == self.current => state.unfocus(),
                Some(focused) if focused + 1 == self.current => {
                    state.focus();
                    mark_focus_dirty();
                }
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

/// Produces an [`Operation`] that activates the currently focused widget
/// by calling [`Focusable::press()`]. This is the preferred way to trigger
/// a focused widget from gamepad input — it targets only the focused widget
/// without injecting synthetic keyboard events.
pub fn press_focused<T>() -> impl Operation<T>
where
    T: Send + 'static,
{
    struct PressFocused;

    impl<T> Operation<T> for PressFocused {
        fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if state.is_focused() {
                let _ = state.press();
            }
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }
    }

    PressFocused
}

/// A snapshot of all focusable widgets: their indices, bounds, and which is focused.
#[derive(Debug, Clone, Default)]
struct SpatialScan {
    /// `(index, bounds, id)` for every focusable widget in tree order.
    widgets: Vec<(usize, Rectangle, Option<Id>)>,
    /// Index + bounds of the currently focused widget, if any.
    focused: Option<(usize, Rectangle)>,
    /// Running counter while scanning.
    total: usize,
    /// The direction to move focus.
    direction: Option<FocusDirection>,
    /// Whether to wrap to the opposite edge when no candidate exists.
    wrap: bool,
    /// Whether the focused widget consumed the direction (e.g. scrolled).
    direction_consumed: bool,
}

/// Produces an [`Operation`] that collects all focusable widget bounds.
fn spatial_scan(direction: FocusDirection, wrap: bool) -> impl Operation<SpatialScan> {
    struct Scan {
        result: SpatialScan,
    }

    impl Operation<SpatialScan> for Scan {
        fn focusable(&mut self, id: Option<&Id>, bounds: Rectangle, state: &mut dyn Focusable) {
            let idx = self.result.total;
            self.result.widgets.push((idx, bounds, id.cloned()));
            if state.is_focused() {
                self.result.focused = Some((idx, bounds));
                // Let the focused widget consume the direction (e.g. scroll).
                if let Some(direction) = self.result.direction
                    && state.consume_direction(direction)
                {
                    self.result.direction_consumed = true;
                }
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
            wrap,
            ..SpatialScan::default()
        },
    }
}

/// Produces an [`Operation`] that moves focus to the nearest focusable widget
/// in the given [`FocusDirection`] based on spatial position.
///
/// If no widget is currently focused, focuses the first widget.
/// If no candidate exists in the requested direction, wraps to the opposite
/// edge (e.g. Down at the bottom → top-most widget).
pub fn focus_directional<T>(direction: FocusDirection) -> impl Operation<T>
where
    T: Send + 'static,
{
    focus_directional_impl(direction, true)
}

/// Like [`focus_directional`], but does **not** wrap to the opposite edge
/// when no candidate exists in the requested direction. Focus simply stays
/// on the currently focused widget.
pub fn focus_directional_no_wrap<T>(direction: FocusDirection) -> impl Operation<T>
where
    T: Send + 'static,
{
    focus_directional_impl(direction, false)
}

fn focus_directional_impl<T>(direction: FocusDirection, wrap: bool) -> impl Operation<T>
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
                    mark_focus_dirty();
                }
            }
            self.current += 1;
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }
    }

    fn apply_directional(scan: SpatialScan) -> FocusDirectional {
        // If the focused widget consumed the direction (e.g. scrolled),
        // don't move focus at all.
        if scan.direction_consumed {
            log::trace!("[FocusDir] direction consumed by focused widget — not moving focus");
            return FocusDirectional {
                scan,
                target_index: None,
                current: 0,
            };
        }

        let direction = scan.direction.unwrap_or(FocusDirection::Down);
        let wrap = scan.wrap;

        log::trace!(
            "[FocusDir] direction={:?}, {} widgets, focused={:?}",
            direction,
            scan.widgets.len(),
            scan.focused
                .map(|(idx, b)| (idx, b.x, b.y, b.width, b.height)),
        );

        if log::log_enabled!(log::Level::Trace) {
            for &(idx, b, ref id) in &scan.widgets {
                log::trace!(
                    "[FocusDir]   widget[{}] id={:?} x={:.0} y={:.0} w={:.0} h={:.0}",
                    idx,
                    id,
                    b.x,
                    b.y,
                    b.width,
                    b.height,
                );
            }
        }

        let target_index = find_directional_target(&scan, direction, wrap);

        log::trace!(
            "[FocusDir] target_index={:?} (from {:?} → {:?})",
            target_index,
            scan.focused.map(|(i, _)| i),
            target_index,
        );

        // Save the current focused widget's center so the *next* directional
        // navigation can use it as a tiebreaker ("return to origin").
        if let Some((_idx, bounds)) = scan.focused
            && target_index.is_some()
        {
            let c = bounds.center();
            PREV_FOCUS_CENTER.set(Some((c.x, c.y)));
        }

        // Remember whichever widget ends up focused (target if moving,
        // current if staying put) so we can restore it later when
        // nothing is focused.
        let last = target_index
            .and_then(|ti| {
                scan.widgets
                    .iter()
                    .find(|(i, _, _)| *i == ti)
                    .map(|&(i, b, _)| (i, b))
            })
            .or(scan.focused);
        if let Some(entry) = last {
            LAST_FOCUSED.set(Some(entry));
        }

        FocusDirectional {
            scan,
            target_index,
            current: 0,
        }
    }

    operation::then(spatial_scan(direction, wrap), apply_directional)
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
/// Returns `Some(index)` of the best candidate, or a direction-appropriate
/// edge widget if nothing is focused, or `None` if there are no candidates.
fn find_directional_target(
    scan: &SpatialScan,
    direction: FocusDirection,
    wrap: bool,
) -> Option<usize> {
    if scan.widgets.is_empty() {
        return None;
    }

    let Some((focused_idx, focused_bounds)) = scan.focused else {
        // Nothing focused — try to restore the last focused widget
        // if it still exists at the same tree position with matching bounds.
        if let Some((last_idx, last_bounds)) = LAST_FOCUSED.get()
            && scan
                .widgets
                .iter()
                .any(|&(idx, b, _)| idx == last_idx && bounds_match(b, last_bounds))
        {
            return Some(last_idx);
        }

        // No restorable widget — pick the edge widget that matches
        // the direction the user is pressing FROM (opposite edge).
        return Some(edge_widget(scan, direction));
    };

    let origin = focused_bounds.center();
    let prev_center = PREV_FOCUS_CENTER.get();

    // Best candidate: (index, in_beam, primary_edge_dist, beam_overlap,
    //                  cross_edge_dist, history_dist).
    let mut best: Option<(usize, bool, f32, f32, f32, f32)> = None;

    for &(idx, bounds, _) in &scan.widgets {
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

    // If a candidate was found, use it. Otherwise, optionally wrap to the
    // opposite edge.
    best.map_or_else(
        || {
            if wrap {
                Some(wrap_widget(scan, focused_idx, direction))
            } else {
                None
            }
        },
        |(idx, _, _, _, _, _)| Some(idx),
    )
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

/// Returns `true` if `a` and `b` have the same position and size
/// (within a small tolerance to handle floating-point drift).
fn bounds_match(a: Rectangle, b: Rectangle) -> bool {
    const E: f32 = 1.0;
    (a.x - b.x).abs() < E
        && (a.y - b.y).abs() < E
        && (a.width - b.width).abs() < E
        && (a.height - b.height).abs() < E
}

/// Picks the widget at the edge the user is pressing *from*.
/// Pressing Up → user is below → start from the bottom-most widget.
/// Pressing Down → user is above → start from the top-most widget.
fn edge_widget(scan: &SpatialScan, direction: FocusDirection) -> usize {
    let &(idx, _, _) = scan
        .widgets
        .iter()
        .max_by(|a, b| {
            let val = |w: &(usize, Rectangle, Option<Id>)| -> f32 {
                match direction {
                    // Pressing Up → want bottom-most (max y)
                    FocusDirection::Up => w.1.y + w.1.height,
                    // Pressing Down → want top-most (min y → negate for max_by)
                    FocusDirection::Down => -(w.1.y),
                    // Pressing Left → want right-most (max x)
                    FocusDirection::Left => w.1.x + w.1.width,
                    // Pressing Right → want left-most (min x → negate)
                    FocusDirection::Right => -(w.1.x),
                }
            };
            val(a)
                .partial_cmp(&val(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or(&scan.widgets[0]);
    idx
}

/// Wraps focus to the opposite edge when no candidate exists in the
/// requested direction. Pressing Up at the top → bottom-most widget,
/// pressing Down at the bottom → top-most widget, etc.
///
/// Returns `focused_idx` (no movement) when:
/// - The edge widget is the currently focused one (single-item list).
/// - All widgets share the same position on the movement axis (e.g.
///   Left/Right in a purely vertical list), since wrapping would jump
///   to an arbitrary widget.
fn wrap_widget(scan: &SpatialScan, focused_idx: usize, direction: FocusDirection) -> usize {
    // Check whether there are at least two distinct positions along the
    // movement axis. Without that, wrapping has nowhere meaningful to go.
    let has_distinct_positions = {
        let tolerance = 1.0_f32;
        let first = scan.widgets.first().map(|w| match direction {
            FocusDirection::Left | FocusDirection::Right => w.1.x,
            FocusDirection::Up | FocusDirection::Down => w.1.y,
        });
        first.is_some_and(|first_val| {
            scan.widgets.iter().any(|w| {
                let val = match direction {
                    FocusDirection::Left | FocusDirection::Right => w.1.x,
                    FocusDirection::Up | FocusDirection::Down => w.1.y,
                };
                (val - first_val).abs() > tolerance
            })
        })
    };

    if !has_distinct_positions {
        return focused_idx;
    }

    // Wrap direction is the opposite of movement: pressing Up wraps
    // to the bottom edge, so we reuse `edge_widget` with the same direction
    // (Up → bottom-most is exactly what `edge_widget` already returns).
    let candidate = edge_widget(scan, direction);

    // If the edge widget is the currently focused one (single-item list),
    // don't move focus.
    if candidate == focused_idx {
        focused_idx
    } else {
        candidate
    }
}
