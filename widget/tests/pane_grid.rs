//! Smoke tests for [`PaneGrid`].
//!
//! [`PaneGrid`]: iced_widget::pane_grid::PaneGrid
use iced_test::Simulator;
use iced_widget::core::{self, Element, Size, mouse};
use iced_widget::{Renderer, Theme, pane_grid, text};

#[derive(Debug, Clone, Copy)]
enum Message {
    PaneClicked(pane_grid::Pane),
    PaneDragged(pane_grid::DragEvent),
}

/// A grid with two panes, split from top to bottom.
fn new_panes() -> (pane_grid::State<u8>, pane_grid::Pane, pane_grid::Pane) {
    let (mut panes, first) = pane_grid::State::new(1);

    let second = panes
        .split(pane_grid::Axis::Horizontal, first, 2)
        .expect("split the first pane")
        .0;

    (panes, first, second)
}

fn view(panes: &pane_grid::State<u8>) -> Element<'_, Message, Theme, Renderer> {
    pane_grid::PaneGrid::new(panes, |_, pane_state, _is_maximized| {
        pane_grid::Content::new(text(format!("Body of pane {pane_state}")))
            .title_bar(pane_grid::TitleBar::new(text(format!("Pane {pane_state}"))))
    })
    .on_click(Message::PaneClicked)
    .on_drag(Message::PaneDragged)
    .into()
}

#[test]
fn panes_are_laid_out_and_found() {
    let (panes, ..) = new_panes();
    let mut simulator = Simulator::new(view(&panes));

    let first_title = simulator.find("Pane 1").expect("find first pane title");
    let second_title = simulator.find("Pane 2").expect("find second pane title");
    let first_body = simulator
        .find("Body of pane 1")
        .expect("find first pane body");
    let second_body = simulator
        .find("Body of pane 2")
        .expect("find second pane body");

    for target in [&first_title, &second_title, &first_body, &second_body] {
        assert_ne!(
            target.bounds().size(),
            Size::ZERO,
            "pane parts should be laid out"
        );
    }

    // A horizontal split arranges the panes from top to bottom
    assert!(
        second_title.bounds().y > first_title.bounds().y,
        "second pane should be below the first"
    );
    assert_eq!(
        first_title.bounds().x,
        second_title.bounds().x,
        "both panes should start at the same position"
    );

    // The body of each pane is placed below its title bar
    assert!(
        first_body.bounds().y > first_title.bounds().y,
        "first pane body should be below its title bar"
    );
    assert!(
        second_body.bounds().y > second_title.bounds().y,
        "second pane body should be below its title bar"
    );
}

#[test]
fn clicking_a_pane_publishes_the_on_click_message() {
    let (panes, .., second) = new_panes();
    let mut simulator = Simulator::new(view(&panes));

    let _ = simulator
        .click("Body of pane 2")
        .expect("click the second pane body");

    let messages: Vec<Message> = simulator.drain().collect();

    match messages.as_slice() {
        [Message::PaneClicked(pane)] if *pane == second => {}
        _ => panic!("expected the second pane to be clicked, got {messages:?}"),
    }
}

#[test]
fn dragging_a_title_bar_publishes_picked_and_dropped_events() {
    let (panes, first, second) = new_panes();
    let size = Size::new(800.0, 600.0);
    let mut simulator = Simulator::with_size(core::Settings::default(), size, view(&panes));

    // Pick up the first pane by its title bar, away from the title text:
    // the title bar is a pick area except for its contents and controls
    let first_title = simulator.find("Pane 1").expect("find first pane title");
    let pick_position = core::Point::new(
        first_title.bounds().x + first_title.bounds().width + 50.0,
        first_title.bounds().y + first_title.bounds().height / 2.0,
    );
    simulator.point_at(pick_position);

    let _ = simulator.simulate([core::Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    ))]);

    let messages: Vec<Message> = simulator.drain().collect();
    assert_eq!(
        messages.len(),
        2,
        "pressing a title bar should publish a click and a pick"
    );
    match messages.as_slice() {
        [Message::PaneClicked(pane), ..] if *pane == first => {}
        _ => panic!("expected the first pane to be clicked, got {messages:?}"),
    }
    match messages[1] {
        Message::PaneDragged(pane_grid::DragEvent::Picked { pane }) => {
            assert_eq!(pane, first);
        }
        other => panic!("expected a picked event, got {other:?}"),
    }

    // Drop the pane over the center of the second one
    let second_title = simulator.find("Pane 2").expect("find second pane title");
    let drop_position = core::Point::new(
        size.width / 2.0,
        (second_title.bounds().y + size.height) / 2.0,
    );
    simulator.point_at(drop_position);

    let _ = simulator.simulate([
        core::Event::Mouse(mouse::Event::CursorMoved {
            position: drop_position,
        }),
        core::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
    ]);

    let messages: Vec<Message> = simulator.drain().collect();
    match messages.as_slice() {
        [
            Message::PaneDragged(pane_grid::DragEvent::Dropped {
                pane,
                target: pane_grid::Target::Pane(target, pane_grid::Region::Center),
            }),
        ] if *pane == first && *target == second => {}
        _ => panic!("expected the first pane dropped over the second, got {messages:?}"),
    }
}
