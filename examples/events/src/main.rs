use iced::alignment;
use iced::event::{self, Event};
use iced::widget::{button, center, checkbox, text, Column};
use iced::window;
use iced::{Alignment, Command, Element, Length, Subscription};

pub fn main() -> iced::Result {
    iced::program("Events - Iced", Events::update, Events::view)
        .subscription(Events::subscription)
        .exit_on_close_request(false)
        .run()
}

#[derive(Debug, Default)]
struct Events {
    last: Vec<Event>,
    enabled: bool,
}

#[derive(Debug, Clone)]
enum Message {
    EventOccurred(Event),
    Toggled(bool),
    Exit,
}

impl Events {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::EventOccurred(event) if self.enabled => {
                self.last.push(event);

                if self.last.len() > 5 {
                    let _ = self.last.remove(0);
                }

                Command::none()
            }
            Message::EventOccurred(event) => {
                if let Event::Window(id, window::Event::CloseRequested) = event
                {
                    window::close(id)
                } else {
                    Command::none()
                }
            }
            Message::Toggled(enabled) => {
                self.enabled = enabled;

                Command::none()
            }
            Message::Exit => window::close(window::Id::MAIN),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::EventOccurred)
    }

    fn view(&self) -> Element<Message> {
        let events = Column::with_children(
            self.last
                .iter()
                .map(|event| text(format!("{event:?}")).size(40))
                .map(Element::from),
        );

        let toggle = checkbox("Listen to runtime events", self.enabled)
            .on_toggle(Message::Toggled);

        let exit = button(
            text("Exit")
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Center),
        )
        .width(100)
        .padding(10)
        .on_press(Message::Exit);

        let content = Column::new()
            .align_items(Alignment::Center)
            .spacing(20)
            .push(events)
            .push(toggle)
            .push(exit);

        center(content).into()
    }
}
