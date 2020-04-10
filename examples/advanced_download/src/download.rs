use std::hash::Hash;

use bytes::BufMut;
use futures_util::stream::{self, BoxStream};
use reqwest;

pub struct Download {
    pub url: String,
}

pub enum State {
    Ready(String),
    Downloading {
        url: String,
        response: reqwest::Response,
        total: u64,
        bytes: Vec<u8>,
    },
    Finished,
}

// Make sure iced can use our download stream
impl<H, I> iced_native::subscription::Recipe<H, I> for Download
where
    H: std::hash::Hasher,
{
    type Output = (String, Progress);

    fn hash(&self, state: &mut H) {
        std::any::TypeId::of::<Self>().hash(state);
        self.url.hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: BoxStream<'static, I>,
    ) -> BoxStream<'static, Self::Output> {
        Box::pin(stream::unfold(
            State::Ready(self.url.to_string()),
            |state| async move {
                match state {
                    State::Ready(url) => match reqwest::get(&url).await {
                        Ok(response) => {
                            if let Some(total) = response.content_length() {
                                Some((
                                    (url.to_string(), Progress::Started),
                                    State::Downloading {
                                        url,
                                        response,
                                        total,
                                        bytes: vec![],
                                    },
                                ))
                            } else {
                                Some((
                                    (url, Progress::Errored),
                                    State::Finished,
                                ))
                            }
                        }
                        Err(_) => {
                            Some(((url, Progress::Errored), State::Finished))
                        }
                    },
                    State::Downloading {
                        url,
                        mut response,
                        total,
                        mut bytes,
                    } => match response.chunk().await {
                        Ok(Some(chunk)) => {
                            let downloaded =
                                bytes.len() as u64 + chunk.len() as u64;
                            let percentage =
                                (downloaded as f32 / total as f32) * 100.0;
                            bytes.put(chunk);
                            Some((
                                (
                                    url.to_string(),
                                    Progress::Advanced(percentage),
                                ),
                                State::Downloading {
                                    url,
                                    response,
                                    total,
                                    bytes,
                                },
                            ))
                        }
                        Ok(None) => Some((
                            (url, Progress::Finished(bytes)),
                            State::Finished,
                        )),
                        Err(_) => {
                            Some(((url, Progress::Errored), State::Finished))
                        }
                    },
                    State::Finished => {
                        // We do not let the stream die, as it would start a
                        // new download repeatedly if the user is not careful
                        // in case of errors.
                        iced::futures::future::pending::<()>().await;

                        None
                    }
                }
            },
        ))
    }
}

#[derive(Debug, Clone)]
pub enum Progress {
    Started,
    Advanced(f32),
    Finished(Vec<u8>),
    Errored,
}
