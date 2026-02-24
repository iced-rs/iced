use std::time::Duration;

use iced::widget::{button, center, column};
use iced::window::{Event, Id};
use iced::{Element, Size, Task, window};

pub fn main() -> iced::Result {
    iced::application(Example::default, Example::update, Example::view)
        .subscription(|_| window::events().map(Message::WinEvent))
        .executor::<tokio::runtime::Runtime>()
        .run()
}

#[derive(Default)]
struct Example { }

#[derive(Debug, Clone)]
enum Message {
    ResizeWindow(Size),
    DelayedResize(Size),
    WinEvent((Id, Event)),
}

impl Example {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ResizeWindow(size) => {
                        return window::latest().and_then(move |id| window::resize(id, size));
                    }
            Message::DelayedResize(size) => {
                        return Task::future(async {
                            tokio::time::sleep(Duration::from_secs(1)).await;
                        })
                        .then(move |_| window::latest().and_then(move |id| window::resize(id, size)));
                    }
            // Message::WinEvent((id, ev)) => println!("Event for {id:?}: {ev:?}"),
            Message::WinEvent((id, ev)) => (),
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let content = column![
            button("Resize to 100x100").on_press(Message::ResizeWindow(Size::new(100., 100.))),
            button("Resize to 200x200").on_press(Message::ResizeWindow(Size::new(200., 200.))),
            button("Resize to 100x100 after 1 second").on_press(Message::DelayedResize(Size::new(100., 100.))),
        ]
        .spacing(20);

        center(content).into()
    }
}
