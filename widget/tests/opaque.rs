//! Behavioral tests for the `Opaque` widget.
//!
//! These drive the `Widget` trait directly (using the null renderer `()`) to
//! verify that an `Opaque` captures mouse button presses by default, and that
//! it captures scroll wheel events only when its `capture_scroll` flag
//! is enabled and the cursor is over its bounds.
use iced_widget::core::event;
use iced_widget::core::layout::{self, Layout, Limits};
use iced_widget::core::mouse::{self, Button, Cursor};
use iced_widget::core::renderer;
use iced_widget::core::shell;
use iced_widget::core::widget::{self, Tree, Widget};
use iced_widget::core::window::Headless;
use iced_widget::core::{Event, Length, Point, Rectangle, Size, Theme};
use iced_widget::{Opaque, Space, opaque, space};

type Message = u8;

/// The message published by the [`WheelSpy`] when it receives a scroll wheel
/// event.
const WHEEL_SEEN: Message = 0x42;

/// The size of the test viewport.
const VIEWPORT: Size = Size::new(1000.0, 1000.0);

/// A left-button press at the given position.
fn press(position: Point) -> (Event, Cursor) {
    (
        Event::Mouse(mouse::Event::ButtonPressed(Button::Left)),
        Cursor::Available(position),
    )
}

/// A scroll wheel event at the given position.
fn scroll(position: Point) -> (Event, Cursor) {
    (
        Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
        }),
        Cursor::Available(position),
    )
}

/// An `Opaque` wrapping a 50x50 `space`, optionally capturing the scroll wheel.
fn new_opaque(capture_scroll: bool) -> Opaque<Space> {
    let space = space().width(50).height(50);

    if capture_scroll {
        opaque(space).capture_scroll(true)
    } else {
        opaque(space)
    }
}

/// Builds the state tree and computes the (initial) layout for the widget.
fn setup<W>(widget: &mut W) -> (Tree, Layout)
where
    W: Widget<Message, Theme, ()>,
{
    let mut tree = Tree::new(widget);

    widget.diff(&mut tree);
    widget.layout(&mut tree, &(), &Limits::new(Size::ZERO, VIEWPORT));

    let layout = Layout::new(tree.size);

    (tree, layout)
}

/// Drives a single event through the widget and returns the resulting event
/// status and the bus of published messages.
fn drive<W>(
    widget: &mut W,
    tree: &mut Tree,
    layout: Layout,
    event: &Event,
    cursor: Cursor,
) -> (event::Status, shell::Bus<Message>)
where
    W: Widget<Message, Theme, ()>,
{
    let viewport = Rectangle::with_size(VIEWPORT);

    let mut bus = shell::Bus::new();
    let mut shell = shell::Shell::new(&Headless, shell::Waker::noop(), &mut bus);

    widget.update(tree, event, layout, cursor, &(), &mut shell, &viewport);

    (shell.event_status(), bus)
}

/// Collects the messages published on the bus.
fn messages(bus: shell::Bus<Message>) -> Vec<Message> {
    bus.into_iter().collect()
}

/// A 50x50 widget that publishes [`WHEEL_SEEN`] when it receives a scroll
/// wheel event.
struct WheelSpy;

impl widget::Meta for WheelSpy {}

impl Widget<Message, Theme, ()> for WheelSpy {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(50.0), Length::Fixed(50.0))
    }

    fn layout(&mut self, tree: &mut Tree, _renderer: &(), limits: &layout::Limits) {
        tree.size = layout::atomic(limits, 50, 50);
    }

    fn draw(
        &self,
        _tree: &Tree,
        _renderer: &mut (),
        _theme: &Theme,
        _style: &renderer::Style,
        _layout: Layout,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
    }

    fn update(
        &mut self,
        _tree: &mut Tree,
        event: &Event,
        _layout: Layout,
        _cursor: mouse::Cursor,
        _renderer: &(),
        shell: &mut shell::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        if matches!(event, Event::Mouse(mouse::Event::WheelScrolled { .. })) {
            shell.publish(WHEEL_SEEN);
        }
    }
}

#[test]
fn mouse_press_is_captured_by_default() {
    let mut element = new_opaque(false);
    let (mut tree, layout) = setup(&mut element);

    // The 50x50 content occupies (0, 0) .. (50, 50); press inside it.
    let (event, cursor) = press(Point::new(25.0, 25.0));
    let (status, _) = drive(&mut element, &mut tree, layout, &event, cursor);

    assert_eq!(
        status,
        event::Status::Captured,
        "an opaque layer should capture mouse button presses by default"
    );
}

#[test]
fn wheel_scroll_is_not_captured_by_default() {
    let mut element = new_opaque(false);
    let (mut tree, layout) = setup(&mut element);

    let (event, cursor) = scroll(Point::new(25.0, 25.0));
    let (status, _) = drive(&mut element, &mut tree, layout, &event, cursor);

    assert_eq!(
        status,
        event::Status::Ignored,
        "scroll wheel events should not be captured unless the flag is enabled"
    );
}

#[test]
fn wheel_scroll_is_captured_when_flag_enabled() {
    let mut element = new_opaque(true);
    let (mut tree, layout) = setup(&mut element);

    let (event, cursor) = scroll(Point::new(25.0, 25.0));
    let (status, _) = drive(&mut element, &mut tree, layout, &event, cursor);

    assert_eq!(
        status,
        event::Status::Captured,
        "scroll wheel events should be captured when the flag is enabled and hovered"
    );
}

#[test]
fn wheel_scroll_is_not_captured_when_not_hovered() {
    let mut element = new_opaque(true);
    let (mut tree, layout) = setup(&mut element);

    // Scroll somewhere far away: outside the opaque bounds.
    let (event, cursor) = scroll(Point::new(900.0, 900.0));
    let (status, _) = drive(&mut element, &mut tree, layout, &event, cursor);

    assert_eq!(
        status,
        event::Status::Ignored,
        "scroll wheel events should only be captured when the cursor is hovered"
    );
}

#[test]
fn mouse_press_is_still_captured_when_flag_enabled() {
    let mut element = new_opaque(true);
    let (mut tree, layout) = setup(&mut element);

    // The flag must not interfere with the default mouse press capture.
    let (event, cursor) = press(Point::new(25.0, 25.0));
    let (status, _) = drive(&mut element, &mut tree, layout, &event, cursor);

    assert_eq!(
        status,
        event::Status::Captured,
        "mouse button presses should still be captured when the flag is enabled"
    );
}

#[test]
fn wrapped_content_still_receives_wheel_scroll() {
    // The flag must not swallow events from the wrapped content: the spy
    // still receives the scroll, even though the event is then captured.
    let mut element = opaque(WheelSpy).capture_scroll(true);
    let (mut tree, layout) = setup(&mut element);

    let (event, cursor) = scroll(Point::new(25.0, 25.0));
    let (status, bus) = drive(&mut element, &mut tree, layout, &event, cursor);

    assert_eq!(
        messages(bus),
        vec![WHEEL_SEEN],
        "the wrapped content should still receive the scroll wheel event"
    );

    assert_eq!(
        status,
        event::Status::Captured,
        "the event should be captured after being delegated to the content"
    );
}
