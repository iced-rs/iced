use iced::futures::StreamExt;
use iced::task::{Straw, sipper};

use std::sync::Arc;

pub fn download(url: impl AsRef<str>) -> impl Straw<(), Progress, Error> {
    sipper(async move |mut progress| {
        let response = reqwest::get(url.as_ref()).await?;
        let total = response.content_length().ok_or(Error::NoContentLength)?;

        let _ = progress.send(Progress { percent: 0.0 }).await;

        let mut byte_stream = response.bytes_stream();
        let mut downloaded = 0;

        while let Some(next_bytes) = byte_stream.next().await {
            let bytes = next_bytes?;
            downloaded += bytes.len();

            let _ = progress
                .send(Progress {
                    percent: 100.0 * downloaded as f32 / total as f32,
                })
                .await;
        }

        Ok(())
    })
}

#[derive(Debug, Clone)]
pub struct Progress {
    pub percent: f32,
}

#[derive(Debug, Clone)]
pub enum Error {
    RequestFailed(Arc<reqwest::Error>),
    NoContentLength,
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::RequestFailed(Arc::new(error))
    }
}
