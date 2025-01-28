use bytes::Bytes;
use serde::Deserialize;
use tokio::task;

use std::fmt;
use std::io;
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize)]
pub struct Image {
    pub id: Id,
    url: String,
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

    pub async fn download(self, size: Size) -> Result<Rgba, Error> {
        let client = reqwest::Client::new();

        let bytes = client
            .get(match size {
                Size::Original => self.url,
                Size::Thumbnail => self
                    .url
                    .split("/")
                    .map(|part| {
                        if part.starts_with("width=") {
                            "width=640"
                        } else {
                            part
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
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize,
)]
pub struct Id(u32);

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
    Thumbnail,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Error {
    RequestFailed(Arc<reqwest::Error>),
    IOFailed(Arc<io::Error>),
    JoinFailed(Arc<task::JoinError>),
    ImageDecodingFailed(Arc<image::ImageError>),
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
