pub mod server;

use iced_futures::futures;
use iced_native::subscription::{self, Subscription};

use futures::channel::mpsc;
use futures::sink::SinkExt;
use futures::stream::StreamExt;

use async_tungstenite::tungstenite;

pub fn connect() -> Subscription<Event> {
    struct Connect;

    subscription::unfold(
        std::any::TypeId::of::<Connect>(),
        State::Disconnected,
        |state| async move {
            match state {
                State::Disconnected => {
                    const ECHO_SERVER: &str = "ws://localhost:3030";

                    match async_tungstenite::tokio::connect_async(ECHO_SERVER)
                        .await
                    {
                        Ok((websocket, _)) => {
                            let (sender, receiver) = mpsc::channel(100);

                            (
                                Some(Event::Connected(Connection(sender))),
                                State::Connected(websocket, receiver),
                            )
                        }
                        Err(_) => {
                            let _ = tokio::time::sleep(
                                tokio::time::Duration::from_secs(1),
                            )
                            .await;

                            (Some(Event::Disconnected), State::Disconnected)
                        }
                    }
                }
                State::Connected(mut websocket, mut input) => {
                    let mut fused_websocket = websocket.by_ref().fuse();

                    futures::select! {
                        received = fused_websocket.select_next_some() => {
                            match received {
                                Ok(tungstenite::Message::Text(message)) => {
                                    (
                                        Some(Event::MessageReceived(Message::User(message))),
                                        State::Connected(websocket, input)
                                    )
                                }
                                Ok(_) => {
                                    (None, State::Connected(websocket, input))
                                }
                                Err(_) => {
                                    (Some(Event::Disconnected), State::Disconnected)
                                }
                            }
                        }

                        message = input.select_next_some() => {
                            let result = websocket.send(tungstenite::Message::Text(String::from(message))).await;

                            if result.is_ok() {
                                (None, State::Connected(websocket, input))
                            } else {
                                (Some(Event::Disconnected), State::Disconnected)
                            }
                        }
                    }
                }
            }
        },
    )
}

#[derive(Debug)]
enum State {
    Disconnected,
    Connected(
        async_tungstenite::WebSocketStream<
            async_tungstenite::tokio::ConnectStream,
        >,
        mpsc::Receiver<Message>,
    ),
}

#[derive(Debug, Clone)]
pub enum Event {
    Connected(Connection),
    Disconnected,
    MessageReceived(Message),
}

#[derive(Debug, Clone)]
pub struct Connection(mpsc::Sender<Message>);

impl Connection {
    pub fn send(&mut self, message: Message) {
        let _ = self
            .0
            .try_send(message)
            .expect("Send message to echo server");
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Connected,
    Disconnected,
    User(String),
}

impl Message {
    pub fn new(message: &str) -> Option<Self> {
        if message.is_empty() {
            None
        } else {
            Some(Self::User(message.to_string()))
        }
    }

    pub fn connected() -> Self {
        Message::Connected
    }

    pub fn disconnected() -> Self {
        Message::Disconnected
    }
}

impl From<Message> for String {
    fn from(message: Message) -> Self {
        match message {
            Message::Connected => String::from("Connected successfully!"),
            Message::Disconnected => {
                String::from("Connection lost... Retrying...")
            }
            Message::User(message) => message,
        }
    }
}
