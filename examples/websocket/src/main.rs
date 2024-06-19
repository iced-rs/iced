mod echo;

use iced::alignment::{self, Alignment};
use iced::widget::{
    self, button, center, column, row, scrollable, text, text_input,
};
use iced::{color, Element, Length, Subscription, Task};
use once_cell::sync::Lazy;

pub fn main() -> iced::Result {
    iced::application("WebSocket - Iced", WebSocket::update, WebSocket::view)
        .load(WebSocket::load)
        .subscription(WebSocket::subscription)
        .run()
}

#[derive(Default)]
struct WebSocket {
    messages: Vec<echo::Message>,
    new_message: String,
    state: State,
}

#[derive(Debug, Clone)]
enum Message {
    NewMessageChanged(String),
    Send(echo::Message),
    Echo(echo::Event),
    Server,
}

impl WebSocket {
    fn load() -> Task<Message> {
        Task::batch([
            Task::perform(echo::server::run(), |_| Message::Server),
            widget::focus_next(),
        ])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NewMessageChanged(new_message) => {
                self.new_message = new_message;

                Task::none()
            }
            Message::Send(message) => match &mut self.state {
                State::Connected(connection) => {
                    self.new_message.clear();

                    connection.send(message);

                    Task::none()
                }
                State::Disconnected => Task::none(),
            },
            Message::Echo(event) => match event {
                echo::Event::Connected(connection) => {
                    self.state = State::Connected(connection);

                    self.messages.push(echo::Message::connected());

                    Task::none()
                }
                echo::Event::Disconnected => {
                    self.state = State::Disconnected;

                    self.messages.push(echo::Message::disconnected());

                    Task::none()
                }
                echo::Event::MessageReceived(message) => {
                    self.messages.push(message);

                    scrollable::snap_to(
                        MESSAGE_LOG.clone(),
                        scrollable::RelativeOffset::END,
                    )
                }
            },
            Message::Server => Task::none(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        echo::connect().map(Message::Echo)
    }

    fn view(&self) -> Element<Message> {
        let message_log: Element<_> = if self.messages.is_empty() {
            center(
                text("Your messages will appear here...")
                    .color(color!(0x888888)),
            )
            .into()
        } else {
            scrollable(
                column(self.messages.iter().map(text).map(Element::from))
                    .spacing(10),
            )
            .id(MESSAGE_LOG.clone())
            .height(Length::Fill)
            .into()
        };

        let new_message_input = {
            let mut input = text_input("Type a message...", &self.new_message)
                .on_input(Message::NewMessageChanged)
                .padding(10);

            let mut button = button(
                text("Send")
                    .height(40)
                    .vertical_alignment(alignment::Vertical::Center),
            )
            .padding([0, 20]);

            if matches!(self.state, State::Connected(_)) {
                if let Some(message) = echo::Message::new(&self.new_message) {
                    input = input.on_submit(Message::Send(message.clone()));
                    button = button.on_press(Message::Send(message));
                }
            }

            row![input, button]
                .spacing(10)
                .align_items(Alignment::Center)
        };

        column![message_log, new_message_input]
            .height(Length::Fill)
            .padding(20)
            .spacing(10)
            .into()
    }
}

enum State {
    Disconnected,
    Connected(echo::Connection),
}

impl Default for State {
    fn default() -> Self {
        Self::Disconnected
    }
}

static MESSAGE_LOG: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);
