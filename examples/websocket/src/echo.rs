pub mod server;

use iced::futures;
use iced::subscription::{self, Subscription};

use futures::channel::mpsc;
use futures::sink::SinkExt;
use futures::stream::StreamExt;

use async_tungstenite::tungstenite;
use std::fmt;

pub fn connect() -> Subscription<Event> {
    struct Connect;

    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        |mut output| async move {
            let mut state = State::Disconnected;

            loop {
                match &mut state {
                    State::Disconnected => {
                        const ECHO_SERVER: &str = "ws://127.0.0.1:3030";

                        match async_tungstenite::tokio::connect_async(
                            ECHO_SERVER,
                        )
                        .await
                        {
                            Ok((websocket, _)) => {
                                let (sender, receiver) = mpsc::channel(100);

                                let _ = output
                                    .send(Event::Connected(Connection(sender)))
                                    .await;

                                state = State::Connected(websocket, receiver);
                            }
                            Err(_) => {
                                tokio::time::sleep(
                                    tokio::time::Duration::from_secs(1),
                                )
                                .await;

                                let _ = output.send(Event::Disconnected).await;
                            }
                        }
                    }
                    State::Connected(websocket, input) => {
                        let mut fused_websocket = websocket.by_ref().fuse();

                        futures::select! {
                            received = fused_websocket.select_next_some() => {
                                match received {
                                    Ok(tungstenite::Message::Text(message)) => {
                                       let _ = output.send(Event::MessageReceived(Message::User(message))).await;
                                    }
                                    Err(_) => {
                                        let _ = output.send(Event::Disconnected).await;

                                        state = State::Disconnected;
                                    }
                                    Ok(_) => continue,
                                }
                            }

                            message = input.select_next_some() => {
                                let result = websocket.send(tungstenite::Message::Text(message.to_string())).await;

                                if result.is_err() {
                                    let _ = output.send(Event::Disconnected).await;

                                    state = State::Disconnected;
                                }
                            }
                        }
                    }
                }
            }
        },
    )
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
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
        self.0
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

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::Connected => write!(f, "Connected successfully!"),
            Message::Disconnected => {
                write!(f, "Connection lost... Retrying...")
            }
            Message::User(message) => write!(f, "{message}"),
        }
    }
}
