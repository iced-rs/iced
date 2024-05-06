use iced::alignment;
use iced::event::{self, Event};
use iced::widget::{
    button, center, checkbox, container, list, scrollable, text, Column,
    Scrollable,
};
use iced::window;
use iced::{Alignment, Command, Element, Font, Length, Subscription};

pub fn main() -> iced::Result {
    iced::program("Events - Iced", Events::update, Events::view)
        .subscription(Events::subscription)
        .exit_on_close_request(false)
        .run()
}

#[derive(Debug, Default)]
struct Events {
    log: list::Content<Event>,
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
                self.log.push(event);

                if self.log.len() > 1_000 {
                    self.log.remove(0);
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
        let events = container(
            Scrollable::with_direction(
                container(list(&self.log, |_i, event| {
                    text(format!("{event:?}"))
                        .size(14)
                        .font(Font::MONOSPACE)
                        .into()
                }))
                .padding(10),
                scrollable::Direction::Vertical(
                    scrollable::Properties::default()
                        .alignment(scrollable::Alignment::End),
                ),
            )
            .height(Length::Fill),
        )
        .style(container::rounded_box)
        .padding(5);

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

        center(content).padding(10).into()
    }
}
