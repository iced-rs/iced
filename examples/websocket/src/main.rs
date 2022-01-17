mod echo;

use iced::alignment::{self, Alignment};
use iced::button::{self, Button};
use iced::executor;
use iced::scrollable::{self, Scrollable};
use iced::text_input::{self, TextInput};
use iced::{
    Application, Color, Column, Command, Container, Element, Length, Row,
    Settings, Subscription, Text,
};

pub fn main() -> iced::Result {
    WebSocket::run(Settings::default())
}

#[derive(Default)]
struct WebSocket {
    messages: Vec<echo::Message>,
    message_log: scrollable::State,
    new_message: String,
    new_message_state: text_input::State,
    new_message_button: button::State,
    state: State,
}

#[derive(Debug, Clone)]
enum Message {
    NewMessageChanged(String),
    Send(echo::Message),
    Echo(echo::Event),
    Server,
}

impl Application for WebSocket {
    type Message = Message;
    type Flags = ();
    type Executor = executor::Default;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self::default(),
            Command::perform(echo::server::run(), |_| Message::Server),
        )
    }

    fn title(&self) -> String {
        String::from("WebSocket - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::NewMessageChanged(new_message) => {
                self.new_message = new_message;
            }
            Message::Send(message) => match &mut self.state {
                State::Connected(connection) => {
                    self.new_message.clear();

                    connection.send(message);
                }
                State::Disconnected => {}
            },
            Message::Echo(event) => match event {
                echo::Event::Connected(connection) => {
                    self.state = State::Connected(connection);

                    self.messages.push(echo::Message::connected());
                }
                echo::Event::Disconnected => {
                    self.state = State::Disconnected;

                    self.messages.push(echo::Message::disconnected());
                }
                echo::Event::MessageReceived(message) => {
                    self.messages.push(message);
                    self.message_log.snap_to(1.0);
                }
            },
            Message::Server => {}
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        echo::connect().map(Message::Echo)
    }

    fn view(&mut self) -> Element<Message> {
        let message_log = if self.messages.is_empty() {
            Container::new(
                Text::new("Your messages will appear here...")
                    .color(Color::from_rgb8(0x88, 0x88, 0x88)),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
        } else {
            self.messages
                .iter()
                .cloned()
                .fold(
                    Scrollable::new(&mut self.message_log),
                    |scrollable, message| scrollable.push(Text::new(message)),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .spacing(10)
                .into()
        };

        let new_message_input = {
            let mut input = TextInput::new(
                &mut self.new_message_state,
                "Type a message...",
                &self.new_message,
                Message::NewMessageChanged,
            )
            .padding(10);

            let mut button = Button::new(
                &mut self.new_message_button,
                Text::new("Send")
                    .height(Length::Fill)
                    .vertical_alignment(alignment::Vertical::Center),
            )
            .padding([0, 20]);

            if matches!(self.state, State::Connected(_)) {
                if let Some(message) = echo::Message::new(&self.new_message) {
                    input = input.on_submit(Message::Send(message.clone()));
                    button = button.on_press(Message::Send(message));
                }
            }

            Row::with_children(vec![input.into(), button.into()])
                .spacing(10)
                .align_items(Alignment::Fill)
        };

        Column::with_children(vec![message_log, new_message_input.into()])
            .width(Length::Fill)
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
