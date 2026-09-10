//! Tests for [`Component`]s with overlays.
use iced::widget::{Component, button, component, tooltip};
use iced::{Element, Event, Point};
use iced_test::Simulator;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Message {
    Pressed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum InnerEvent {
    Pressed,
}

/// A [`Component`] whose view is a [`tooltip`](tooltip) over a button.
struct WithTooltip;

impl<'a> Component<'a, Message> for WithTooltip {
    type State = ();
    type Event = InnerEvent;

    fn update(
        &self,
        _state: &mut Self::State,
        event: Self::Event,
        _renderer: &iced::Renderer,
    ) -> Option<Message> {
        (event == Self::Event::Pressed).then_some(Message::Pressed)
    }

    fn view(&self, _state: &Self::State) -> Element<'a, Self::Event> {
        tooltip(
            button("Press").on_press(Self::Event::Pressed),
            "Hover me",
            tooltip::Position::Top,
        )
        .delay(Duration::ZERO)
        .into()
    }
}

#[test]
fn component_with_overlay() {
    let component = component(WithTooltip);

    let mut simulator = Simulator::new(component);

    // Move the cursor over the button: the tooltip opens
    simulator.point_at(Point::new(10.0, 10.0));

    let _ = simulator.simulate([Event::Mouse(iced::mouse::Event::CursorMoved {
        position: Point::new(10.0, 10.0),
    })]);

    // The tooltip overlay is part of the user interface now
    let found = simulator
        .find("Hover me")
        .expect("tooltip overlay should be present");
    assert_ne!(found.bounds().size(), iced::Size::ZERO);

    // The tooltip does not block the button: clicking it still forwards the
    // component's message
    let _ = simulator.simulate([
        Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
        Event::Mouse(iced::mouse::Event::ButtonReleased(
            iced::mouse::Button::Left,
        )),
    ]);

    assert_eq!(
        simulator.into_messages().collect::<Vec<_>>(),
        [Message::Pressed]
    );
}
