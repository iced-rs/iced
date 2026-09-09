//! Behavioral tests for the [`popover`] widget.
//!
//! These drive the [`Widget`](iced::advanced::widget::Widget) and
//! [`Overlay`](iced::advanced::Overlay) traits directly (using the null
//! renderer `()`) to verify that the popover always displays its overlay (the
//! application removes it from the view when it is "closed"), that the base is
//! a plain element, and that the popover notifies the application through its
//! `on_close` handler when the user clicks outside of its bounds.
use iced::advanced::layout::{self, Limits};
use iced::advanced::mouse::{self, Button, Cursor};
use iced::advanced::shell;
use iced::advanced::widget::Tree;
use iced::advanced::Layout;
use iced::widget::{popover, space};
use iced::window::Headless;
use iced::{Element, Event, Point, Rectangle, Size, Theme, Vector};

type Message = u8;

/// The message produced by the popover when a close is requested.
const ON_CLOSE: Message = 0x42;

/// The size of the test viewport.
const VIEWPORT: Size = Size::new(1000.0, 1000.0);

/// A left-button press at the given position.
fn press(position: Point) -> (Event, Cursor) {
    (
        Event::Mouse(mouse::Event::ButtonPressed(Button::Left)),
        Cursor::Available(position),
    )
}

/// A popover with a 50x50 base and an 80x80 popup, with an `on_close` handler.
fn new_popover() -> Element<'static, Message, Theme, ()> {
    popover(
        space().width(50).height(50),
        space().width(80).height(80),
        popover::Position::Bottom,
    )
    .on_close(ON_CLOSE)
    .into()
}

/// Builds the state tree and computes the (initial) layout for the element.
fn setup(
    element: &mut Element<'static, Message, Theme, ()>,
) -> (Tree, layout::Node) {
    let mut tree = Tree::new(&mut *element);
    element.as_widget_mut().diff(&mut tree);

    let node = element
        .as_widget_mut()
        .layout(&mut tree, &(), &Limits::new(Size::ZERO, VIEWPORT));

    (tree, node)
}

/// Returns whether the widget currently produces an overlay.
fn has_overlay(
    element: &mut Element<'static, Message, Theme, ()>,
    tree: &mut Tree,
    layout: Layout<'_>,
) -> bool {
    element
        .as_widget_mut()
        .overlay(tree, layout, &(), &Rectangle::with_size(VIEWPORT), Vector::ZERO)
        .is_some()
}

/// Drives a single event through the (open) popover overlay and returns the
/// bus of published messages.
fn drive_overlay(
    element: &mut Element<'static, Message, Theme, ()>,
    tree: &mut Tree,
    layout: Layout<'_>,
    event: &Event,
    cursor: Cursor,
) -> shell::Bus<Message> {
    let viewport = Rectangle::with_size(VIEWPORT);

    let mut overlay = element
        .as_widget_mut()
        .overlay(tree, layout, &(), &viewport, Vector::ZERO)
        .expect("popover overlay should exist");

    let overlay_node = overlay.as_overlay_mut().layout(&(), VIEWPORT);
    let overlay_layout = Layout::new(&overlay_node);

    let mut bus = shell::Bus::new();
    let mut shell = shell::Shell::new(&Headless, shell::Waker::noop(), &mut bus);

    overlay.as_overlay_mut().update(event, overlay_layout, cursor, &(), &mut shell);

    bus
}

/// Collects the messages published on the bus.
fn messages(bus: shell::Bus<Message>) -> Vec<Message> {
    bus.into_iter().collect()
}

#[test]
fn popover_layout_is_plain_base() {
    let mut element = new_popover();
    let (_, node) = setup(&mut element);

    let bounds = node.bounds();

    // The base is a plain 50x50 element: the popover does not wrap it in
    // padding or any other styling.
    assert_eq!(
        bounds.size(),
        Size::new(50.0, 50.0),
        "popover layout {bounds:?} should be the plain base, with no padding"
    );
}

#[test]
fn overlay_is_always_present() {
    let mut element = new_popover();
    let (mut tree, node) = setup(&mut element);
    let layout = Layout::new(&node);

    assert!(
        has_overlay(&mut element, &mut tree, layout),
        "popover should always show its overlay"
    );
}

#[test]
fn click_outside_requests_close() {
    let mut element = new_popover();
    let (mut tree, node) = setup(&mut element);
    let layout = Layout::new(&node);

    // Press somewhere far away: outside both the base and the popover.
    let (event, cursor) = press(Point::new(900.0, 900.0));
    let bus = drive_overlay(&mut element, &mut tree, layout, &event, cursor);

    assert_eq!(
        messages(bus),
        vec![ON_CLOSE],
        "clicking outside should request a close"
    );
}

#[test]
fn click_on_base_does_not_request_close() {
    let mut element = new_popover();
    let (mut tree, node) = setup(&mut element);
    let layout = Layout::new(&node);

    // The base occupies (0, 0) .. (50, 50). Pressing it must not request a
    // close: the base is responsible for its own behavior.
    let (event, cursor) = press(Point::new(25.0, 25.0));
    let bus = drive_overlay(&mut element, &mut tree, layout, &event, cursor);

    assert!(
        messages(bus).is_empty(),
        "clicking the base should not request a close"
    );
}

#[test]
fn click_inside_does_not_request_close() {
    let mut element = new_popover();
    let (mut tree, node) = setup(&mut element);
    let layout = Layout::new(&node);

    // The popover appears below the base (Position::Bottom) with its default
    // padding, so it occupies roughly (0, 50) .. (90, 140). Click a point that
    // lies inside the popover.
    let (event, cursor) = press(Point::new(40.0, 95.0));
    let bus = drive_overlay(&mut element, &mut tree, layout, &event, cursor);

    assert!(
        messages(bus).is_empty(),
        "clicking inside the popover should not request a close"
    );
}

#[test]
fn no_on_close_publishes_nothing() {
    // A popover without an `on_close` handler simply does not publish anything
    // when clicked outside.
    let mut element: Element<'static, Message, Theme, ()> = popover(
        space().width(50).height(50),
        space().width(80).height(80),
        popover::Position::Bottom,
    )
    .into();

    let (mut tree, node) = setup(&mut element);
    let layout = Layout::new(&node);

    let (event, cursor) = press(Point::new(900.0, 900.0));
    let bus = drive_overlay(&mut element, &mut tree, layout, &event, cursor);

    assert!(
        messages(bus).is_empty(),
        "a popover without an on_close handler should publish nothing"
    );
}
