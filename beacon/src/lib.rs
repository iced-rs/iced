pub use iced_core as core;
pub use semver::Version;

pub mod client;
pub mod span;

mod error;
mod stream;

pub use client::Client;
pub use span::Span;

use crate::core::theme;
use crate::core::time::{Duration, SystemTime};
use crate::error::Error;
use crate::span::present;

use futures::{SinkExt, Stream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net;
use tokio::sync::mpsc;
use tokio::task;

#[derive(Debug, Clone)]
pub struct Connection {
    commands: mpsc::Sender<client::Command>,
}

impl Connection {
    pub fn rewind_to<'a>(
        &self,
        message: usize,
    ) -> impl Future<Output = ()> + 'a {
        let commands = self.commands.clone();

        async move {
            let _ = commands.send(client::Command::RewindTo { message }).await;
        }
    }

    pub fn go_live<'a>(&self) -> impl Future<Output = ()> + 'a {
        let commands = self.commands.clone();

        async move {
            let _ = commands.send(client::Command::GoLive).await;
        }
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Connected {
        connection: Connection,
        at: SystemTime,
        name: String,
        version: Version,
        theme: Option<theme::Palette>,
        can_time_travel: bool,
    },
    Disconnected {
        at: SystemTime,
    },
    ThemeChanged {
        at: SystemTime,
        palette: theme::Palette,
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
            let Ok((stream, _)) = server.accept().await else {
                continue;
            };

            let (mut reader, mut writer) = {
                let _ = stream.set_nodelay(true);
                stream.into_split()
            };

            let (command_sender, mut command_receiver) = mpsc::channel(1);
            let mut last_message = String::new();
            let mut last_update_number = 0;
            let mut last_tasks = 0;
            let mut last_subscriptions = 0;
            let mut last_present_layers = 0;
            let mut last_prepare = present::Stage::default();
            let mut last_render = present::Stage::default();

            drop(task::spawn(async move {
                let mut last_message_number = None;

                while let Some(command) = command_receiver.recv().await {
                    match command {
                        client::Command::RewindTo { message } => {
                            if Some(message) == last_message_number {
                                continue;
                            }

                            last_message_number = Some(message);
                        }
                        client::Command::GoLive => {
                            last_message_number = None;
                        }
                    }

                    let _ =
                        send(&mut writer, command).await.inspect_err(|error| {
                            log::error!("Error when sending command: {error}")
                        });
                }
            }));

            loop {
                match receive(&mut reader, &mut buffer).await {
                    Ok(message) => {
                        match message {
                            client::Message::Connected {
                                at,
                                name,
                                version,
                                theme,
                                can_time_travel,
                            } => {
                                let _ = output
                                    .send(Event::Connected {
                                        connection: Connection {
                                            commands: command_sender.clone(),
                                        },
                                        at,
                                        name,
                                        version,
                                        theme,
                                        can_time_travel,
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
                                        last_subscriptions = amount_alive;
                                    }
                                    client::Event::MessageLogged {
                                        number,
                                        message,
                                    } => {
                                        last_update_number = number;
                                        last_message = message;
                                    }
                                    client::Event::CommandsSpawned(
                                        commands,
                                    ) => {
                                        last_tasks = commands;
                                    }
                                    client::Event::LayersRendered(layers) => {
                                        last_present_layers = layers;
                                    }
                                    client::Event::SpanStarted(
                                        span::Stage::Update,
                                    ) => {
                                        last_message.clear();
                                        last_tasks = 0;
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
                                                    number: last_update_number,
                                                    message: last_message
                                                        .clone(),
                                                    tasks: last_tasks,
                                                    subscriptions:
                                                        last_subscriptions,
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
                                            span::Stage::Prepare(primitive)
                                            | span::Stage::Render(primitive) => {
                                                let stage = if matches!(
                                                    stage,
                                                    span::Stage::Prepare(_),
                                                ) {
                                                    &mut last_prepare
                                                } else {
                                                    &mut last_render
                                                };

                                                let primitive = match primitive {
                                                    present::Primitive::Quad => &mut stage.quads,
                                                    present::Primitive::Triangle => &mut stage.triangles,
                                                    present::Primitive::Shader => &mut stage.shaders,
                                                    present::Primitive::Text => &mut stage.text,
                                                    present::Primitive::Image => &mut stage.images,
                                                };

                                                *primitive += duration;

                                                continue;
                                            }
                                            span::Stage::Present(window) => {
                                                let span = Span::Present {
                                                    window,
                                                    prepare: last_prepare,
                                                    render: last_render,
                                                    layers: last_present_layers,
                                                };

                                                last_prepare =
                                                    present::Stage::default();
                                                last_render =
                                                    present::Stage::default();
                                                last_present_layers = 0;

                                                span
                                            }
                                            span::Stage::Custom(name) => {
                                                Span::Custom { name }
                                            }
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
    stream: &mut net::tcp::OwnedReadHalf,
    buffer: &mut Vec<u8>,
) -> Result<client::Message, Error> {
    let size = stream.read_u64().await? as usize;

    if buffer.len() < size {
        buffer.resize(size, 0);
    }

    let _n = stream.read_exact(&mut buffer[..size]).await?;

    Ok(bincode::deserialize(buffer)?)
}

async fn send(
    stream: &mut net::tcp::OwnedWriteHalf,
    command: client::Command,
) -> Result<(), io::Error> {
    let bytes = bincode::serialize(&command).expect("Encode input message");
    let size = bytes.len() as u64;

    stream.write_all(&size.to_be_bytes()).await?;
    stream.write_all(&bytes).await?;
    stream.flush().await?;

    Ok(())
}

async fn delay() {
    tokio::time::sleep(Duration::from_secs(2)).await;
}
