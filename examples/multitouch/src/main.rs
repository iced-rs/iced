//! This example shows how to use touch events in `Canvas` to draw
//! a circle around each fingertip. This only works on touch-enabled
//! computers like Microsoft Surface.
use iced::widget::canvas::event;
use iced::widget::canvas::{self, Canvas, Cursor, Geometry, Path, Stroke};
use iced::{
    executor, touch, window, Application, Color, Command, Element, Length,
    Point, Rectangle, Settings, Subscription, Theme,
};

use std::collections::HashMap;

pub fn main() -> iced::Result {
    env_logger::builder().format_timestamp(None).init();

    Multitouch::run(Settings {
        antialiasing: true,
        window: window::Settings {
            position: window::Position::Centered,
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

struct Multitouch {
    state: State,
}

#[derive(Debug)]
struct State {
    cache: canvas::Cache,
    fingers: HashMap<touch::Finger, Point>,
}

impl State {
    fn new() -> Self {
        Self {
            cache: canvas::Cache::new(),
            fingers: HashMap::new(),
        }
    }
}

#[derive(Debug)]
enum Message {
    FingerPressed { id: touch::Finger, position: Point },
    FingerLifted { id: touch::Finger },
}

impl Application for Multitouch {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Multitouch {
                state: State::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Multitouch - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::FingerPressed { id, position } => {
                self.state.fingers.insert(id, position.clone());
                self.state.cache.clear();
            }
            Message::FingerLifted { id } => {
                self.state.fingers.remove(&id);
                self.state.cache.clear();
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    fn view(&self) -> Element<Message> {
        Canvas::new(&self.state)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl<'a> canvas::Program<Message> for State {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: event::Event,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> (event::Status, Option<Message>) {
        match event {
            event::Event::Touch(touch_event) => match touch_event {
                touch::Event::FingerPressed { id, position }
                | touch::Event::FingerMoved { id, position } => (
                    event::Status::Captured,
                    Some(Message::FingerPressed { id, position }),
                ),
                touch::Event::FingerLifted { id, .. }
                | touch::Event::FingerLost { id, .. } => (
                    event::Status::Captured,
                    Some(Message::FingerLifted { id }),
                ),
            },
            _ => (event::Status::Ignored, None),
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let fingerweb = self.cache.draw(bounds.size(), |frame| {
            for finger in &self.fingers {
                dbg!(&finger);

                let circle = Path::new(|p| p.circle(*finger.1, 50.0));

                frame.stroke(
                    &circle,
                    Stroke {
                        color: Color::BLACK,
                        width: 3.0,
                        ..Stroke::default()
                    },
                );
            }
        });

        vec![fingerweb]
    }
}
