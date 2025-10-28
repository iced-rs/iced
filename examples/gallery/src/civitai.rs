use serde::Deserialize;
use sipper::{Straw, sipper};
use tokio::task;

use std::fmt;
use std::io;
use std::sync::{Arc, LazyLock};

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

#[derive(Debug, Clone, Deserialize)]
pub struct Image {
    pub id: Id,
    url: String,
    hash: String,
}

impl Image {
    pub const LIMIT: usize = 96;

    pub async fn list() -> Result<Vec<Self>, Error> {
        #[derive(Deserialize)]
        struct Response {
            items: Vec<Image>,
        }

        let response: Response = CLIENT
            .get("https://civitai.com/api/v1/images")
            .query(&[
                ("sort", "Most Reactions"),
                ("period", "Month"),
                ("nsfw", "None"),
                ("limit", &Image::LIMIT.to_string()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(response
            .items
            .into_iter()
            .filter(|image| !image.url.ends_with(".mp4"))
            .collect())
    }

    pub async fn blurhash(
        self,
        width: u32,
        height: u32,
    ) -> Result<Blurhash, Error> {
        task::spawn_blocking(move || {
            let pixels = blurhash::decode(&self.hash, width, height, 1.0)?;

            Ok::<_, Error>(Blurhash {
                rgba: Rgba {
                    width,
                    height,
                    pixels: Bytes(pixels.into()),
                },
            })
        })
        .await?
    }

    pub fn download(self, size: Size) -> impl Straw<Bytes, Blurhash, Error> {
        sipper(async move |mut sender| {
            if let Size::Thumbnail { width, height } = size {
                let image = self.clone();

                drop(task::spawn(async move {
                    if let Ok(blurhash) = image.blurhash(width, height).await {
                        sender.send(blurhash).await;
                    }
                }));
            }

            let bytes = CLIENT
                .get(match size {
                    Size::Original => self.url,
                    Size::Thumbnail { width, .. } => self
                        .url
                        .split("/")
                        .map(|part| {
                            if part.starts_with("width=")
                                || part.starts_with("original=")
                            {
                                format!("width={}", width * 2) // High DPI
                            } else {
                                part.to_owned()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("/"),
                })
                .send()
                .await?
                .error_for_status()?
                .bytes()
                .await?;

            Ok(Bytes(bytes))
        })
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize,
)]
pub struct Id(u32);

#[derive(Debug, Clone)]
pub struct Blurhash {
    pub rgba: Rgba,
}

#[derive(Clone)]
pub struct Rgba {
    pub width: u32,
    pub height: u32,
    pub pixels: Bytes,
}

impl fmt::Debug for Rgba {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Rgba")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

#[derive(Clone)]
pub struct Bytes(bytes::Bytes);

impl Bytes {
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl From<Bytes> for bytes::Bytes {
    fn from(value: Bytes) -> Self {
        value.0
    }
}

impl fmt::Debug for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Compressed")
            .field("bytes", &self.0.len())
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Size {
    Original,
    Thumbnail { width: u32, height: u32 },
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Error {
    RequestFailed(Arc<reqwest::Error>),
    IOFailed(Arc<io::Error>),
    JoinFailed(Arc<task::JoinError>),
    ImageDecodingFailed,
    BlurhashDecodingFailed(Arc<blurhash::Error>),
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::RequestFailed(Arc::new(error))
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::IOFailed(Arc::new(error))
    }
}

impl From<task::JoinError> for Error {
    fn from(error: task::JoinError) -> Self {
        Self::JoinFailed(Arc::new(error))
    }
}

impl From<blurhash::Error> for Error {
    fn from(error: blurhash::Error) -> Self {
        Self::BlurhashDecodingFailed(Arc::new(error))
    }
}
