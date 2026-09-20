//! Integration tests for the `scrollable` widget.
//!
//! These tests drive the widget through the `Simulator` and observe its
//! behavior through the `on_scroll` notifications: no redundant viewports
//! are published, so the number of messages also tells us how many frames
//! the smooth scrolling animation took.
use iced_test::Simulator;
use iced_widget::Renderer;
use iced_widget::core::layout::{self, Layout};
use iced_widget::core::mouse::{self, ScrollDelta};
use iced_widget::core::widget::{Id, Tree, operation};
use iced_widget::core::window;
use iced_widget::core::{self, Event, Length, Point, Rectangle, Size, Theme};
use iced_widget::scrollable::{
    AbsoluteOffset, Direction, RelativeOffset, Scroll, Scrollable, Scrollbar, Source, Viewport,
};
use iced_widget::space;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// One frame of the simulated animation.
const FRAME: Duration = Duration::from_millis(16);

/// The distance (in pixels) scrolled per wheel line.
///
/// Keep in sync with the private `WHEEL_PX_PER_LINE` constant in the
/// `scrollable` module.
const PX_PER_LINE: f32 = 120.0;

/// The distance (in pixels) scrolled by a wheel event of the given number
/// of lines.
fn px(lines: f32) -> f32 {
    lines * PX_PER_LINE
}

/// A `Scrollable` with the given content and viewport heights, publishing
/// its viewport on every scroll.
fn element(
    content_height: u32,
    viewport_height: u32,
) -> Scrollable<'static, Scroll, Theme, Renderer> {
    Scrollable::new(space().height(content_height))
        .id("scrollable")
        .width(Length::Fill)
        .height(viewport_height)
        .on_scroll(Some)
}

/// The [`Id`] of the [`element`] under test.
const ELEMENT_ID: Id = Id::new("scrollable");

/// Steps the given number of frames, keeping the real-time clock ahead of
/// the synthetic frame timeline.
///
/// The `Simulator` timestamps wheel events with the real-time clock, so if
/// the synthetic frame instants run ahead of it, a wheel event would be
/// observed "in the past" of the animation's timeline. Sleeping for each
/// frame guarantees the two timelines stay consistent.
fn step_frames(simulator: &mut Simulator<'_, Scroll>, instant: &mut Instant, frames: usize) {
    for _ in 0..frames {
        *instant += FRAME;
        std::thread::sleep(FRAME);

        let _ = simulator.simulate([Event::Window(window::Event::RedrawRequested(*instant))]);
    }
}

#[test]
fn wheel_scrolling_is_smooth() {
    let mut simulator = Simulator::new(element(3000, 200));
    simulator.point_at(Point::new(500.0, 100.0));

    // Scroll down two lines
    let _ = simulator.scroll(ScrollDelta::Lines { x: 0.0, y: -2.0 });

    // Settle the animation, frame by frame
    let mut instant = Instant::now();

    for _ in 0..60 {
        instant += FRAME;

        let _ = simulator.simulate([Event::Window(window::Event::RedrawRequested(instant))]);
    }

    let (offsets, animating): (Vec<f32>, Vec<bool>) = simulator
        .into_messages()
        .map(|scroll| (scroll.viewport.absolute_offset().y, scroll.target.is_some()))
        .unzip();

    // The wheel event must not have moved the offset: the first
    // notification still reports the original position
    assert!(
        !offsets.is_empty(),
        "expected on_scroll notifications, got: {offsets:?}"
    );
    assert_eq!(
        offsets.first(),
        Some(&0.0),
        "offset moved before the first frame: {offsets:?}"
    );

    // The scroll must have been gradual: it took several frames to settle
    assert!(
        offsets.len() > 2,
        "expected multiple intermediate offsets, got: {offsets:?}"
    );
    assert!(
        *offsets.get(1).unwrap() < px(2.0) / 2.0,
        "scrolling should not jump instantly: {offsets:?}"
    );

    // ... and it must have settled exactly on the target (two lines)
    assert_eq!(offsets.last(), Some(&px(2.0)));

    // The notifications report the animation as in progress until the
    // scroll settles
    assert!(
        animating[..animating.len() - 1]
            .iter()
            .all(|&is_animating| is_animating),
        "expected the scroll to be animating until it settles: {offsets:?} {animating:?}"
    );
    assert!(
        !animating.last().unwrap(),
        "expected the scroll to have settled: {offsets:?} {animating:?}"
    );
}

#[test]
fn wheel_scrolling_is_immediate_when_smooth_scroll_disabled() {
    let mut simulator = Simulator::new(element(3000, 200).smooth_scroll(false));
    simulator.point_at(Point::new(500.0, 100.0));

    // Scroll down two lines
    let _ = simulator.scroll(ScrollDelta::Lines { x: 0.0, y: -2.0 });

    // No frames needed: a single notification, fully scrolled,
    // on the wheel event itself — and no animation
    let notifications: Vec<(f32, bool)> = simulator
        .into_messages()
        .map(|scroll| (scroll.viewport.absolute_offset().y, scroll.target.is_some()))
        .collect();

    assert_eq!(notifications, [(px(2.0), false)]);
}

#[test]
fn pixel_scrolling_is_immediate_even_when_smooth_scroll_enabled() {
    let mut simulator = Simulator::new(element(3000, 200));
    simulator.point_at(Point::new(500.0, 100.0));

    // Scroll down 120 pixels
    let _ = simulator.scroll(ScrollDelta::Pixels { x: 0.0, y: -120.0 });

    // High-precision scrolls are applied immediately, even though
    // smooth scrolling is enabled by default — and there is no animation
    let notifications: Vec<(f32, bool)> = simulator
        .into_messages()
        .map(|scroll| (scroll.viewport.absolute_offset().y, scroll.target.is_some()))
        .collect();

    assert_eq!(notifications, [(120.0, false)]);
}

#[test]
fn smooth_scroll_settles_on_target() {
    let mut simulator = Simulator::new(element(3000, 200));
    simulator.point_at(Point::new(500.0, 100.0));

    // Scroll down one line
    let _ = simulator.scroll(ScrollDelta::Lines { x: 0.0, y: -1.0 });

    // Settle the animation, frame by frame
    let mut instant = Instant::now();

    for _ in 0..60 {
        instant += FRAME;

        let _ = simulator.simulate([Event::Window(window::Event::RedrawRequested(instant))]);
    }

    let offsets: Vec<f32> = simulator
        .into_messages()
        .map(|scroll| scroll.viewport.absolute_offset().y)
        .collect();

    // The offset must approach the target monotonically
    let mut previous = 0.0;

    for offset in &offsets {
        assert!(
            *offset >= previous,
            "offset must not scroll backwards: {offsets:?}"
        );

        previous = *offset;
    }

    assert!(
        offsets.len() > 2,
        "smooth scroll must take more than one frame: {offsets:?}"
    );

    // ... and it must settle exactly on the target (one line)
    assert_eq!(offsets.last(), Some(&px(1.0)));
}

#[test]
fn smooth_scroll_accumulates_pending_delta() {
    let mut simulator = Simulator::new(element(3000, 200));
    simulator.point_at(Point::new(500.0, 100.0));

    // Scroll down one line...
    let _ = simulator.scroll(ScrollDelta::Lines { x: 0.0, y: -1.0 });

    // ...a few frames in, keeping real time consistent with the animation's
    // timeline...
    let mut instant = Instant::now();
    step_frames(&mut simulator, &mut instant, 3);

    // ...a new wheel movement must add to the pending target, not replace it
    let _ = simulator.scroll(ScrollDelta::Lines { x: 0.0, y: -0.5 });

    // Settle the animation, frame by frame
    for _ in 0..60 {
        instant += FRAME;

        let _ = simulator.simulate([Event::Window(window::Event::RedrawRequested(instant))]);
    }

    let offsets: Vec<f32> = simulator
        .into_messages()
        .map(|scroll| scroll.viewport.absolute_offset().y)
        .collect();

    // The offset must not scroll backwards
    let mut previous = 0.0;

    for offset in &offsets {
        assert!(
            *offset >= previous,
            "offset must not scroll backwards: {offsets:?}"
        );

        previous = *offset;
    }

    // The full distance of both movements was scrolled
    assert_eq!(offsets.last(), Some(&px(1.5)));
}

#[test]
fn smooth_scroll_duration_scales_inversely_with_distance() {
    /// The number of messages (i.e. scrolled frames) a wheel movement of the
    /// given number of lines takes to settle.
    fn settle_frames(lines: f32) -> Vec<f32> {
        let mut simulator = Simulator::new(element(3000, 200));
        simulator.point_at(Point::new(500.0, 100.0));

        let _ = simulator.scroll(ScrollDelta::Lines { x: 0.0, y: -lines });

        let mut instant = Instant::now();

        for _ in 0..60 {
            instant += FRAME;

            let _ = simulator.simulate([Event::Window(window::Event::RedrawRequested(instant))]);
        }

        simulator
            .into_messages()
            .map(|scroll| scroll.viewport.absolute_offset().y)
            .collect()
    }

    // No redundant viewports are published, so the number of messages tells
    // us how many frames the animation took

    let short = settle_frames(1.0);
    let medium = settle_frames(2.5);
    let long = settle_frames(10.0);

    assert_eq!(short.last(), Some(&px(1.0)));
    assert_eq!(medium.last(), Some(&px(2.5)));
    assert_eq!(long.last(), Some(&px(10.0)));

    // The longer the distance, the shorter (snappier) the animation
    assert!(
        short.len() > medium.len(),
        "a short scroll ({:?}) must take more frames than a medium one ({:?})",
        short,
        medium
    );
    assert!(
        medium.len() > long.len(),
        "a medium scroll ({:?}) must take more frames than a long one ({:?})",
        medium,
        long
    );
}

#[test]
fn smooth_scroll_retarget_preserves_velocity() {
    let mut simulator = Simulator::new(element(3000, 200));
    simulator.point_at(Point::new(500.0, 100.0));

    // Scrolling down...
    let _ = simulator.scroll(ScrollDelta::Lines { x: 0.0, y: -1.0 });

    // ...a few frames in, keeping real time consistent with the animation's
    // timeline...
    let mut instant = Instant::now();
    step_frames(&mut simulator, &mut instant, 3);

    // ...a new wheel movement retargets the animation
    let _ = simulator.scroll(ScrollDelta::Lines { x: 0.0, y: -0.5 });

    // ...and the animation continues, frame by frame
    for _ in 0..60 {
        instant += FRAME;

        let _ = simulator.simulate([Event::Window(window::Event::RedrawRequested(instant))]);
    }

    let offsets: Vec<f32> = simulator
        .into_messages()
        .map(|scroll| scroll.viewport.absolute_offset().y)
        .collect();

    // The wheel event itself does not publish a new viewport (the offset
    // has not moved yet), so:
    // - `offsets[2]` and `offsets[3]` are the last two frames of the
    //   original animation, and
    // - `offsets[4]` is the first frame of the retargeted one
    let before = offsets[3] - offsets[2];
    let after = offsets[4] - offsets[3];

    // The retargeted animation must keep the velocity of the original one:
    // it must not restart from rest
    assert!(
        after >= before / 2.0,
        "expected the retarget to preserve most of the previous velocity \
         ({before}px/frame), got {after}px/frame: {offsets:?}"
    );

    // ...and it settles exactly on the target (one and a half lines)
    assert_eq!(offsets.last(), Some(&px(1.5)));
}

#[test]
fn smooth_scroll_reversal_carries_momentum() {
    let mut simulator = Simulator::new(element(3000, 200));
    simulator.point_at(Point::new(500.0, 100.0));

    // Scrolling down...
    let _ = simulator.scroll(ScrollDelta::Lines { x: 0.0, y: -1.0 });

    // ...a few frames in, keeping real time consistent with the animation's
    // timeline...
    let mut instant = Instant::now();
    step_frames(&mut simulator, &mut instant, 3);

    // ...and then the user scrolls back up past the current position
    let _ = simulator.scroll(ScrollDelta::Lines { x: 0.0, y: 1.0 });

    // ...and the animation continues, frame by frame
    for _ in 0..60 {
        instant += FRAME;

        let _ = simulator.simulate([Event::Window(window::Event::RedrawRequested(instant))]);
    }

    let offsets: Vec<f32> = simulator
        .into_messages()
        .map(|scroll| scroll.viewport.absolute_offset().y)
        .collect();

    // The wheel event itself does not publish a new viewport (the offset
    // has not moved yet), so:
    // - `offsets[3]` is the last frame of the original animation, and
    // - `offsets[4]` is the first frame of the retargeted one

    // The retargeted animation must keep moving down (carrying the previous
    // momentum) before it turns up
    assert!(
        offsets[4] > offsets[3],
        "expected the reversal to carry the previous momentum: {offsets:?}"
    );

    // ...and it settles exactly back on the original position
    assert_eq!(offsets.last(), Some(&0.0));
}

#[test]
fn high_precision_scrolling_cancels_smooth_scroll() {
    let mut simulator = Simulator::new(element(3000, 200));
    simulator.point_at(Point::new(500.0, 100.0));

    // Start a smooth scroll...
    let _ = simulator.scroll(ScrollDelta::Lines { x: 0.0, y: -1.0 });

    let mut instant = Instant::now();

    for _ in 0..2 {
        instant += FRAME;

        let _ = simulator.simulate([Event::Window(window::Event::RedrawRequested(instant))]);
    }

    // ...and then a high-precision scroll arrives: it must be applied
    // immediately, and cancel the pending smooth scroll
    let _ = simulator.scroll(ScrollDelta::Pixels { x: 0.0, y: -10.0 });

    // Settle for a while longer
    for _ in 0..30 {
        instant += FRAME;

        let _ = simulator.simulate([Event::Window(window::Event::RedrawRequested(instant))]);
    }

    let (offsets, animating): (Vec<f32>, Vec<bool>) = simulator
        .into_messages()
        .map(|scroll| (scroll.viewport.absolute_offset().y, scroll.target.is_some()))
        .unzip();

    // Exactly four notifications: the wheel event (at the original
    // position), the two frames of the smooth scroll, and the immediate
    // high-precision scroll — and nothing after that
    assert_eq!(offsets.len(), 4, "unexpected notifications: {offsets:?}");

    // The high-precision scroll moved the offset by exactly its delta
    assert!((offsets[3] - (offsets[2] + 10.0)).abs() < 1e-3);

    // The notifications report the smooth scroll as in progress until the
    // high-precision scroll cancels it
    assert_eq!(&animating[..], &[true, true, true, false]);
}

#[test]
fn smooth_scroll_is_noop_without_overflow() {
    // The content fits: there is nothing to scroll
    let mut simulator = Simulator::new(element(100, 200));
    simulator.point_at(Point::new(500.0, 100.0));

    let _ = simulator.scroll(ScrollDelta::Lines { x: 0.0, y: -1.0 });

    let mut instant = Instant::now();

    for _ in 0..30 {
        instant += FRAME;

        let _ = simulator.simulate([Event::Window(window::Event::RedrawRequested(instant))]);
    }

    // Nothing is published: no offset ever moved
    let offsets: Vec<f32> = simulator
        .into_messages()
        .map(|scroll| scroll.viewport.absolute_offset().y)
        .collect();

    assert!(offsets.is_empty(), "unexpected notifications: {offsets:?}");
}

#[test]
fn scroll_to_with_smooth_behavior_is_smooth() {
    let mut simulator = Simulator::new(element(3000, 200));

    let mut operation = operation::scrollable::scroll_to(
        ELEMENT_ID,
        AbsoluteOffset {
            x: None,
            y: Some(px(2.0)),
        },
        operation::Animation::Smooth,
    );

    simulator.operate(&mut operation);

    // Settle the animation, frame by frame
    let mut instant = Instant::now();
    step_frames(&mut simulator, &mut instant, 60);

    let (offsets, animating): (Vec<f32>, Vec<bool>) = simulator
        .into_messages()
        .map(|scroll| (scroll.viewport.absolute_offset().y, scroll.target.is_some()))
        .unzip();

    // The operation is backdated by one nominal frame: the first drawn
    // frame must already show progress
    assert!(
        offsets.first() > Some(&0.0) && offsets.first() < Some(&px(2.0)),
        "the first frame must already show progress: {offsets:?}"
    );

    // The scroll must have been gradual: it took several frames to settle
    assert!(
        offsets.len() > 2,
        "expected multiple intermediate offsets, got: {offsets:?}"
    );

    // ... and it must have settled exactly on the target (two lines)
    assert_eq!(offsets.last(), Some(&px(2.0)));

    // The notifications report the animation as in progress until the
    // scroll settles
    assert!(
        animating[..animating.len() - 1]
            .iter()
            .all(|&is_animating| is_animating),
        "expected the scroll to be animating until it settles: {offsets:?} {animating:?}"
    );
    assert!(
        !animating.last().unwrap(),
        "expected the scroll to have settled: {offsets:?} {animating:?}"
    );
}

#[test]
fn scroll_to_with_instant_behavior_is_immediate() {
    let mut simulator = Simulator::new(element(3000, 200));

    let mut operation = operation::scrollable::scroll_to(
        ELEMENT_ID,
        AbsoluteOffset {
            x: None,
            y: Some(px(2.0)),
        },
        operation::Animation::Instant,
    );

    simulator.operate(&mut operation);

    let mut instant = Instant::now();
    step_frames(&mut simulator, &mut instant, 2);

    let (offsets, animating): (Vec<f32>, Vec<bool>) = simulator
        .into_messages()
        .map(|scroll| (scroll.viewport.absolute_offset().y, scroll.target.is_some()))
        .unzip();

    // The offset must be at the target on the very first frame, with no
    // animation in between
    assert_eq!(
        offsets.len(),
        1,
        "expected a single notification, got: {offsets:?}"
    );
    assert_eq!(offsets.first(), Some(&px(2.0)));
    assert!(!animating[0]);
}

#[test]
fn scroll_to_with_auto_behavior_follows_smooth_scroll() {
    // With smooth scrolling enabled (the default), `Auto` resolves to a
    // smooth scroll
    {
        let mut simulator = Simulator::new(element(3000, 200));

        let mut operation = operation::scrollable::scroll_to(
            ELEMENT_ID,
            AbsoluteOffset {
                x: None,
                y: Some(px(2.0)),
            },
            operation::Animation::Auto,
        );

        simulator.operate(&mut operation);

        let mut instant = Instant::now();
        step_frames(&mut simulator, &mut instant, 60);

        let offsets: Vec<f32> = simulator
            .into_messages()
            .map(|scroll| scroll.viewport.absolute_offset().y)
            .collect();

        assert!(
            offsets.len() > 2,
            "`Auto` must follow the enabled smooth scrolling: {offsets:?}"
        );
        assert_eq!(offsets.last(), Some(&px(2.0)));
    }

    // ... while with smooth scrolling disabled, it resolves to an immediate
    // scroll
    {
        let mut simulator = Simulator::new(element(3000, 200).smooth_scroll(false));

        let mut operation = operation::scrollable::scroll_to(
            ELEMENT_ID,
            AbsoluteOffset {
                x: None,
                y: Some(px(2.0)),
            },
            operation::Animation::Auto,
        );

        simulator.operate(&mut operation);

        let mut instant = Instant::now();
        step_frames(&mut simulator, &mut instant, 2);

        let offsets: Vec<f32> = simulator
            .into_messages()
            .map(|scroll| scroll.viewport.absolute_offset().y)
            .collect();

        assert_eq!(
            offsets.len(),
            1,
            "`Auto` must follow the disabled smooth scrolling: {offsets:?}"
        );
        assert_eq!(offsets.first(), Some(&px(2.0)));
    }
}

#[test]
fn snap_to_with_smooth_behavior_settles_at_percentage() {
    let mut simulator = Simulator::new(element(3000, 200));

    let mut operation = operation::scrollable::snap_to(
        ELEMENT_ID,
        RelativeOffset {
            x: None,
            y: Some(1.0),
        },
        operation::Animation::Smooth,
    );

    simulator.operate(&mut operation);

    let mut instant = Instant::now();
    step_frames(&mut simulator, &mut instant, 60);

    let offsets: Vec<f32> = simulator
        .into_messages()
        .map(|scroll| scroll.viewport.absolute_offset().y)
        .collect();

    // The 100% snap must settle exactly at the end of the content
    assert_eq!(offsets.last(), Some(&(3000.0 - 200.0)));
    assert!(
        offsets.len() > 2,
        "expected a smooth animation: {offsets:?}"
    );
}

#[test]
fn scroll_by_with_smooth_behavior_accumulates() {
    let mut simulator = Simulator::new(element(3000, 200));

    // Start a smooth scroll of one line...
    let mut operation = operation::scrollable::scroll_by(
        ELEMENT_ID,
        AbsoluteOffset { x: 0.0, y: px(1.0) },
        operation::Animation::Smooth,
    );

    simulator.operate(&mut operation);

    let mut instant = Instant::now();
    step_frames(&mut simulator, &mut instant, 2);

    // ... and add another line while it is still running
    let mut operation = operation::scrollable::scroll_by(
        ELEMENT_ID,
        AbsoluteOffset { x: 0.0, y: px(1.0) },
        operation::Animation::Smooth,
    );

    simulator.operate(&mut operation);
    step_frames(&mut simulator, &mut instant, 60);

    let offsets: Vec<f32> = simulator
        .into_messages()
        .map(|scroll| scroll.viewport.absolute_offset().y)
        .collect();

    // The delta must accumulate onto the pending target
    assert_eq!(offsets.last(), Some(&px(2.0)));
}

#[test]
fn scroll_to_with_smooth_behavior_replaces_pending_target() {
    let mut simulator = Simulator::new(element(3000, 200));

    // Start a smooth scroll of one line...
    let mut operation = operation::scrollable::scroll_by(
        ELEMENT_ID,
        AbsoluteOffset { x: 0.0, y: px(1.0) },
        operation::Animation::Smooth,
    );

    simulator.operate(&mut operation);

    let mut instant = Instant::now();
    step_frames(&mut simulator, &mut instant, 2);

    // ... then scroll to an absolute target while it is still running
    let mut operation = operation::scrollable::scroll_to(
        ELEMENT_ID,
        AbsoluteOffset {
            x: None,
            y: Some(px(3.0)),
        },
        operation::Animation::Smooth,
    );

    simulator.operate(&mut operation);
    step_frames(&mut simulator, &mut instant, 60);

    let offsets: Vec<f32> = simulator
        .into_messages()
        .map(|scroll| scroll.viewport.absolute_offset().y)
        .collect();

    // The absolute target must replace the pending one, not accumulate
    assert_eq!(offsets.last(), Some(&px(3.0)));
}

/// A single observation of the content's coordinates, as seen by one pass
/// (`update` or `draw`) within a frame.
#[derive(Debug)]
struct Observation {
    drawn: bool,
    cursor: f32,
    viewport: f32,
}

/// A child widget that records the `cursor` and `viewport` it receives in
/// both `update` and `draw`, so we can assert that the two passes agree
/// within a frame.
struct Recorder {
    observations: Rc<RefCell<Vec<Observation>>>,
}

impl Recorder {
    fn observe(&self, drawn: bool, cursor: mouse::Cursor, viewport: &Rectangle) {
        let mut observations = self.observations.borrow_mut();

        observations.push(Observation {
            drawn,
            cursor: cursor
                .position()
                .map(|position| position.y)
                .unwrap_or(f32::NAN),
            viewport: viewport.y,
        });
    }
}

impl<Message, Theme, Renderer> core::Widget<Message, Theme, Renderer> for Recorder
where
    Renderer: core::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, Length::Fill, Length::Fixed(1000.0))
    }

    fn update(
        &mut self,
        _tree: &mut Tree,
        _event: &Event,
        _layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _shell: &mut core::Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.observe(false, cursor, viewport);
    }

    fn draw(
        &self,
        _tree: &Tree,
        _renderer: &mut Renderer,
        _theme: &Theme,
        _style: &core::renderer::Style,
        _layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.observe(true, cursor, viewport);
    }
}

impl<'a, Message, Theme, Renderer> From<Recorder> for core::Element<'a, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
    Message: 'a,
{
    fn from(recorder: Recorder) -> core::Element<'a, Message, Theme, Renderer> {
        core::Element::new(recorder)
    }
}

#[test]
fn content_update_and_draw_see_the_same_translation() {
    let observations = Rc::new(RefCell::new(Vec::new()));

    let element: Scrollable<'static, Viewport, Theme, Renderer> = Scrollable::new(Recorder {
        observations: observations.clone(),
    })
    .width(Length::Fill)
    .height(200);

    let mut simulator = Simulator::new(element);
    simulator.point_at(Point::new(500.0, 100.0));

    // A smooth scroll moves the content on every redraw frame; within
    // each frame the content's `update` and `draw` must observe the same
    // translation
    let _ = simulator.scroll(ScrollDelta::Lines { x: 0.0, y: -5.0 });

    for _ in 0..30 {
        let before = observations.borrow().len();

        // A frame: the `RedrawRequested` update pass and the draw pass
        simulator.draw(&Theme::Dark);

        let current = observations.borrow_mut().split_off(before);

        let (Some(update), Some(draw)) = (
            current.iter().find(|observation| !observation.drawn),
            current.iter().find(|observation| observation.drawn),
        ) else {
            panic!("expected an `update` and a `draw` observation: {current:?}");
        };

        assert_eq!(
            update.viewport, draw.viewport,
            "content `update` and `draw` disagree on the viewport within a frame"
        );

        assert!(
            update.cursor == draw.cursor || (update.cursor.is_nan() && draw.cursor.is_nan()),
            "content `update` and `draw` disagree on the cursor within a frame: \
             {update:?} vs {draw:?}"
        );

        // Give the animation a frame's worth of time before the next redraw
        std::thread::sleep(FRAME);
    }
}

#[test]
fn snap_to_end_with_smooth_behavior_stays_snapped_when_content_grows() {
    let mut simulator = Simulator::new(element(3000, 200));

    let mut operation = operation::scrollable::snap_to(
        ELEMENT_ID,
        RelativeOffset {
            x: None,
            y: Some(1.0),
        },
        operation::Animation::Smooth,
    );

    simulator.operate(&mut operation);

    // Settle the animation, frame by frame
    let mut instant = Instant::now();
    step_frames(&mut simulator, &mut instant, 60);

    // Grow the content while keeping the state, like an app would on a new
    // message
    simulator = simulator.rebuild(element(6000, 200));

    // The snapped (relative) offset must re-resolve to the new end
    step_frames(&mut simulator, &mut instant, 2);

    let offsets: Vec<f32> = simulator
        .into_messages()
        .map(|scroll| scroll.viewport.absolute_offset().y)
        .collect();

    assert_eq!(
        offsets.last(),
        Some(&5800.0),
        "the smooth snap must stay snapped to the end after the content grows: {offsets:?}"
    );
}

#[test]
fn snap_to_end_with_smooth_behavior_tracks_content_growth_mid_scroll() {
    let mut simulator = Simulator::new(element(3000, 200));

    let mut operation = operation::scrollable::snap_to(
        ELEMENT_ID,
        RelativeOffset {
            x: None,
            y: Some(1.0),
        },
        operation::Animation::Smooth,
    );

    simulator.operate(&mut operation);

    // A few frames into the animation, grow the content
    let mut instant = Instant::now();
    step_frames(&mut simulator, &mut instant, 5);
    simulator = simulator.rebuild(element(6000, 200));

    // Settle the retargeted animation
    step_frames(&mut simulator, &mut instant, 60);

    let offsets: Vec<f32> = simulator
        .into_messages()
        .map(|scroll| scroll.viewport.absolute_offset().y)
        .collect();

    // The animation must retarget to the new end, not settle on the stale one
    assert_eq!(
        offsets.last(),
        Some(&5800.0),
        "the scroll must settle on the grown end: {offsets:?}"
    );

    // ... and the offset must never move backwards along the way
    assert!(
        offsets.windows(2).all(|pair| pair[1] >= pair[0]),
        "the offset must not jump: {offsets:?}"
    );
}

#[test]
fn wheel_during_smooth_snap_to_end_drops_the_snap() {
    let mut simulator = Simulator::new(element(3000, 200));
    simulator.point_at(Point::new(500.0, 100.0));

    let mut operation = operation::scrollable::snap_to(
        ELEMENT_ID,
        RelativeOffset {
            x: None,
            y: Some(1.0),
        },
        operation::Animation::Smooth,
    );

    simulator.operate(&mut operation);

    // A few frames into the animation, scroll up one line with the wheel
    let mut instant = Instant::now();
    step_frames(&mut simulator, &mut instant, 5);

    let _ = simulator.scroll(ScrollDelta::Lines { x: 0.0, y: 1.0 });

    // Settle the retargeted animation: the pending destination (the end,
    // 2800 px) minus one line (120 px)
    step_frames(&mut simulator, &mut instant, 60);

    // The snap must have been replaced by the wheel's absolute target:
    // growing the content no longer moves the offset
    simulator = simulator.rebuild(element(6000, 200));
    step_frames(&mut simulator, &mut instant, 2);

    let offsets: Vec<f32> = simulator
        .into_messages()
        .map(|scroll| scroll.viewport.absolute_offset().y)
        .collect();

    assert_eq!(
        offsets.last(),
        Some(&2680.0),
        "the wheel must replace the snap with an absolute target: {offsets:?}"
    );
}

#[test]
fn snap_to_end_with_smooth_behavior_stays_snapped_when_already_at_destination() {
    let mut simulator = Simulator::new(element(3000, 200));

    // Scroll to the end with an immediate (absolute) offset
    let mut operation = operation::scrollable::scroll_to(
        ELEMENT_ID,
        AbsoluteOffset {
            x: None,
            y: Some(2800.0),
        },
        operation::Animation::Instant,
    );

    simulator.operate(&mut operation);

    // Let the immediate scroll notify
    let mut instant = Instant::now();
    step_frames(&mut simulator, &mut instant, 2);

    // Snap to the end smoothly: there is nothing to animate, but the offset
    // must still become relative (snapped)
    let mut operation = operation::scrollable::snap_to(
        ELEMENT_ID,
        RelativeOffset {
            x: None,
            y: Some(1.0),
        },
        operation::Animation::Smooth,
    );

    simulator.operate(&mut operation);

    // Grow the content: the snapped offset must pin the scroll to the end
    simulator = simulator.rebuild(element(6000, 200));
    step_frames(&mut simulator, &mut instant, 2);

    let offsets: Vec<f32> = simulator
        .into_messages()
        .map(|scroll| scroll.viewport.absolute_offset().y)
        .collect();

    assert_eq!(
        offsets.last(),
        Some(&5800.0),
        "the snap must apply even when already at the destination: {offsets:?}"
    );
}

#[test]
fn snap_to_end_with_smooth_behavior_stays_snapped_without_overflow() {
    let mut simulator = Simulator::new(element(100, 200));

    let mut operation = operation::scrollable::snap_to(
        ELEMENT_ID,
        RelativeOffset {
            x: None,
            y: Some(1.0),
        },
        operation::Animation::Smooth,
    );

    simulator.operate(&mut operation);

    // The content does not overflow, so there is nothing to animate; the
    // snapped (relative) offset must still be applied
    let mut instant = Instant::now();
    step_frames(&mut simulator, &mut instant, 2);

    // Once the content overflows, the snapped offset must pin the scroll
    // to the end
    simulator = simulator.rebuild(element(3000, 200));
    step_frames(&mut simulator, &mut instant, 2);

    let offsets: Vec<f32> = simulator
        .into_messages()
        .map(|scroll| scroll.viewport.absolute_offset().y)
        .collect();

    assert_eq!(
        offsets.last(),
        Some(&2800.0),
        "the snap must apply even without overflow: {offsets:?}"
    );
}

/// A `Scrollable` with content of the given size and a 1024 × 200 viewport,
/// so that the content overflows on both axes.
fn element_xy(
    content_width: u32,
    content_height: u32,
) -> Scrollable<'static, Scroll, Theme, Renderer> {
    Scrollable::new(
        space()
            .width(Length::Fixed(content_width as f32))
            .height(Length::Fixed(content_height as f32)),
    )
    .id("scrollable")
    .width(Length::Fill)
    .height(200)
    .direction(Direction::Both {
        vertical: Scrollbar::default(),
        horizontal: Scrollbar::default(),
    })
    .on_scroll(Some)
}

#[test]
fn snap_to_end_with_smooth_behavior_preserves_other_axis_snappedness() {
    let mut simulator = Simulator::new(element_xy(2000, 3000));

    // Pin the horizontal scroll to the end (instantly)
    let mut operation = operation::scrollable::snap_to(
        ELEMENT_ID,
        RelativeOffset {
            x: Some(1.0),
            y: None,
        },
        operation::Animation::Instant,
    );

    simulator.operate(&mut operation);

    let mut instant = Instant::now();
    step_frames(&mut simulator, &mut instant, 2);

    // Snap the vertical scroll to the end smoothly: the horizontal offset
    // must keep its snappedness
    let mut operation = operation::scrollable::snap_to(
        ELEMENT_ID,
        RelativeOffset {
            x: None,
            y: Some(1.0),
        },
        operation::Animation::Smooth,
    );

    simulator.operate(&mut operation);
    step_frames(&mut simulator, &mut instant, 60);

    // Grow the content wider: the horizontally-snapped offset must follow
    // the new right edge (3000 − 1024), not freeze at the old one (976)
    simulator = simulator.rebuild(element_xy(3000, 3000));
    step_frames(&mut simulator, &mut instant, 2);

    let offsets: Vec<f32> = simulator
        .into_messages()
        .map(|scroll| scroll.viewport.absolute_offset().x)
        .collect();

    assert_eq!(
        offsets.last(),
        Some(&1976.0),
        "the horizontal snappedness must survive the vertical snap: {offsets:?}"
    );
}

#[test]
fn scrollbar_drag_unsnaps_only_the_dragged_axis() {
    let mut simulator = Simulator::new(element_xy(2000, 3000));

    // Pin the horizontal scroll to the end (instantly)
    let mut operation = operation::scrollable::snap_to(
        ELEMENT_ID,
        RelativeOffset {
            x: Some(1.0),
            y: None,
        },
        operation::Animation::Instant,
    );

    simulator.operate(&mut operation);

    let mut instant = Instant::now();
    step_frames(&mut simulator, &mut instant, 2);

    // Drag the vertical scrollbar (right edge, 10 px wide) partway down
    simulator.point_at(Point::new(1019.0, 60.0));
    let _ = simulator.simulate([Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    ))]);

    simulator.point_at(Point::new(1019.0, 150.0));
    let _ = simulator.simulate([Event::Mouse(mouse::Event::CursorMoved {
        position: Point::new(1019.0, 150.0),
    })]);

    let _ = simulator.simulate([Event::Mouse(mouse::Event::ButtonReleased(
        mouse::Button::Left,
    ))]);

    step_frames(&mut simulator, &mut instant, 2);

    // Grow the content wider
    simulator = simulator.rebuild(element_xy(3000, 3000));
    step_frames(&mut simulator, &mut instant, 2);

    let viewports: Vec<Scroll> = simulator.into_messages().collect();

    let (dragged, grown) = (
        viewports[viewports.len() - 2].viewport.absolute_offset(),
        viewports.last().unwrap().viewport.absolute_offset(),
    );

    // The drag must have registered, moving the vertical scroll
    assert!(
        grown.y > 1000.0,
        "the drag must move the vertical scroll: {viewports:?}"
    );

    // The drag must have materialized the vertical offset: growing the
    // content leaves it where the drag left it
    assert_eq!(
        dragged.y, grown.y,
        "the dragged axis must not keep its snappedness: {viewports:?}"
    );

    // ... while the horizontal snappedness follows the new right edge
    assert_eq!(
        grown.x, 1976.0,
        "the horizontal snappedness must survive the vertical drag: {viewports:?}"
    );
}

/// A `Scrollable` that fills the simulator's window, with content of the
/// given height.
fn fill_element(content_height: u32) -> Scrollable<'static, Scroll, Theme, Renderer> {
    Scrollable::new(space().height(content_height))
        .id("scrollable")
        .width(Length::Fill)
        .height(Length::Fill)
        .on_scroll(Some)
}

#[test]
fn notifications_report_the_wheel_source() {
    let mut simulator = Simulator::new(element(3000, 200));
    simulator.point_at(Point::new(500.0, 100.0));

    let _ = simulator.scroll(ScrollDelta::Lines { x: 0.0, y: -1.0 });

    let mut instant = Instant::now();
    step_frames(&mut simulator, &mut instant, 60);

    let sources: Vec<Source> = simulator
        .into_messages()
        .map(|scroll| scroll.source)
        .collect();

    assert!(
        !sources.is_empty(),
        "expected wheel notifications: {sources:?}"
    );
    assert!(
        sources.iter().all(|source| matches!(source, Source::Wheel)),
        "all wheel notifications must report `Wheel`: {sources:?}"
    );
}

#[test]
fn notifications_report_the_operation_source() {
    let mut simulator = Simulator::new(element(3000, 200));

    let mut instant = Instant::now();

    // A smooth scroll operation
    let mut operation = operation::scrollable::scroll_by(
        ELEMENT_ID,
        AbsoluteOffset { x: 0.0, y: 1000.0 },
        operation::Animation::Smooth,
    );

    simulator.operate(&mut operation);

    step_frames(&mut simulator, &mut instant, 60);

    // An immediate scroll operation
    let mut operation = operation::scrollable::scroll_to(
        ELEMENT_ID,
        AbsoluteOffset {
            x: None,
            y: Some(600.0),
        },
        operation::Animation::Instant,
    );

    simulator.operate(&mut operation);

    step_frames(&mut simulator, &mut instant, 2);

    // A smooth snap operation
    let mut operation = operation::scrollable::snap_to(
        ELEMENT_ID,
        RelativeOffset {
            x: None,
            y: Some(1.0),
        },
        operation::Animation::Smooth,
    );

    simulator.operate(&mut operation);
    step_frames(&mut simulator, &mut instant, 60);

    let sources: Vec<Source> = simulator
        .into_messages()
        .map(|scroll| scroll.source)
        .collect();

    assert!(
        !sources.is_empty(),
        "expected operation notifications: {sources:?}"
    );
    assert!(
        sources
            .iter()
            .all(|source| matches!(source, Source::Operation)),
        "all operation notifications must report `Operation`: {sources:?}"
    );
}

#[test]
fn notifications_report_the_scrollbar_source() {
    let mut simulator = Simulator::new(element_xy(2000, 3000));

    let mut instant = Instant::now();
    step_frames(&mut simulator, &mut instant, 2);

    // Drag the vertical scrollbar (right edge, 10 px wide) partway down
    simulator.point_at(Point::new(1019.0, 60.0));
    let _ = simulator.simulate([Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    ))]);

    simulator.point_at(Point::new(1019.0, 150.0));
    let _ = simulator.simulate([Event::Mouse(mouse::Event::CursorMoved {
        position: Point::new(1019.0, 150.0),
    })]);

    let _ = simulator.simulate([Event::Mouse(mouse::Event::ButtonReleased(
        mouse::Button::Left,
    ))]);

    let sources: Vec<Source> = simulator
        .into_messages()
        .map(|scroll| scroll.source)
        .collect();

    assert_eq!(
        sources,
        vec![Source::Content, Source::Scrollbar, Source::Scrollbar],
        "the drag must report `Scrollbar`: {sources:?}"
    );
}

#[test]
fn notifications_report_the_auto_scroll_source() {
    let element = Scrollable::new(space().height(Length::Fixed(3000.0)))
        .id("scrollable")
        .width(Length::Fill)
        .height(200)
        .auto_scroll(true)
        .on_scroll(Some);

    let mut simulator = Simulator::new(element);

    let mut instant = Instant::now();
    step_frames(&mut simulator, &mut instant, 2);

    // Press the middle button over the content, then move the cursor below
    // the scrollable to auto-scroll down
    simulator.point_at(Point::new(500.0, 100.0));
    let _ = simulator.simulate([Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Middle,
    ))]);

    simulator.point_at(Point::new(500.0, 400.0));
    let _ = simulator.simulate([Event::Mouse(mouse::Event::CursorMoved {
        position: Point::new(500.0, 400.0),
    })]);

    step_frames(&mut simulator, &mut instant, 2);

    let sources: Vec<Source> = simulator
        .into_messages()
        .map(|scroll| scroll.source)
        .collect();

    assert!(
        sources
            .iter()
            .any(|source| matches!(source, Source::AutoScroll)),
        "the auto-scroll must report `AutoScroll`: {sources:?}"
    );
}

#[test]
fn notifications_report_the_layout_source() {
    let mut simulator = Simulator::new(fill_element(3000));

    // The first notification reports the initial layout
    let mut instant = Instant::now();
    step_frames(&mut simulator, &mut instant, 1);

    let sources: Vec<Source> = simulator
        .drain()
        .into_iter()
        .map(|scroll| scroll.source)
        .collect();
    assert_eq!(
        sources,
        vec![Source::Content],
        "the initial notification must report `Content`: {sources:?}"
    );

    // Growing the content reports `Content`
    simulator = simulator.rebuild(fill_element(6000));
    step_frames(&mut simulator, &mut instant, 1);

    let sources: Vec<Source> = simulator
        .drain()
        .into_iter()
        .map(|scroll| scroll.source)
        .collect();
    assert_eq!(
        sources,
        vec![Source::Content],
        "the content growth must report `Content`: {sources:?}"
    );

    // Resizing the window reports `Resize`
    simulator = simulator.resize(Size::new(1024.0, 600.0));
    step_frames(&mut simulator, &mut instant, 1);

    let sources: Vec<Source> = simulator
        .drain()
        .into_iter()
        .map(|scroll| scroll.source)
        .collect();
    assert_eq!(
        sources,
        vec![Source::Resize],
        "the resize must report `Resize`: {sources:?}"
    );
}

#[test]
fn destination_is_snapped_during_animation() {
    let mut simulator = Simulator::new(element(3000, 200));

    // Smoothly snap to the end: the animation targets a relative offset,
    // so the Y axis is snapped while it is still in flight
    let mut operation = operation::scrollable::snap_to(
        ELEMENT_ID,
        RelativeOffset {
            x: None,
            y: Some(1.0),
        },
        operation::Animation::Smooth,
    );

    simulator.operate(&mut operation);

    let mut instant = Instant::now();
    step_frames(&mut simulator, &mut instant, 1);

    let scroll = simulator.drain().pop().unwrap();
    assert!(
        scroll.target.is_some(),
        "expected an in-flight animation: {scroll:?}"
    );
    assert!(
        scroll.destination().y.is_snapped(),
        "the Y axis must be snapped mid-flight, towards its relative target: {scroll:?}"
    );

    // Settled: the offset itself is relative now
    step_frames(&mut simulator, &mut instant, 60);

    let scroll = simulator.drain().pop().unwrap();
    assert!(
        scroll.target.is_none(),
        "expected a settled animation: {scroll:?}"
    );
    assert!(
        scroll.viewport.y.is_snapped(),
        "the Y offset must be relative once settled: {scroll:?}"
    );
}

#[test]
fn destination_is_unsnapped_for_absolute_targets() {
    let mut simulator = Simulator::new(element(3000, 200));
    simulator.point_at(Point::new(500.0, 100.0));

    // A wheel targets an absolute offset
    let _ = simulator.scroll(ScrollDelta::Lines { x: 0.0, y: -1.0 });

    let mut instant = Instant::now();
    step_frames(&mut simulator, &mut instant, 1);

    let scroll = simulator.drain().pop().unwrap();
    assert!(
        scroll.target.is_some(),
        "expected an in-flight animation: {scroll:?}"
    );
    assert!(
        !scroll.destination().y.is_snapped(),
        "a wheel targets an absolute offset: {scroll:?}"
    );

    step_frames(&mut simulator, &mut instant, 60);

    let scroll = simulator.drain().pop().unwrap();
    assert!(
        !scroll.viewport.y.is_snapped(),
        "the settled offset must be absolute: {scroll:?}"
    );
}
