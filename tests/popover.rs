//! Behavioral tests for the [`popover`] widget.
//!
//! These drive the [`Widget`](iced::advanced::widget::Widget) and
//! [`Overlay`](iced::advanced::Overlay) traits directly (using the null
//! renderer `()`) to verify the popover opens when its base is released (like
//! a button) and closes when the user clicks outside of its bounds.
use iced::advanced::layout::{self, Limits};
use iced::advanced::mouse::{self, Button, Cursor};
use iced::advanced::shell;
use iced::advanced::widget::Tree;
use iced::advanced::Layout;
use iced::widget::{popover, space};
use iced::window::Headless;
use iced::{Element, Event, Point, Rectangle, Size, Theme, Vector};

type Message = u8;

/// The size of the test viewport.
const VIEWPORT: Size = Size::new(1000.0, 1000.0);

/// A left-button press at the given position.
fn press(position: Point) -> (Event, Cursor) {
    (
        Event::Mouse(mouse::Event::ButtonPressed(Button::Left)),
        Cursor::Available(position),
    )
}

/// A left-button release at the given position.
fn release(position: Point) -> (Event, Cursor) {
    (
        Event::Mouse(mouse::Event::ButtonReleased(Button::Left)),
        Cursor::Available(position),
    )
}

/// A popover with a 50x50 base and an 80x80 popup.
fn new_popover() -> Element<'static, Message, Theme, ()> {
    popover(
        space().width(50).height(50),
        space().width(80).height(80),
        popover::Position::Bottom,
    )
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

/// Drives a single event through the widget (the base layer).
fn update_widget(
    element: &mut Element<'static, Message, Theme, ()>,
    tree: &mut Tree,
    layout: Layout<'_>,
    event: &Event,
    cursor: Cursor,
) {
    let mut messages = shell::Bus::new();
    let mut shell = shell::Shell::new(&Headless, shell::Waker::noop(), &mut messages);

    element
        .as_widget_mut()
        .update(tree, event, layout, cursor, &(), &mut shell, &Rectangle::with_size(VIEWPORT));
}

/// Drives a single event through the (open) popover overlay.
fn update_overlay(
    element: &mut Element<'static, Message, Theme, ()>,
    tree: &mut Tree,
    layout: Layout<'_>,
    event: &Event,
    cursor: Cursor,
) {
    let viewport = Rectangle::with_size(VIEWPORT);

    let mut overlay = element
        .as_widget_mut()
        .overlay(tree, layout, &(), &viewport, Vector::ZERO)
        .expect("popover overlay should exist");

    let overlay_node = overlay.as_overlay_mut().layout(&(), VIEWPORT);
    let overlay_layout = Layout::new(&overlay_node);

    let mut messages = shell::Bus::new();
    let mut shell = shell::Shell::new(&Headless, shell::Waker::noop(), &mut messages);

    overlay.as_overlay_mut().update(event, overlay_layout, cursor, &(), &mut shell);
}

#[test]
fn popover_starts_closed() {
    let mut element = new_popover();
    let (mut tree, node) = setup(&mut element);
    let layout = Layout::new(&node);

    assert!(!has_overlay(&mut element, &mut tree, layout), "popover should start closed");
}

#[test]
fn popover_opens_when_base_is_clicked() {
    let mut element = new_popover();
    let (mut tree, node) = setup(&mut element);
    let layout = Layout::new(&node);

    // The base occupies (0, 0) .. (70, 60) (50x50 content + button padding).
    // Press and release its center: the popover opens on release, like a
    // button.
    let (event, cursor) = press(Point::new(25.0, 25.0));
    update_widget(&mut element, &mut tree, layout, &event, cursor);
    let (event, cursor) = release(Point::new(25.0, 25.0));
    update_widget(&mut element, &mut tree, layout, &event, cursor);

    assert!(
        has_overlay(&mut element, &mut tree, layout),
        "popover should be open after clicking the base"
    );
}

#[test]
fn popover_opens_on_release_not_press() {
    let mut element = new_popover();
    let (mut tree, node) = setup(&mut element);
    let layout = Layout::new(&node);

    // Pressing the base alone must not open the popover.
    let (event, cursor) = press(Point::new(25.0, 25.0));
    update_widget(&mut element, &mut tree, layout, &event, cursor);

    assert!(
        !has_overlay(&mut element, &mut tree, layout),
        "popover should not open on press alone"
    );

    // Releasing over the base opens it.
    let (event, cursor) = release(Point::new(25.0, 25.0));
    update_widget(&mut element, &mut tree, layout, &event, cursor);

    assert!(
        has_overlay(&mut element, &mut tree, layout),
        "popover should open on release over the base"
    );
}

#[test]
fn popover_does_not_open_when_released_outside_base() {
    let mut element = new_popover();
    let (mut tree, node) = setup(&mut element);
    let layout = Layout::new(&node);

    // Press the base, then release somewhere else (outside the base), like a
    // button that is pressed but released elsewhere.
    let (event, cursor) = press(Point::new(25.0, 25.0));
    update_widget(&mut element, &mut tree, layout, &event, cursor);
    let (event, cursor) = release(Point::new(900.0, 900.0));
    update_widget(&mut element, &mut tree, layout, &event, cursor);

    assert!(
        !has_overlay(&mut element, &mut tree, layout),
        "popover should not open when released outside the base"
    );
}

#[test]
fn popover_closes_when_clicking_outside() {
    let mut element = new_popover();
    let (mut tree, node) = setup(&mut element);
    let layout = Layout::new(&node);

    // Open the popover by clicking the base (press + release).
    let (event, cursor) = press(Point::new(25.0, 25.0));
    update_widget(&mut element, &mut tree, layout, &event, cursor);
    let (event, cursor) = release(Point::new(25.0, 25.0));
    update_widget(&mut element, &mut tree, layout, &event, cursor);
    assert!(has_overlay(&mut element, &mut tree, layout), "popover should be open");

    // Press somewhere far away: outside both the base and the popover.
    let (event, cursor) = press(Point::new(900.0, 900.0));
    update_overlay(&mut element, &mut tree, layout, &event, cursor);

    assert!(
        !has_overlay(&mut element, &mut tree, layout),
        "popover should close after clicking outside its bounds"
    );
}

#[test]
fn popover_stays_open_when_clicking_inside() {
    let mut element = new_popover();
    let (mut tree, node) = setup(&mut element);
    let layout = Layout::new(&node);

    // Open the popover by clicking the base (press + release).
    let (event, cursor) = press(Point::new(25.0, 25.0));
    update_widget(&mut element, &mut tree, layout, &event, cursor);
    let (event, cursor) = release(Point::new(25.0, 25.0));
    update_widget(&mut element, &mut tree, layout, &event, cursor);

    // The popover is 80x80 and appears below the base (Position::Bottom), so
    // it occupies roughly (0, 50 + gap + padding) .. (80, ...). Click a point
    // that lies inside the popover.
    let (event, cursor) = press(Point::new(25.0, 100.0));
    update_overlay(&mut element, &mut tree, layout, &event, cursor);
    let (event, cursor) = release(Point::new(25.0, 100.0));
    update_overlay(&mut element, &mut tree, layout, &event, cursor);

    assert!(
        has_overlay(&mut element, &mut tree, layout),
        "popover should stay open when clicking inside it"
    );
}

#[test]
fn base_is_padded_like_a_button() {
    let mut element = new_popover();
    let (_, node) = setup(&mut element);

    // The raw content is a 50x50 space, but the base is rendered like a
    // button, so it is wrapped in the default button padding and its bounds
    // are larger than the content.
    let bounds = node.bounds();

    assert!(
        bounds.width > 50.0 && bounds.height > 50.0,
        "base {bounds:?} should include the default button padding"
    );
}

#[test]
fn base_reports_pointer_when_hovered() {
    let mut element = new_popover();
    let (tree, node) = setup(&mut element);
    let layout = Layout::new(&node);
    let viewport = Rectangle::with_size(VIEWPORT);

    // The base behaves like a button: hovering it reports a `Pointer`
    // interaction, like a [`Button`] would.
    //
    // [`Button`]: iced::widget::Button
    let hovered = element.as_widget().mouse_interaction(
        &tree,
        layout,
        Cursor::Available(Point::new(25.0, 25.0)),
        &viewport,
        &(),
    );

    assert_eq!(
        hovered,
        mouse::Interaction::Pointer,
        "base should report a pointer interaction when hovered"
    );

    // Hovering away from the base reports no interaction.
    let away = element.as_widget().mouse_interaction(
        &tree,
        layout,
        Cursor::Available(Point::new(500.0, 500.0)),
        &viewport,
        &(),
    );

    assert_eq!(
        away,
        mouse::Interaction::default(),
        "base should report no interaction when hovered away"
    );
}
