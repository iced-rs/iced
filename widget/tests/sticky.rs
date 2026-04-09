//! Tests for the `sticky` widget.
use iced_widget::core::{
    Length::Fill, Never, Point, Rectangle, Settings, Size, Vector, layout, mouse, widget,
};
use iced_widget::scrollable::{AbsoluteOffset, Direction, Scrollbar};
use iced_widget::{Renderer, Theme, column, container, pick_list, row, scrollable, space, sticky};

type Element<Message = Never, Renderer = ()> =
    iced_widget::core::Element<'static, Message, Theme, Renderer>;

const VIEWPORT: Rectangle = Rectangle::new(Point::ORIGIN, Size::new(1024.0, 768.0));
const DEFAULT_LIMITS: layout::Limits = layout::Limits::new(Size::ZERO, VIEWPORT.size());

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
        sticky(container("Header").width(50).height(100)),
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
    element.as_widget_mut().operate(
        tree,
        layout::Layout::new(node),
        &VIEWPORT,
        &(),
        &mut scroll_to,
    );
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
            let node = overlays[0].as_overlay_mut().layout(&(), VIEWPORT.size(), iced_widget::core::Direction::default());

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
        .layout(&mut tree, &(), &DEFAULT_LIMITS, iced_widget::core::Direction::default());

    // The sticky contents are in view, so they are not displayed on an
    // overlay.
    assert_eq!(overlay_bounds(&mut element, &mut tree, &node), None);
}

#[test]
fn sticky_partially_out_of_view() {
    let mut element = vertical_view();
    let mut tree = build(&mut element);

    let node = element
        .as_widget_mut()
        .layout(&mut tree, &(), &DEFAULT_LIMITS, iced_widget::core::Direction::default());

    // Scroll down a little: the sticky contents are partially out of the
    // visible bounds, so they are displayed on an overlay, inside them.
    scroll(&mut element, &mut tree, &node, 0.0, 30.0);
    assert_eq!(
        overlay_bounds(&mut element, &mut tree, &node),
        Some(Rectangle::new(Point::ORIGIN, Size::new(1024.0, 50.0)))
    );
}

#[test]
fn sticky_out_of_view() {
    let mut element = vertical_view();
    let mut tree = build(&mut element);

    let node = element
        .as_widget_mut()
        .layout(&mut tree, &(), &DEFAULT_LIMITS, iced_widget::core::Direction::default());

    // Scroll down, so that the sticky contents go out of view: they are
    // displayed on an overlay, inside the visible bounds.
    scroll(&mut element, &mut tree, &node, 0.0, 100.0);
    assert_eq!(
        overlay_bounds(&mut element, &mut tree, &node),
        Some(Rectangle::new(Point::ORIGIN, Size::new(1024.0, 50.0)))
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
        .layout(&mut tree, &(), &DEFAULT_LIMITS, iced_widget::core::Direction::default());

    // Scroll a little: the sticky contents are out of the visible bounds on
    // the bottom edge, so they are displayed on an overlay, pinned to it.
    scroll(&mut element, &mut tree, &node, 0.0, 30.0);
    assert_eq!(
        overlay_bounds(&mut element, &mut tree, &node),
        Some(Rectangle::new(
            Point::new(0.0, 718.0),
            Size::new(1024.0, 50.0)
        ))
    );

    // Scroll past the sticky contents: they are displayed on an overlay,
    // pinned to the top edge.
    scroll(&mut element, &mut tree, &node, 0.0, 1060.0);
    assert_eq!(
        overlay_bounds(&mut element, &mut tree, &node),
        Some(Rectangle::new(Point::ORIGIN, Size::new(1024.0, 50.0)))
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
        .layout(&mut tree, &(), &DEFAULT_LIMITS, iced_widget::core::Direction::default());

    // Scroll down, so that the sticky contents go out of the visible bounds:
    // the overlay is clamped, and the 2000px-wide contents are displayed
    // with a width of 1024px.
    scroll(&mut element, &mut tree, &node, 0.0, 100.0);
    assert_eq!(
        overlay_bounds(&mut element, &mut tree, &node),
        Some(Rectangle::new(Point::ORIGIN, Size::new(1024.0, 50.0)))
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
        .layout(&mut tree, &(), &DEFAULT_LIMITS, iced_widget::core::Direction::default());

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
        .layout(&mut tree, &(), &DEFAULT_LIMITS, iced_widget::core::Direction::default());

    // Scroll to the right, so that the sticky contents go out of view: they
    // are displayed on an overlay, inside the visible bounds.
    scroll(&mut element, &mut tree, &node, 100.0, 0.0);
    assert_eq!(
        overlay_bounds(&mut element, &mut tree, &node),
        Some(Rectangle::new(Point::ORIGIN, Size::new(50.0, 100.0)))
    );
}

#[test]
fn sticky_pick_list_menu_opens_at_floating_position() -> Result<(), iced_test::Error> {
    use iced_test::simulator::{self, Simulator};

    /// The messages produced by the `pick_list`.
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Message {
        Opened,
        Selected(&'static str),
    }

    let pick_list_widget = pick_list(Some("One"), ["One", "Two", "Three"], |option| {
        option.to_string()
    })
    .placeholder("Select an option")
    .on_open(Message::Opened)
    .on_select(Message::Selected);

    let view: Element<Message, Renderer> = scrollable(column![
        space().height(300),
        sticky(pick_list_widget),
        space().height(1000),
    ])
    .width(Fill)
    .height(Fill)
    .id("scrollable")
    .into();

    let mut ui = Simulator::with_size(Settings::default(), VIEWPORT.size(), view);

    // Scroll down, so that the sticky contents go out of the visible bounds.
    ui.point_at(VIEWPORT.center());
    let _ = ui.scroll(mouse::ScrollDelta::Pixels {
        x: 0.0,
        y: -1_000.0,
    });

    // Click the pick_list at its floating position, at the top of the
    // visible bounds.
    let _ = ui.click("One")?;

    // The menu is displayed below the floating pick_list, inside the
    // visible bounds, not at its original, scrolled-out position.
    let option = ui.find("Three").expect("the menu option should be found");
    let visible = option
        .visible_bounds()
        .expect("the menu option should be visible");

    assert!(visible.y > 0.0 && visible.y < 100.0);

    // Move the cursor over the option, and select it from the floating
    // menu.
    ui.point_at(visible.center());
    let _ = ui.simulate(simulator::click());

    let messages: Vec<Message> = ui.into_messages().collect();
    assert_eq!(messages, vec![Message::Opened, Message::Selected("Three")]);

    Ok(())
}
