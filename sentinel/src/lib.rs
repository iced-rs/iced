pub use iced_core as core;
pub use iced_style as style;
pub use semver::Version;

pub mod client;
pub mod timing;

use crate::core::time::SystemTime;
use crate::style::theme;
use crate::timing::Timing;

use futures::future;
use futures::stream::{self, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::io::{self, AsyncReadExt, BufStream};
use tokio::net;

pub const SOCKET_ADDRESS: &str = "127.0.0.1:9167";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Input {
    Connected {
        at: SystemTime,
        version: Version,
    },
    ThemeChanged {
        at: SystemTime,
        palette: theme::Palette,
    },
    TimingMeasured(Timing),
}

#[derive(Debug, Clone)]
pub enum Event {
    Connected {
        at: SystemTime,
        version: Version,
    },
    Disconnected {
        at: SystemTime,
    },
    ThemeChanged {
        at: SystemTime,
        palette: theme::Palette,
    },
    TimingMeasured(Timing),
}

impl Event {
    pub fn at(&self) -> SystemTime {
        match self {
            Self::Connected { at, .. }
            | Self::Disconnected { at }
            | Self::ThemeChanged { at, .. } => *at,
            Self::TimingMeasured(timing) => timing.start,
        }
    }
}

pub fn run() -> impl Stream<Item = Event> {
    enum State {
        Disconnected,
        Connected(BufStream<net::TcpStream>),
    }

    stream::unfold(State::Disconnected, |state| async {
        match state {
            State::Disconnected => match connect().await {
                Ok(stream) => {
                    let stream = BufStream::new(stream);

                    Some((None, State::Connected(stream)))
                }
                Err(_error) => Some((None, State::Disconnected)),
            },
            State::Connected(stream) => match receive(stream).await {
                Ok((stream, input)) => {
                    let event = match dbg!(input) {
                        Input::Connected { at, version } => {
                            Event::Connected { at, version }
                        }
                        Input::TimingMeasured(timing) => {
                            Event::TimingMeasured(timing)
                        }
                        Input::ThemeChanged { at, palette } => {
                            Event::ThemeChanged { at, palette }
                        }
                    };

                    Some((Some(event), State::Connected(stream)))
                }
                Err(_) => Some((
                    Some(Event::Disconnected {
                        at: SystemTime::now(),
                    }),
                    State::Disconnected,
                )),
            },
        }
    })
    .filter_map(future::ready)
}

async fn connect() -> Result<net::TcpStream, io::Error> {
    let listener = net::TcpListener::bind(SOCKET_ADDRESS).await?;

    let (stream, _) = listener.accept().await?;

    stream.set_nodelay(true)?;
    stream.readable().await?;

    Ok(stream)
}

async fn receive(
    mut stream: BufStream<net::TcpStream>,
) -> Result<(BufStream<net::TcpStream>, Input), io::Error> {
    let mut bytes = Vec::new();

    loop {
        let size = stream.read_u64().await? as usize;

        if bytes.len() < size {
            bytes.resize(size, 0);
        }

        let _n = stream.read_exact(&mut bytes[..size]).await?;

        match bincode::deserialize(&bytes) {
            Ok(input) => {
                return Ok((stream, input));
            }
            Err(_) => {
                log::warn!("Error decoding sentinel message");
            }
        }
    }
}
