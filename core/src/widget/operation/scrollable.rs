//! Operate on widgets that can be scrolled.
use crate::animation::Easing;
use crate::widget::operation::Outcome;
use crate::widget::{Id, Operation};
use crate::{Rectangle, Vector};

/// The internal state of a widget that can be scrolled.
pub trait Scrollable {
    /// Snaps the scroll of the widget to the given `percentage` along the horizontal & vertical axis.
    fn snap_to(&mut self, offset: RelativeOffset<Option<f32>>);

    /// Scroll the widget to the given [`AbsoluteOffset`] along the horizontal & vertical axis.
    fn scroll_to(&mut self, offset: AbsoluteOffset<Option<f32>>);

    /// Scroll the widget by the given [`AbsoluteOffset`] along the horizontal & vertical axis.
    fn scroll_by(&mut self, offset: AbsoluteOffset, bounds: Rectangle, content_bounds: Rectangle);

    /// Start an animated scroll to the given [`AbsoluteOffset`].
    ///
    /// The default implementation falls back to [`scroll_to`](Scrollable::scroll_to)
    /// (instant, no animation).
    fn scroll_to_animated(
        &mut self,
        offset: AbsoluteOffset<Option<f32>>,
        _animation: ScrollAnimation,
    ) {
        self.scroll_to(offset);
    }
}

/// Configuration for animated scrolling.
///
/// When attached to an [`EnsureVisibleConfig`], the scrollable will smoothly
/// animate to the target position using the configured [`Easing`] curve.
#[derive(Debug, Clone, Copy)]
pub struct ScrollAnimation {
    /// Duration of the animation in seconds.
    pub duration_secs: f32,
    /// The easing curve applied to the animation.
    pub easing: Easing,
}

impl Default for ScrollAnimation {
    /// Default animation: 0.3 seconds with ease-out cubic.
    fn default() -> Self {
        Self {
            duration_secs: 0.3,
            easing: Easing::EaseOutCubic,
        }
    }
}

impl ScrollAnimation {
    /// Creates a new animation with the given duration in seconds
    /// and the default easing ([`Easing::EaseOutCubic`]).
    #[must_use]
    pub fn new(duration_secs: f32) -> Self {
        Self {
            duration_secs: duration_secs.max(0.0),
            easing: Easing::EaseOutCubic,
        }
    }

    /// Sets the easing curve for this animation.
    #[must_use]
    pub const fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }
}

/// Produces an [`Operation`] that snaps the widget with the given [`Id`] to
/// the provided `percentage`.
pub fn snap_to<T>(target: Id, offset: RelativeOffset<Option<f32>>) -> impl Operation<T> {
    struct SnapTo {
        target: Id,
        offset: RelativeOffset<Option<f32>>,
    }

    impl<T> Operation<T> for SnapTo {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }

        fn scrollable(
            &mut self,
            id: Option<&Id>,
            _bounds: Rectangle,
            _content_bounds: Rectangle,
            _translation: Vector,
            state: &mut dyn Scrollable,
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
pub fn scroll_to<T>(target: Id, offset: AbsoluteOffset<Option<f32>>) -> impl Operation<T> {
    struct ScrollTo {
        target: Id,
        offset: AbsoluteOffset<Option<f32>>,
    }

    impl<T> Operation<T> for ScrollTo {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }

        fn scrollable(
            &mut self,
            id: Option<&Id>,
            _bounds: Rectangle,
            _content_bounds: Rectangle,
            _translation: Vector,
            state: &mut dyn Scrollable,
        ) {
            if Some(&self.target) == id {
                state.scroll_to(self.offset);
            }
        }
    }

    ScrollTo { target, offset }
}

/// Produces an [`Operation`] that scrolls the widget with the given [`Id`] by
/// the provided [`AbsoluteOffset`].
pub fn scroll_by<T>(target: Id, offset: AbsoluteOffset) -> impl Operation<T> {
    struct ScrollBy {
        target: Id,
        offset: AbsoluteOffset,
    }

    impl<T> Operation<T> for ScrollBy {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }

        fn scrollable(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            content_bounds: Rectangle,
            _translation: Vector,
            state: &mut dyn Scrollable,
        ) {
            if Some(&self.target) == id {
                state.scroll_by(self.offset, bounds, content_bounds);
            }
        }
    }

    ScrollBy { target, offset }
}

/// The amount of absolute offset in each direction of a [`Scrollable`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AbsoluteOffset<T = f32> {
    /// The amount of horizontal offset
    pub x: T,
    /// The amount of vertical offset
    pub y: T,
}

impl From<AbsoluteOffset> for AbsoluteOffset<Option<f32>> {
    fn from(offset: AbsoluteOffset) -> Self {
        Self {
            x: Some(offset.x),
            y: Some(offset.y),
        }
    }
}

/// The amount of relative offset in each direction of a [`Scrollable`].
///
/// A value of `0.0` means start, while `1.0` means end.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RelativeOffset<T = f32> {
    /// The amount of horizontal offset
    pub x: T,
    /// The amount of vertical offset
    pub y: T,
}

impl RelativeOffset {
    /// A relative offset that points to the top-left of a [`Scrollable`].
    pub const START: Self = Self { x: 0.0, y: 0.0 };

    /// A relative offset that points to the bottom-right of a [`Scrollable`].
    pub const END: Self = Self { x: 1.0, y: 1.0 };
}

impl From<RelativeOffset> for RelativeOffset<Option<f32>> {
    fn from(offset: RelativeOffset) -> Self {
        Self {
            x: Some(offset.x),
            y: Some(offset.y),
        }
    }
}

/// Configuration for [`ensure_focused_visible`].
///
/// Controls how the focused widget is positioned within the scrollable
/// viewport after scrolling.
#[derive(Debug, Clone, Copy)]
pub struct EnsureVisibleConfig {
    /// Vertical alignment of the focused widget within the viewport.
    ///
    /// - `0.0` — align to the top edge
    /// - `0.35` — 35% from the top (Flutter Grid default)
    /// - `0.5` — center vertically
    /// - `1.0` — align to the bottom edge
    pub alignment_y: f32,

    /// Horizontal alignment of the focused widget within the viewport.
    ///
    /// - `0.0` — align to the left edge
    /// - `0.5` — center horizontally
    /// - `1.0` — align to the right edge
    pub alignment_x: f32,

    /// Optional animation. When `Some`, the scrollable animates smoothly
    /// to the target position. When `None`, it jumps instantly.
    pub animation: Option<ScrollAnimation>,
}

impl Default for EnsureVisibleConfig {
    /// Default: 35% alignment on both axes, animated with default duration.
    fn default() -> Self {
        Self {
            alignment_y: 0.35,
            alignment_x: 0.35,
            animation: Some(ScrollAnimation::default()),
        }
    }
}

impl EnsureVisibleConfig {
    /// Creates a config with custom vertical and horizontal alignment.
    /// Animated by default.
    #[must_use]
    pub fn new(alignment_y: f32, alignment_x: f32) -> Self {
        Self {
            alignment_y: alignment_y.clamp(0.0, 1.0),
            alignment_x: alignment_x.clamp(0.0, 1.0),
            animation: Some(ScrollAnimation::default()),
        }
    }

    /// Creates a config with a custom vertical alignment.
    ///
    /// Horizontal alignment defaults to `0.35`. Animated by default.
    #[must_use]
    pub fn vertical(alignment_y: f32) -> Self {
        Self {
            alignment_y: alignment_y.clamp(0.0, 1.0),
            alignment_x: 0.35,
            animation: Some(ScrollAnimation::default()),
        }
    }

    /// Align the focused widget to the center of the viewport on both axes.
    /// Animated by default.
    pub const CENTER: Self = Self {
        alignment_y: 0.5,
        alignment_x: 0.5,
        animation: Some(ScrollAnimation {
            duration_secs: 0.3,
            easing: Easing::EaseOutCubic,
        }),
    };

    /// Sets the animation configuration.
    #[must_use]
    pub const fn with_animation(mut self, animation: ScrollAnimation) -> Self {
        self.animation = Some(animation);
        self
    }

    /// Disables animation — the scroll will jump instantly.
    #[must_use]
    pub const fn instant(mut self) -> Self {
        self.animation = None;
        self
    }
}

/// A self-contained operation that finds the focused widget inside the nearest
/// scrollable ancestor and scrolls it into view — all in one step, without
/// requiring the scrollable to have an explicit [`Id`].
///
/// Phase 1 walks the tree to find the focused widget and its scrollable
/// context. Phase 2 (via [`Outcome::Chain`]) walks the tree again and
/// calls [`Scrollable::scroll_to`] on the matching scrollable.
pub fn ensure_focused_visible_op<T>(config: EnsureVisibleConfig) -> impl Operation<T> {
    EnsureFocusedVisibleOp {
        config,
        scrollable_ctx: None,
        scrollable_index: 0,
        target: None,
    }
}

struct EnsureFocusedVisibleOp {
    config: EnsureVisibleConfig,
    /// The most recently encountered scrollable (tracked by index).
    scrollable_ctx: Option<(usize, ScrollableContext)>,
    scrollable_index: usize,
    /// The computed scroll target: (scrollable tree index, offset).
    target: Option<(usize, AbsoluteOffset<Option<f32>>)>,
}

#[derive(Clone)]
struct ScrollableContext {
    bounds: Rectangle,
    content_bounds: Rectangle,
}

impl<T> Operation<T> for EnsureFocusedVisibleOp {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
        operate(self);
    }

    fn scrollable(
        &mut self,
        _id: Option<&Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        _translation: Vector,
        _state: &mut dyn Scrollable,
    ) {
        if self.target.is_some() {
            return;
        }

        self.scrollable_ctx = Some((
            self.scrollable_index,
            ScrollableContext {
                bounds,
                content_bounds,
            },
        ));
        self.scrollable_index += 1;
    }

    fn focusable(&mut self, _id: Option<&Id>, bounds: Rectangle, state: &mut dyn super::Focusable) {
        if self.target.is_some() || !state.is_focused() {
            return;
        }

        let Some((idx, ctx)) = &self.scrollable_ctx else {
            return;
        };

        if let Some(offset) =
            compute_scroll_offset(bounds, ctx.bounds, ctx.content_bounds, self.config)
        {
            self.target = Some((*idx, offset));
        }
    }

    fn finish(&self) -> Outcome<T> {
        if let Some((idx, offset)) = &self.target {
            Outcome::Chain(Box::new(ApplyScrollAtIndex {
                target_index: *idx,
                current: 0,
                offset: *offset,
                animation: self.config.animation,
            }))
        } else {
            Outcome::None
        }
    }
}

/// Phase 2: walks the tree counting scrollables and scrolls the one
/// at the recorded index.
struct ApplyScrollAtIndex {
    target_index: usize,
    current: usize,
    offset: AbsoluteOffset<Option<f32>>,
    animation: Option<ScrollAnimation>,
}

impl<T> Operation<T> for ApplyScrollAtIndex {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
        operate(self);
    }

    fn scrollable(
        &mut self,
        _id: Option<&Id>,
        _bounds: Rectangle,
        _content_bounds: Rectangle,
        _translation: Vector,
        state: &mut dyn Scrollable,
    ) {
        if self.current == self.target_index {
            if let Some(animation) = self.animation {
                state.scroll_to_animated(self.offset, animation);
            } else {
                state.scroll_to(self.offset);
            }
        }
        self.current += 1;
    }
}

/// Computes the absolute scroll offset needed to position a widget
/// at the configured alignment within a scrollable viewport.
///
/// Layout bounds inside a scrollable are **not** translated by the scroll
/// offset — translation is only applied during rendering. So the
/// content-relative position is simply `bounds.origin - scrollable.origin`.
pub fn compute_scroll_offset(
    focused_bounds: Rectangle,
    scrollable_bounds: Rectangle,
    content_bounds: Rectangle,
    config: EnsureVisibleConfig,
) -> Option<AbsoluteOffset<Option<f32>>> {
    // --- Vertical axis ---
    let target_y = if content_bounds.height > scrollable_bounds.height {
        let focused_y = focused_bounds.y - scrollable_bounds.y;
        let focused_center_y = focused_y + focused_bounds.height / 2.0;
        let viewport_target_y = scrollable_bounds.height * config.alignment_y;
        let desired = focused_center_y - viewport_target_y;
        let max_scroll = content_bounds.height - scrollable_bounds.height;
        Some(desired.clamp(0.0, max_scroll))
    } else {
        None
    };

    // --- Horizontal axis ---
    let target_x = if content_bounds.width > scrollable_bounds.width {
        let focused_x = focused_bounds.x - scrollable_bounds.x;
        let focused_center_x = focused_x + focused_bounds.width / 2.0;
        let viewport_target_x = scrollable_bounds.width * config.alignment_x;
        let desired = focused_center_x - viewport_target_x;
        let max_scroll = content_bounds.width - scrollable_bounds.width;
        Some(desired.clamp(0.0, max_scroll))
    } else {
        None
    };

    if target_x.is_some() || target_y.is_some() {
        Some(AbsoluteOffset {
            x: target_x,
            y: target_y,
        })
    } else {
        None
    }
}
