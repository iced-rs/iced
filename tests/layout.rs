//! Layout tests with the built-in widgets.
use iced::advanced::layout;
use iced::advanced::widget;
use iced::widget::{column, row, space};
use iced::{Element, Fill, FillPortion, Never, Pixels, Size, Theme};

const DEFAULT_LIMITS: layout::Limits = layout::Limits::new(
    Size::ZERO,
    Size {
        width: 1024.0,
        height: 768.0,
    },
);

#[test]
fn layout_fill_max() {
    assert_layout_eq(
        column![
            space().height(30).width(Fill.max(300)),
            space().width(Fill.max(400))
        ],
        node(
            (0, 0),
            (400, 30),
            [node((0, 0), (300, 30), []), node((0, 30), (400, 0), [])],
        ),
    );
}

#[test]
fn layout_fill_max_combined() {
    // The fixed element gets laid out first, then the bounded element, and finally
    // the unbounded fluid one.
    assert_layout_eq(
        row![
            space().height(10).width(Fill),
            space().height(20).width(Fill.max(300)),
            space().height(30).width(50),
        ],
        node(
            (0, 0),
            (1024, 30),
            [
                node((0, 0), (1024 - 300 - 50, 10), []),
                node((1024 - 300 - 50, 0), (300, 20), []),
                node((1024 - 50, 0), (50, 30), []),
            ],
        ),
    );
}

#[test]
fn layout_fill_max_nested() {
    // A nested row with bounded contents is laid out first over other fluid
    // elements, which get the leftovers.
    assert_layout_eq(
        row![
            space().height(10).width(Fill),
            row![
                space().height(20).width(Fill.max(300)),
                space().height(30).width(50),
            ]
        ],
        node(
            (0, 0),
            (1024, 30),
            [
                node((0, 0), (1024 - 300 - 50, 10), []),
                node(
                    (1024 - 300 - 50, 0),
                    (300 + 50, 30),
                    [node((0, 0), (300, 20), []), node((300, 0), (50, 30), [])],
                ),
            ],
        ),
    );
}

#[test]
fn layout_fill_max_nested_and_bounded() {
    // If the bounded constraints do not activate, the nested row behaves like
    // another fluid element.
    assert_layout_eq(
        row![
            space().height(10).width(Fill),
            row![
                space().height(20).width(Fill.max(300)),
                space().height(30).width(50),
            ]
        ]
        .width(500),
        node(
            (0, 0),
            (500, 30),
            [
                node((0, 0), (250, 10), []),
                node(
                    (250, 0),
                    (250, 30),
                    [
                        node((0, 0), (250 - 50, 20), []),
                        node((250 - 50, 0), (50, 30), []),
                    ],
                ),
            ],
        ),
    );
}

#[test]
fn layout_fill_min_bounded() {
    // Natural share is 300 each; min(400) is laid out first
    // and the remaining 200 flows to the first element.
    assert_layout_eq(
        row![
            space().height(10).width(Fill),
            space().height(10).width(Fill.min(400)),
        ]
        .width(600),
        node(
            (0, 0),
            (600, 10),
            [node((0, 0), (200, 10), []), node((200, 0), (400, 10), [])],
        ),
    );
}

#[test]
fn layout_fill_min_bounded_distribution() {
    // Like the previous case, but the remaining 200 is distributed by the
    // two fluid elements.
    assert_layout_eq(
        row![
            space().height(10).width(Fill),
            space().height(10).width(Fill.min(400)),
            space().height(10).width(Fill),
        ]
        .width(600),
        node(
            (0, 0),
            (600, 10),
            [
                node((0, 0), (100, 10), []),
                node((100, 0), (400, 10), []),
                node((500, 0), (100, 10), []),
            ],
        ),
    );
}

#[test]
fn layout_fill_min_nested() {
    // The share is 250, but the first element takes 260 and the nested row gets 240.
    // The 240 is shared as 120, but the second element takes 220 and the first fluid
    // gets the rest (20).
    assert_layout_eq(
        row![
            space().height(10).width(Fill.min(260)),
            row![
                space().height(30).width(Fill),
                space().height(20).width(Fill.min(220)),
            ]
        ]
        .width(500),
        node(
            (0, 0),
            (500, 30),
            [
                node((0, 0), (260, 10), []),
                node(
                    (260, 0),
                    (240, 30),
                    [node((0, 0), (20, 30), []), node((20, 0), (220, 20), [])],
                ),
            ],
        ),
    );
}

#[test]
fn layout_fill_min_max() {
    // The share is 250, but the first element takes 260 and the nested row gets 240.
    // The 240 is shared as 120, but the second element can only take 10 and the first
    // one gets the rest (230).
    assert_layout_eq(
        row![
            space().height(10).width(Fill.min(260)),
            row![
                space().height(20).width(Fill.min(220)),
                space().height(30).width(Fill.max(10)),
            ]
        ]
        .width(500),
        node(
            (0, 0),
            (500, 30),
            [
                node((0, 0), (260, 10), []),
                node(
                    (260, 0),
                    (240, 30),
                    [node((0, 0), (230, 20), []), node((230, 0), (10, 30), [])],
                ),
            ],
        ),
    );
}

#[test]
fn layout_fill_min_max_reverse() {
    assert_layout_eq(
        row![
            row![
                space().height(20).width(Fill.min(220)),
                space().height(30).width(Fill.max(10)),
            ],
            space().height(10).width(Fill.min(260)),
        ]
        .width(500),
        node(
            (0, 0),
            (500, 30),
            [
                node(
                    (0, 0),
                    (240, 30),
                    [node((0, 0), (230, 20), []), node((230, 0), (10, 30), [])],
                ),
                node((240, 0), (260, 10), []),
            ],
        ),
    );
}

#[test]
fn layout_fill_min_max_priority() {
    assert_layout_eq(
        row![
            space().width(Fill.min(400)).height(50),
            space().width(Fill.max(250)).height(50),
        ]
        .width(600),
        node(
            (0, 0),
            (600, 50),
            [node((0, 0), (400, 50), []), node((400, 0), (200, 50), [])],
        ),
    );
}

#[test]
fn layout_fill_min_max_distribution() {
    assert_layout_eq(
        row![
            space().width(Fill.min(400)).height(50),
            space().width(Fill.max(250)).height(50),
            space().width(Fill.max(200)).height(50),
        ]
        .width(900),
        node(
            (0, 0),
            (900, 50),
            [
                node((0, 0), (450, 50), []),
                node((450, 0), (250, 50), []),
                node((700, 0), (200, 50), []),
            ],
        ),
    );
}

#[test]
fn layout_fill_min_nested_competing() {
    assert_layout_eq(
        row![
            space().height(20).width(Fill.min(220)),
            row![space().height(10).width(Fill.min(260))],
        ]
        .width(500),
        node(
            (0, 0),
            (500, 20),
            [
                node((0, 0), (240, 20), []),
                node((240, 0), (260, 10), [node((0, 0), (260, 10), [])]),
            ],
        ),
    );
}

#[test]
fn layout_fill_has_priority_over_max() {
    assert_layout_eq(
        column![space().height(20).width(Fill.max(500))].width(Fill),
        node((0, 0), (1024, 20), [node((0, 0), (500, 20), [])]),
    );
}

#[test]
fn layout_fill_nested_min_max() {
    assert_layout_eq(
        column![
            space().height(Fill.min(100)),
            row![
                column![
                    space().height(Fill.min(200)),
                    space().height(Fill.min(300)),
                    space().width(Fill.max(300))
                ],
                space().width(Fill.min(200))
            ]
        ],
        node(
            (0, 0),
            (1024, 768),
            [
                node((0, 0), (0, 268), []),
                node(
                    (0, 268),
                    (1024, 500),
                    [
                        node(
                            (0, 0),
                            (300, 500),
                            [
                                node((0, 0), (0, 200), []),
                                node((0, 200), (0, 300), []),
                                node((0, 500), (300, 0), []),
                            ],
                        ),
                        node((300, 0), (724, 0), []),
                    ],
                ),
            ],
        ),
    );
}

#[test]
fn layout_fill_min_max_sidebar() {
    let widths = [1000, 800, 700, 500];

    for screen_width in widths {
        let view = {
            let sidebar = space::vertical().width(Fill.min(150).max(200));
            let content = space::horizontal().width(FillPortion(3));

            row![sidebar, content].width(screen_width)
        };

        let sidebar_width = (screen_width as f32 / 4.0).max(150.0).min(200.0);

        let layout = {
            let sidebar = node((0, 0), (sidebar_width, 768), []);
            let content = node(
                (sidebar_width, 0),
                (screen_width as f32 - sidebar_width, 0),
                [],
            );

            node((0, 0), (screen_width, 768), [sidebar, content])
        };

        assert_layout_eq(view, layout);
    }
}

#[test]
fn layout_fill_min_max_takes_leftover_space() {
    assert_layout_eq(
        row![
            space().width(Fill.min(400).max(500)).height(50),
            space().width(Fill.min(250).max(300)).height(50),
            space().width(Fill.max(300)).height(50),
        ],
        node(
            (0, 0),
            (1024, 50),
            [
                node((0, 0), (424, 50), []),
                node((424, 0), (300, 50), []),
                node((724, 0), (300, 50), []),
            ],
        ),
    );
}

fn assert_layout_eq<'a>(element: impl Into<Element<'a, Never, Theme, ()>>, expect: layout::Node) {
    let mut element = element.into();

    let mut tree = widget::Tree::new(&element);
    element.as_widget_mut().diff(&mut tree);

    let layout = element
        .as_widget_mut()
        .layout(&mut tree, &(), &DEFAULT_LIMITS);

    assert_eq!(layout, expect);
}

fn node(
    (x, y): (impl Into<Pixels>, impl Into<Pixels>),
    (width, height): (impl Into<Pixels>, impl Into<Pixels>),
    children: impl IntoIterator<Item = layout::Node>,
) -> layout::Node {
    let x = x.into().0;
    let y = y.into().0;
    let width = width.into().0;
    let height = height.into().0;

    layout::Node::with_children(Size { width, height }, children.into_iter().collect())
        .move_to((x, y))
}
