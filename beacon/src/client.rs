use crate::Error;
use crate::core::time::{Duration, SystemTime};
use crate::span;
use crate::theme;

use semver::Version;
use serde::{Deserialize, Serialize};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net;
use tokio::sync::{Mutex, mpsc};
use tokio::task;
use tokio::time;

use std::sync::Arc;
use std::sync::atomic::{self, AtomicBool};
use std::thread;

pub const SERVER_ADDRESS: &str = "127.0.0.1:9167";

#[derive(Debug, Clone)]
pub struct Client {
    sender: mpsc::Sender<Action>,
    is_connected: Arc<AtomicBool>,
    _handle: Arc<thread::JoinHandle<()>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    Connected {
        at: SystemTime,
        name: String,
        version: Version,
        theme: Option<theme::Palette>,
        can_time_travel: bool,
    },
    EventLogged {
        at: SystemTime,
        event: Event,
    },
    Quit {
        at: SystemTime,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    ThemeChanged(theme::Palette),
    SpanStarted(span::Stage),
    SpanFinished(span::Stage, Duration),
    MessageLogged { number: usize, message: String },
    CommandsSpawned(usize),
    SubscriptionsTracked(usize),
    LayersRendered(usize),
}

impl Client {
    pub fn log(&self, event: Event) {
        let _ = self.sender.try_send(Action::Send(Message::EventLogged {
            at: SystemTime::now(),
            event,
        }));
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected.load(atomic::Ordering::Relaxed)
    }

    pub fn quit(&self) {
        let _ = self.sender.try_send(Action::Send(Message::Quit {
            at: SystemTime::now(),
        }));
    }

    pub fn subscribe(&self) -> mpsc::Receiver<Command> {
        let (sender, receiver) = mpsc::channel(100);
        let _ = self.sender.try_send(Action::Forward(sender));

        receiver
    }
}

#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub name: &'static str,
    pub theme: Option<theme::Palette>,
    pub can_time_travel: bool,
}

#[must_use]
pub fn connect(metadata: Metadata) -> Client {
    let (sender, receiver) = mpsc::channel(10_000);
    let is_connected = Arc::new(AtomicBool::new(false));

    let handle = {
        let is_connected = is_connected.clone();

        std::thread::spawn(move || run(metadata, is_connected, receiver))
    };

    Client {
        sender,
        is_connected,
        _handle: Arc::new(handle),
    }
}

enum Action {
    Send(Message),
    Forward(mpsc::Sender<Command>),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Command {
    RewindTo { message: usize },
    GoLive,
}

#[tokio::main]
async fn run(
    mut metadata: Metadata,
    is_connected: Arc<AtomicBool>,
    mut receiver: mpsc::Receiver<Action>,
) {
    let version = semver::Version::parse(env!("CARGO_PKG_VERSION"))
        .expect("Parse package version");

    let command_sender = {
        // Discard by default
        let (sender, _receiver) = mpsc::channel(1);

        Arc::new(Mutex::new(sender))
    };

    loop {
        match _connect().await {
            Ok(stream) => {
                is_connected.store(true, atomic::Ordering::Relaxed);

                let (mut reader, mut writer) = stream.into_split();

                let _ = send(
                    &mut writer,
                    Message::Connected {
                        at: SystemTime::now(),
                        name: metadata.name.to_owned(),
                        version: version.clone(),
                        can_time_travel: metadata.can_time_travel,
                        theme: metadata.theme,
                    },
                )
                .await;

                {
                    let command_sender = command_sender.clone();

                    drop(task::spawn(async move {
                        let mut buffer = Vec::new();

                        loop {
                            match receive(&mut reader, &mut buffer).await {
                                Ok(command) => {
                                    match command {
                                        Command::RewindTo { .. }
                                        | Command::GoLive
                                            if !metadata.can_time_travel =>
                                        {
                                            continue;
                                        }
                                        _ => {}
                                    }

                                    let sender = command_sender.lock().await;
                                    let _ = sender.send(command).await;
                                }
                                Err(Error::DecodingFailed(_)) => {}
                                Err(Error::IOFailed(_)) => break,
                            }
                        }
                    }))
                };

                while let Some(action) = receiver.recv().await {
                    match action {
                        Action::Send(message) => {
                            if let Message::EventLogged {
                                event: Event::ThemeChanged(palette),
                                ..
                            } = message
                            {
                                metadata.theme = Some(palette);
                            }

                            match send(&mut writer, message).await {
                                Ok(()) => {}
                                Err(error) => {
                                    if error.kind() != io::ErrorKind::BrokenPipe
                                    {
                                        log::warn!(
                                            "Error sending message to server: {error}"
                                        );
                                    }

                                    is_connected.store(
                                        false,
                                        atomic::Ordering::Relaxed,
                                    );
                                    break;
                                }
                            }
                        }
                        Action::Forward(sender) => {
                            *command_sender.lock().await = sender;
                        }
                    }
                }
            }
            Err(_) => {
                is_connected.store(false, atomic::Ordering::Relaxed);
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
    stream: &mut net::tcp::OwnedWriteHalf,
    message: Message,
) -> Result<(), io::Error> {
    let bytes = bincode::serialize(&message).expect("Encode input message");
    let size = bytes.len() as u64;

    stream.write_all(&size.to_be_bytes()).await?;
    stream.write_all(&bytes).await?;
    stream.flush().await?;

    Ok(())
}

async fn receive(
    stream: &mut net::tcp::OwnedReadHalf,
    buffer: &mut Vec<u8>,
) -> Result<Command, Error> {
    let size = stream.read_u64().await? as usize;

    if buffer.len() < size {
        buffer.resize(size, 0);
    }

    let _n = stream.read_exact(&mut buffer[..size]).await?;

    Ok(bincode::deserialize(buffer)?)
}
