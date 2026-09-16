//! Tests for the `sticky` widget.
use iced::advanced::layout;
use iced::advanced::widget;
use iced::widget::scrollable::{AbsoluteOffset, Direction, Scrollbar};
use iced::widget::{column, container, row, scrollable, space, sticky};
use iced::{Fill, Never, Point, Rectangle, Size, Theme, Vector};

type Element = iced::Element<'static, Never, Theme, ()>;

const DEFAULT_LIMITS: layout::Limits = layout::Limits::new(
    Size::ZERO,
    Size {
        width: 1024.0,
        height: 768.0,
    },
);

const VIEWPORT: Rectangle = Rectangle::new(Point::ORIGIN, Size::new(1024.0, 768.0));

fn vertical_view() -> Element {
    scrollable(column![
        sticky(container("Header").width(Fill).height(50)),
        space().height(1000),
    ])
    .width(Fill)
    .height(Fill)
    .id("scrollable")
    .into()
}

fn horizontal_view() -> Element {
    scrollable(row![
        sticky(container("Header").height(100).width(50)),
        space().width(3000),
    ])
    .direction(Direction::Horizontal(Scrollbar::default()))
    .width(Fill)
    .height(Fill)
    .id("scrollable")
    .into()
}

fn build(element: &mut Element) -> widget::Tree {
    let mut tree = widget::Tree::new(&*element);
    element.as_widget_mut().diff(&mut tree);

    tree
}

fn scroll(element: &mut Element, tree: &mut widget::Tree, node: &layout::Node, x: f32, y: f32) {
    let mut scroll_to = widget::operation::scrollable::scroll_to(
        "scrollable".into(),
        AbsoluteOffset {
            x: Some(x),
            y: Some(y),
        },
    );
    element
        .as_widget_mut()
        .operate(tree, layout::Layout::new(node), &(), &mut scroll_to);
}

fn overlay_bounds(
    element: &mut Element,
    tree: &mut widget::Tree,
    node: &layout::Node,
) -> Option<Rectangle> {
    let mut overlays = element.as_widget_mut().overlay(
        tree,
        layout::Layout::new(node),
        &(),
        &VIEWPORT,
        Vector::ZERO,
    );

    match overlays.len() {
        0 => None,
        1 => {
            let node = overlays[0]
                .as_overlay_mut()
                .layout(&(), Size::new(1024.0, 768.0));

            Some(node.bounds())
        }
        _ => panic!("expected at most one overlay, found {}", overlays.len()),
    }
}

#[test]
fn sticky_in_view() {
    let mut element = vertical_view();
    let mut tree = build(&mut element);

    let node = element
        .as_widget_mut()
        .layout(&mut tree, &(), &DEFAULT_LIMITS);

    // The sticky contents are in view, so they are not displayed on an
    // overlay.
    let overlays = element.as_widget_mut().overlay(
        &mut tree,
        layout::Layout::new(&node),
        &(),
        &VIEWPORT,
        Vector::ZERO,
    );

    assert!(overlays.is_empty());
}

#[test]
fn sticky_partially_out_of_view() {
    let mut element = vertical_view();
    let mut tree = build(&mut element);

    let node = element
        .as_widget_mut()
        .layout(&mut tree, &(), &DEFAULT_LIMITS);

    // Scroll down a little: the sticky contents are partially out of the
    // visible bounds, so they are displayed on an overlay, inside them.
    scroll(&mut element, &mut tree, &node, 0.0, 30.0);

    let bounds = overlay_bounds(&mut element, &mut tree, &node).unwrap();

    assert_eq!(
        bounds,
        Rectangle::new(Point::ORIGIN, Size::new(1024.0, 50.0))
    );
}

#[test]
fn sticky_out_of_view() {
    let mut element = vertical_view();
    let mut tree = build(&mut element);

    let node = element
        .as_widget_mut()
        .layout(&mut tree, &(), &DEFAULT_LIMITS);

    // Scroll down, so that the sticky contents go out of view.
    scroll(&mut element, &mut tree, &node, 0.0, 100.0);

    // The sticky contents are displayed on an overlay, inside the visible
    // bounds.
    let bounds = overlay_bounds(&mut element, &mut tree, &node).unwrap();

    assert_eq!(
        bounds,
        Rectangle::new(Point::ORIGIN, Size::new(1024.0, 50.0))
    );
}

#[test]
fn sticky_pinned_to_nearest_edge() {
    let mut element: Element = scrollable(column![
        space().height(1000),
        sticky(container("Header").width(Fill).height(50)),
        space().height(1000),
    ])
    .width(Fill)
    .height(Fill)
    .id("scrollable")
    .into();
    let mut tree = build(&mut element);

    let node = element
        .as_widget_mut()
        .layout(&mut tree, &(), &DEFAULT_LIMITS);

    // Scroll a little: the sticky contents are out of the visible bounds on
    // the bottom edge, so they are displayed on an overlay, pinned to it.
    scroll(&mut element, &mut tree, &node, 0.0, 30.0);

    {
        let bounds = overlay_bounds(&mut element, &mut tree, &node).unwrap();

        assert_eq!(
            bounds,
            Rectangle::new(Point::new(0.0, 718.0), Size::new(1024.0, 50.0))
        );
    }

    // Scroll past the sticky contents: they are displayed on an overlay,
    // pinned to the top edge.
    scroll(&mut element, &mut tree, &node, 0.0, 1060.0);

    let bounds = overlay_bounds(&mut element, &mut tree, &node).unwrap();

    assert_eq!(
        bounds,
        Rectangle::new(Point::ORIGIN, Size::new(1024.0, 50.0))
    );
}

#[test]
fn sticky_clamped_to_visible_bounds() {
    // The sticky contents are clamped so that their edges never go out of
    // the visible bounds, even when they are larger than the viewport.
    let mut element: Element = scrollable(
        column![
            sticky(container("Header").width(2000).height(50)),
            space().height(1000),
        ]
        .width(2000),
    )
    .direction(Direction::Both {
        vertical: Scrollbar::default(),
        horizontal: Scrollbar::default(),
    })
    .width(Fill)
    .height(Fill)
    .id("scrollable")
    .into();
    let mut tree = build(&mut element);

    let node = element
        .as_widget_mut()
        .layout(&mut tree, &(), &DEFAULT_LIMITS);

    // Scroll down, so that the sticky contents go out of the visible bounds.
    scroll(&mut element, &mut tree, &node, 0.0, 100.0);

    // The overlay is clamped to the visible bounds: the 2000px-wide contents
    // are displayed with a width of 1024px.
    let bounds = overlay_bounds(&mut element, &mut tree, &node).unwrap();

    assert_eq!(
        bounds,
        Rectangle::new(Point::ORIGIN, Size::new(1024.0, 50.0))
    );
}

#[test]
fn sticky_stays_attached_to_parent_bounds() {
    let mut element: Element = scrollable(column![
        space().height(300),
        container(sticky(container("Header").width(Fill).height(50))).center_y(500),
        space().height(1000),
    ])
    .width(Fill)
    .height(Fill)
    .id("scrollable")
    .into();
    let mut tree = build(&mut element);

    let node = element
        .as_widget_mut()
        .layout(&mut tree, &(), &DEFAULT_LIMITS);

    // Scroll down: the sticky contents are out of the visible bounds, but
    // their parent is still visible, so they are displayed on an overlay,
    // pinned to the top edge.
    scroll(&mut element, &mut tree, &node, 0.0, 600.0);
    assert_eq!(
        overlay_bounds(&mut element, &mut tree, &node),
        Some(Rectangle::new(Point::ORIGIN, Size::new(1024.0, 50.0)))
    );

    // Scroll down until the bottom edge of the contents reaches the bottom
    // edge of their parent: the contents simply stay put.
    scroll(&mut element, &mut tree, &node, 0.0, 750.0);
    assert_eq!(
        overlay_bounds(&mut element, &mut tree, &node),
        Some(Rectangle::new(Point::ORIGIN, Size::new(1024.0, 50.0)))
    );

    // Scroll down further: the contents are attached to the bottom edge of
    // their parent: they keep their original bounds, and are clipped to the
    // visible bounds instead.
    scroll(&mut element, &mut tree, &node, 0.0, 770.0);
    assert_eq!(
        overlay_bounds(&mut element, &mut tree, &node),
        Some(Rectangle::new(Point::ORIGIN, Size::new(1024.0, 30.0)))
    );

    // Scroll down until the parent has gone out of the visible bounds: the
    // contents are released and scroll with it.
    scroll(&mut element, &mut tree, &node, 0.0, 900.0);
    assert_eq!(overlay_bounds(&mut element, &mut tree, &node), None);
}

#[test]
fn sticky_out_of_view_horizontally() {
    let mut element = horizontal_view();
    let mut tree = build(&mut element);

    let node = element
        .as_widget_mut()
        .layout(&mut tree, &(), &DEFAULT_LIMITS);

    // Scroll to the right, so that the sticky contents go out of view.
    scroll(&mut element, &mut tree, &node, 100.0, 0.0);

    // The sticky contents are displayed on an overlay, inside the visible
    // bounds.
    let bounds = overlay_bounds(&mut element, &mut tree, &node).unwrap();

    assert_eq!(
        bounds,
        Rectangle::new(Point::ORIGIN, Size::new(50.0, 100.0))
    );
}
