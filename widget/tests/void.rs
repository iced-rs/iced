//! Tests for filtering "void" (i.e. `None`) children out of containers.
use iced_widget::button::Button;
use iced_widget::core::widget;
use iced_widget::core::{Never, Widget};
use iced_widget::{Theme, button, column, row, space};

fn build(element: &mut impl Widget<Never, Theme, ()>) -> widget::Tree {
    let mut tree = widget::Tree::new(&*element);
    element.diff(&mut tree);

    tree
}

/// `column!["Hello", Some(button)]` keeps both children.
#[test]
fn column_macro_accepts_mixed_plain_and_optional_widgets() {
    let maybe: Option<Button<'static, Never, &'static str, Theme>> = Some(button("Click"));

    let mut col = column!["Hello", maybe];
    let tree = build(&mut col);

    // Both `"Hello"` and the button survive.
    assert_eq!(tree.children.len(), 2);
}

/// A `None` child is filtered out of the column entirely: it takes no
/// layout slot and contributes no spacing.
#[test]
fn column_macro_filters_none_children() {
    let maybe: Option<Button<'static, Never, &'static str, Theme>> = None;

    let mut col = column!["Hello", maybe];
    let tree = build(&mut col);

    // Only `"Hello"` survives.
    assert_eq!(tree.children.len(), 1);
}

/// Same behaviour for `row!`.
#[test]
fn row_macro_filters_none_children() {
    let maybe: Option<space::Space> = None;

    let mut row = row!["Hello", space(), maybe];
    let tree = build(&mut row);

    // Only `"Hello"` and the plain `space()` survive.
    assert_eq!(tree.children.len(), 2);
}
