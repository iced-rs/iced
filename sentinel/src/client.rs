use crate::core::time::SystemTime;
use crate::theme;
use crate::{Input, Timing, SOCKET_ADDRESS};

use tokio::io::{self, AsyncWriteExt};
use tokio::net;
use tokio::sync::mpsc;
use tokio::time;

#[derive(Debug, Clone)]
pub struct Client {
    sender: mpsc::Sender<Input>,
}

impl Client {
    pub fn report_theme_change(&mut self, palette: theme::Palette) {
        let _ = self.sender.try_send(Input::ThemeChanged {
            at: SystemTime::now(),
            palette,
        });
    }

    pub fn report_timing(&mut self, timing: Timing) {
        let _ = self.sender.try_send(Input::TimingMeasured(timing));
    }
}

#[must_use]
pub fn connect() -> Client {
    let (sender, receiver) = mpsc::channel(1_000);

    std::thread::spawn(move || run(receiver));

    Client { sender }
}

#[tokio::main]
async fn run(mut receiver: mpsc::Receiver<Input>) {
    let version = semver::Version::parse(env!("CARGO_PKG_VERSION"))
        .expect("Parse package version");

    loop {
        match _connect().await {
            Ok(mut stream) => {
                let _ = send(
                    &mut stream,
                    Input::Connected {
                        at: SystemTime::now(),
                        version: version.clone(),
                    },
                )
                .await;

                while let Some(input) = receiver.recv().await {
                    match send(&mut stream, input).await {
                        Ok(()) => {}
                        Err(error) => {
                            log::warn!("Error sending message to sentinel server: {error}");
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

async fn _connect() -> Result<io::BufStream<net::TcpStream>, io::Error> {
    log::debug!("Attempting to connect sentinel to server...");
    let stream = net::TcpStream::connect(SOCKET_ADDRESS).await?;

    stream.set_nodelay(true)?;
    stream.writable().await?;

    Ok(io::BufStream::new(stream))
}

async fn send(
    stream: &mut io::BufStream<net::TcpStream>,
    input: Input,
) -> Result<(), io::Error> {
    let bytes = bincode::serialize(&input).expect("Encode input message");
    let size = bytes.len() as u64;

    stream.write_all(&size.to_be_bytes()).await?;
    stream.write_all(&bytes).await?;
    stream.flush().await?;

    Ok(())
}
