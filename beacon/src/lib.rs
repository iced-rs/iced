pub use iced_core as core;
pub use semver::Version;

pub mod client;
pub mod span;

mod stream;

pub use client::Client;
pub use span::Span;

use crate::core::theme;
use crate::core::time::{Duration, SystemTime};

use futures::{SinkExt, Stream};
use tokio::io::{self, AsyncReadExt};
use tokio::net;

#[derive(Debug, Clone)]
pub enum Event {
    Connected {
        at: SystemTime,
        name: String,
        version: Version,
    },
    Disconnected {
        at: SystemTime,
    },
    ThemeChanged {
        at: SystemTime,
        palette: theme::Palette,
    },
    SubscriptionsTracked {
        at: SystemTime,
        amount_alive: usize,
    },
    SpanFinished {
        at: SystemTime,
        duration: Duration,
        span: Span,
    },
    QuitRequested {
        at: SystemTime,
    },
    AlreadyRunning {
        at: SystemTime,
    },
}

impl Event {
    pub fn at(&self) -> SystemTime {
        match self {
            Self::Connected { at, .. }
            | Self::Disconnected { at, .. }
            | Self::ThemeChanged { at, .. }
            | Self::SubscriptionsTracked { at, .. }
            | Self::SpanFinished { at, .. }
            | Self::QuitRequested { at }
            | Self::AlreadyRunning { at } => *at,
        }
    }
}

pub fn is_running() -> bool {
    std::net::TcpListener::bind(client::SERVER_ADDRESS).is_err()
}

pub fn run() -> impl Stream<Item = Event> {
    stream::channel(|mut output| async move {
        let mut buffer = Vec::new();

        let server = loop {
            match net::TcpListener::bind(client::SERVER_ADDRESS).await {
                Ok(server) => break server,
                Err(error) => {
                    if error.kind() == io::ErrorKind::AddrInUse {
                        let _ = output
                            .send(Event::AlreadyRunning {
                                at: SystemTime::now(),
                            })
                            .await;
                    }
                    delay().await;
                }
            };
        };

        loop {
            let Ok((mut stream, _)) = server.accept().await else {
                continue;
            };

            let _ = stream.set_nodelay(true);

            let mut last_message = String::new();
            let mut last_commands_spawned = 0;

            loop {
                match receive(&mut stream, &mut buffer).await {
                    Ok(message) => {
                        match message {
                            client::Message::Connected {
                                at,
                                name,
                                version,
                            } => {
                                let _ = output
                                    .send(Event::Connected {
                                        at,
                                        name,
                                        version,
                                    })
                                    .await;
                            }
                            client::Message::EventLogged { at, event } => {
                                match event {
                                    client::Event::ThemeChanged(palette) => {
                                        let _ = output
                                            .send(Event::ThemeChanged {
                                                at,
                                                palette,
                                            })
                                            .await;
                                    }
                                    client::Event::SubscriptionsTracked(
                                        amount_alive,
                                    ) => {
                                        let _ = output
                                            .send(Event::SubscriptionsTracked {
                                                at,
                                                amount_alive,
                                            })
                                            .await;
                                    }
                                    client::Event::MessageLogged(message) => {
                                        last_message = message;
                                    }
                                    client::Event::CommandsSpawned(
                                        commands,
                                    ) => {
                                        last_commands_spawned = commands;
                                    }
                                    client::Event::SpanStarted(
                                        span::Stage::Update,
                                    ) => {
                                        last_message.clear();
                                        last_commands_spawned = 0;
                                    }
                                    client::Event::SpanStarted(_) => {}
                                    client::Event::SpanFinished(
                                        stage,
                                        duration,
                                    ) => {
                                        let span = match stage {
                                            span::Stage::Boot => Span::Boot,
                                            span::Stage::Update => {
                                                Span::Update {
                                                    message: last_message
                                                        .clone(),
                                                    commands_spawned:
                                                        last_commands_spawned,
                                                }
                                            }
                                            span::Stage::View(window) => {
                                                Span::View { window }
                                            }
                                            span::Stage::Layout(window) => {
                                                Span::Layout { window }
                                            }
                                            span::Stage::Interact(window) => {
                                                Span::Interact { window }
                                            }
                                            span::Stage::Draw(window) => {
                                                Span::Draw { window }
                                            }
                                            span::Stage::Present(window) => {
                                                Span::Present { window }
                                            }
                                            span::Stage::Custom(
                                                window,
                                                name,
                                            ) => Span::Custom { window, name },
                                        };

                                        let _ = output
                                            .send(Event::SpanFinished {
                                                at,
                                                duration,
                                                span,
                                            })
                                            .await;
                                    }
                                }
                            }
                            client::Message::Quit { at } => {
                                let _ = output
                                    .send(Event::QuitRequested { at })
                                    .await;
                            }
                        };
                    }
                    Err(Error::IOFailed(_)) => {
                        let _ = output
                            .send(Event::Disconnected {
                                at: SystemTime::now(),
                            })
                            .await;
                        break;
                    }
                    Err(Error::DecodingFailed(error)) => {
                        log::warn!("Error decoding beacon output: {error}")
                    }
                }
            }
        }
    })
}

async fn receive(
    stream: &mut net::TcpStream,
    buffer: &mut Vec<u8>,
) -> Result<client::Message, Error> {
    let size = stream.read_u64().await? as usize;

    if buffer.len() < size {
        buffer.resize(size, 0);
    }

    let _n = stream.read_exact(&mut buffer[..size]).await?;

    Ok(bincode::deserialize(buffer)?)
}

async fn delay() {
    tokio::time::sleep(Duration::from_secs(2)).await;
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("input/output operation failed: {0}")]
    IOFailed(#[from] io::Error),
    #[error("decoding failed: {0}")]
    DecodingFailed(#[from] Box<bincode::ErrorKind>),
}
