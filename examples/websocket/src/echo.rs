pub mod server;

use iced::futures;
use iced::task::{Never, Sipper, sipper};
use iced::widget::text;

use futures::channel::mpsc;
use futures::sink::SinkExt;
use futures::stream::StreamExt;

use async_tungstenite::tungstenite;
use std::fmt;

pub fn connect() -> impl Sipper<Never, Event> {
    sipper(async |mut output| {
        loop {
            const ECHO_SERVER: &str = "ws://127.0.0.1:3030";

            let (mut websocket, mut input) =
                match async_tungstenite::tokio::connect_async(ECHO_SERVER).await
                {
                    Ok((websocket, _)) => {
                        let (sender, receiver) = mpsc::channel(100);

                        output.send(Event::Connected(Connection(sender))).await;

                        (websocket.fuse(), receiver)
                    }
                    Err(_) => {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1))
                            .await;

                        output.send(Event::Disconnected).await;
                        continue;
                    }
                };

            loop {
                futures::select! {
                    received = websocket.select_next_some() => {
                        match received {
                            Ok(tungstenite::Message::Text(message)) => {
                                output.send(Event::MessageReceived(Message::User(message))).await;
                            }
                            Err(_) => {
                                output.send(Event::Disconnected).await;
                                break;
                            }
                            Ok(_) => {},
                        }
                    }
                    message = input.select_next_some() => {
                        let result = websocket.send(tungstenite::Message::Text(message.to_string())).await;

                        if result.is_err() {
                            output.send(Event::Disconnected).await;
                        }
                    }
                }
            }
        }
    })
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

    pub fn as_str(&self) -> &str {
        match self {
            Message::Connected => "Connected successfully!",
            Message::Disconnected => "Connection lost... Retrying...",
            Message::User(message) => message.as_str(),
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'a> text::IntoFragment<'a> for &'a Message {
    fn into_fragment(self) -> text::Fragment<'a> {
        text::Fragment::Borrowed(self.as_str())
    }
}
