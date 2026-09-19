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
use iced_widget::core::widget::Tree;
use iced_widget::core::window;
use iced_widget::core::{self, Event, Length, Point, Rectangle, Size, Theme};
use iced_widget::scrollable::{Scrollable, Viewport};
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
) -> Scrollable<'static, Viewport, Theme, Renderer> {
    Scrollable::new(space().height(content_height))
        .width(Length::Fill)
        .height(viewport_height)
        .on_scroll(Some)
}

/// Steps the given number of frames, keeping the real-time clock ahead of
/// the synthetic frame timeline.
///
/// The `Simulator` timestamps wheel events with the real-time clock, so if
/// the synthetic frame instants run ahead of it, a wheel event would be
/// observed "in the past" of the animation's timeline. Sleeping for each
/// frame guarantees the two timelines stay consistent.
fn step_frames(simulator: &mut Simulator<'_, Viewport>, instant: &mut Instant, frames: usize) {
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
        .map(|viewport| (viewport.absolute_offset().y, viewport.is_animating()))
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
        .map(|viewport| (viewport.absolute_offset().y, viewport.is_animating()))
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
        .map(|viewport| (viewport.absolute_offset().y, viewport.is_animating()))
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
        .map(|viewport| viewport.absolute_offset().y)
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
        .map(|viewport| viewport.absolute_offset().y)
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
            .map(|viewport| viewport.absolute_offset().y)
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
        .map(|viewport| viewport.absolute_offset().y)
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
        .map(|viewport| viewport.absolute_offset().y)
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
        .map(|viewport| (viewport.absolute_offset().y, viewport.is_animating()))
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
        .map(|viewport| viewport.absolute_offset().y)
        .collect();

    assert!(offsets.is_empty(), "unexpected notifications: {offsets:?}");
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
