pub use iced_core as core;
pub use iced_style as style;

pub mod client;
pub mod timing;

use crate::style::theme;
use crate::timing::Timing;

use futures::future;
use futures::stream::{self, Stream, StreamExt};
use semver::Version;
use serde::{Deserialize, Serialize};
use tokio::io::{self, AsyncReadExt, BufStream};
use tokio::net;

pub const SOCKET_ADDRESS: &str = "127.0.0.1:9167";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Input {
    Connected(Version),
    TimingMeasured(Timing),
    ThemeChanged(theme::Palette),
}

#[derive(Debug, Clone)]
pub enum Event {
    Connected(Version),
    Disconnected,
    TimingMeasured(Timing),
    ThemeChanged(theme::Palette),
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
                Ok((_, Event::Disconnected)) | Err(_) => {
                    Some((Some(Event::Disconnected), State::Disconnected))
                }
                Ok((stream, message)) => {
                    Some((Some(message), State::Connected(stream)))
                }
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
) -> Result<(BufStream<net::TcpStream>, Event), io::Error> {
    let mut bytes = Vec::new();

    loop {
        let size = stream.read_u64().await? as usize;

        if bytes.len() < size {
            bytes.resize(size, 0);
        }

        let _n = stream.read_exact(&mut bytes[..size]).await?;

        match bincode::deserialize(&bytes) {
            Ok(input) => {
                return Ok((
                    stream,
                    match dbg!(input) {
                        Input::Connected(version) => Event::Connected(version),
                        Input::TimingMeasured(timing) => {
                            Event::TimingMeasured(timing)
                        }
                        Input::ThemeChanged(palette) => {
                            Event::ThemeChanged(palette)
                        }
                    },
                ));
            }
            Err(_) => {
                // TODO: Log decoding error
            }
        }
    }
}
