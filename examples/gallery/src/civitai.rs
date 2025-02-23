use bytes::Bytes;
use serde::Deserialize;
use sipper::{Straw, sipper};
use tokio::task;

use std::fmt;
use std::io;
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize)]
pub struct Image {
    pub id: Id,
    url: String,
    hash: String,
}

impl Image {
    pub const LIMIT: usize = 99;

    pub async fn list() -> Result<Vec<Self>, Error> {
        let client = reqwest::Client::new();

        #[derive(Deserialize)]
        struct Response {
            items: Vec<Image>,
        }

        let response: Response = client
            .get("https://civitai.com/api/v1/images")
            .query(&[
                ("sort", "Most Reactions"),
                ("period", "Week"),
                ("nsfw", "None"),
                ("limit", &Image::LIMIT.to_string()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(response.items)
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
                    pixels: Bytes::from(pixels),
                },
            })
        })
        .await?
    }

    pub fn download(self, size: Size) -> impl Straw<Rgba, Blurhash, Error> {
        sipper(async move |mut sender| {
            let client = reqwest::Client::new();

            if let Size::Thumbnail { width, height } = size {
                let image = self.clone();

                drop(task::spawn(async move {
                    if let Ok(blurhash) = image.blurhash(width, height).await {
                        sender.send(blurhash).await;
                    }
                }));
            }

            let bytes = client
                .get(match size {
                    Size::Original => self.url,
                    Size::Thumbnail { width, .. } => self
                        .url
                        .split("/")
                        .map(|part| {
                            if part.starts_with("width=") {
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

            let image = task::spawn_blocking(move || {
                Ok::<_, Error>(
                    image::ImageReader::new(io::Cursor::new(bytes))
                        .with_guessed_format()?
                        .decode()?
                        .to_rgba8(),
                )
            })
            .await??;

            Ok(Rgba {
                width: image.width(),
                height: image.height(),
                pixels: Bytes::from(image.into_raw()),
            })
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
    ImageDecodingFailed(Arc<image::ImageError>),
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

impl From<image::ImageError> for Error {
    fn from(error: image::ImageError) -> Self {
        Self::ImageDecodingFailed(Arc::new(error))
    }
}

impl From<blurhash::Error> for Error {
    fn from(error: blurhash::Error) -> Self {
        Self::BlurhashDecodingFailed(Arc::new(error))
    }
}
