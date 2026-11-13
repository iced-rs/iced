//! Tests for the `checkbox` widget.
use iced_test::Simulator;
use iced_widget::checkbox;
use iced_widget::core::widget::{Operation, operation};
use iced_widget::core::{Event, Point, mouse};

/// Dragging over the label of a `checkbox` selects the text under the
/// cursor, not text offset by the size of the box and the spacing.
#[test]
fn label_selection_follows_cursor() {
    let mut simulator: Simulator<'_, ()> =
        Simulator::new(checkbox(false).label("Hello, world!").size(16).spacing(8));

    // Press at the left edge of the label (the size of the box plus the
    // spacing) and drag past its end
    simulator.point_at(Point::new(16.0 + 8.0, 10.0));
    let _ = simulator.simulate([Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    ))]);

    let position = Point::new(500.0, 10.0);
    simulator.point_at(position);
    let _ = simulator.simulate([Event::Mouse(mouse::Event::CursorMoved { position })]);
    let _ = simulator.simulate([Event::Mouse(mouse::Event::ButtonReleased(
        mouse::Button::Left,
    ))]);

    let mut copy = operation::text::copy();
    simulator.operate(&mut operation::black_box(&mut copy));

    let operation::Outcome::Some(selection) = copy.finish() else {
        panic!("copy operation should produce an outcome");
    };

    assert_eq!(selection, "Hello, world!");
}
