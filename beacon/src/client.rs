use crate::core::time::{Duration, SystemTime};
use crate::span;
use crate::theme;

use semver::Version;
use serde::{Deserialize, Serialize};
use tokio::io::{self, AsyncWriteExt};
use tokio::net;
use tokio::sync::mpsc;
use tokio::time;

use std::sync::Arc;
use std::thread;

pub const SERVER_ADDRESS: &str = "127.0.0.1:9167";

#[derive(Debug, Clone)]
pub struct Client {
    sender: mpsc::Sender<Message>,
    _handle: Arc<thread::JoinHandle<()>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    Connected {
        at: SystemTime,
        name: String,
        version: Version,
    },
    EventLogged {
        at: SystemTime,
        event: Event,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    ThemeChanged(theme::Palette),
    SpanStarted(span::Stage),
    SpanFinished(span::Stage, Duration),
}

impl Client {
    pub fn log(&self, event: Event) {
        let _ = self.sender.try_send(Message::EventLogged {
            at: SystemTime::now(),
            event,
        });
    }
}

#[must_use]
pub fn connect(name: String) -> Client {
    let (sender, receiver) = mpsc::channel(100);

    let handle = std::thread::spawn(move || run(name, receiver));

    Client {
        sender,
        _handle: Arc::new(handle),
    }
}

#[tokio::main]
async fn run(name: String, mut receiver: mpsc::Receiver<Message>) {
    let version = semver::Version::parse(env!("CARGO_PKG_VERSION"))
        .expect("Parse package version");

    loop {
        match _connect().await {
            Ok(mut stream) => {
                let _ = send(
                    &mut stream,
                    Message::Connected {
                        at: SystemTime::now(),
                        name: name.clone(),
                        version: version.clone(),
                    },
                )
                .await;

                while let Some(output) = receiver.recv().await {
                    match send(&mut stream, output).await {
                        Ok(()) => {}
                        Err(error) => {
                            log::warn!(
                                "Error sending message to server: {error}"
                            );
                            break;
                        }
                    }
                }
            }
            Err(_) => {
                time::sleep(time::Duration::from_secs(2)).await;
            }
        }
    }
}

async fn _connect() -> Result<net::TcpStream, io::Error> {
    log::debug!("Attempting to connect to server...");
    let stream = net::TcpStream::connect(SERVER_ADDRESS).await?;

    stream.set_nodelay(true)?;
    stream.writable().await?;

    Ok(stream)
}

async fn send(
    stream: &mut net::TcpStream,
    message: Message,
) -> Result<(), io::Error> {
    let bytes = bincode::serialize(&message).expect("Encode input message");
    let size = bytes.len() as u64;

    stream.write_all(&size.to_be_bytes()).await?;
    stream.write_all(&bytes).await?;
    stream.flush().await?;

    Ok(())
}
