use iced::futures::{SinkExt, Stream, StreamExt};
use iced::stream::try_channel;

use std::sync::Arc;

pub fn download(
    url: impl AsRef<str>,
) -> impl Stream<Item = Result<Progress, Error>> {
    try_channel(1, move |mut output| async move {
        let response = reqwest::get(url.as_ref()).await?;
        let total = response.content_length().ok_or(Error::NoContentLength)?;

        let _ = output.send(Progress::Downloading { percent: 0.0 }).await;

        let mut byte_stream = response.bytes_stream();
        let mut downloaded = 0;

        while let Some(next_bytes) = byte_stream.next().await {
            let bytes = next_bytes?;
            downloaded += bytes.len();

            let _ = output
                .send(Progress::Downloading {
                    percent: 100.0 * downloaded as f32 / total as f32,
                })
                .await;
        }

        let _ = output.send(Progress::Finished).await;

        Ok(())
    })
}

#[derive(Debug, Clone)]
pub enum Progress {
    Downloading { percent: f32 },
    Finished,
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
