//! Tests for the [`lazy`] widget.
//!
//! [`lazy`]: iced_widget::lazy
use iced_test::Simulator;
use iced_widget::core::{Element, Length};
use iced_widget::{lazy, row, text};

/// A row with a [`lazy`] child whose cached content fills the main axis,
/// followed by a static text.
///
/// The `lazy` child must report its `Fill` size hint on every frame, so the
/// row lays it out as a fluid item and the static text keeps its intrinsic
/// width. If the hint goes stale (i.e. the default `Fit` hint of a freshly
/// created [`Lazy`] value), the cached content is laid out as a static item
/// with the whole available width and the static text is collapsed to zero.
///
/// [`Lazy`]: iced_widget::Lazy
fn view(dependency: u8) -> Element<'static, (), iced_widget::Theme, iced_widget::Renderer> {
    row![
        lazy(dependency, |_| text("fill").width(Length::Fill)),
        text("anchor"),
    ]
    .into()
}

/// Asserts that the static text keeps a non-zero intrinsic width, which only
/// happens if the `Fill` hint of the `lazy` content was reported to the row.
fn assert_hints_respected(
    simulator: &mut Simulator<'_, (), iced_widget::Theme, iced_widget::Renderer>,
) {
    let fill = simulator.find("fill").unwrap().bounds().width;
    let anchor = simulator.find("anchor").unwrap().bounds().width;

    assert!(
        anchor > 0.0,
        "the static text should keep its intrinsic width, got {anchor}"
    );

    // The row is the root of the UI: the two children should share the whole
    // window width (1024, the default simulator size).
    assert!(
        (fill + anchor - 1024.0).abs() < 1.0,
        "the row should distribute the available width between the children: {fill} + {anchor}"
    );
}

#[test]
fn size_hint_is_respected_on_first_frame() {
    let mut simulator = Simulator::new(view(0));

    assert_hints_respected(&mut simulator);
}

#[test]
fn size_hint_is_respected_when_dependency_is_unchanged() {
    let simulator = Simulator::new(view(0));

    // Rebuild the UI with the same dependency: the cached element is reused,
    // but the `Lazy` widget value is recreated from scratch.
    let mut simulator = simulator.rebuild(view(0));

    assert_hints_respected(&mut simulator);
}

#[test]
fn size_hint_is_respected_when_dependency_changes() {
    let simulator = Simulator::new(view(0));

    // Rebuild the UI with a new dependency: the cached element is replaced.
    let mut simulator = simulator.rebuild(view(1));

    assert_hints_respected(&mut simulator);
}
