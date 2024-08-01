use iced::futures;
use iced::Subscription;

use std::hash::Hash;

#[cfg(target_arch = "wasm32")]
use futures::{Stream, StreamExt};
#[cfg(target_arch = "wasm32")]
use std::pin::Pin;
// Just a little utility function
pub fn file<I: 'static + Hash + Copy + Send + Sync, T: ToString>(
    id: I,
    url: T,
) -> iced::Subscription<(I, Progress)> {
    Subscription::run_with_id(
        id,
        futures::stream::unfold(State::Ready(url.to_string()), move |state| {
            use iced::futures::FutureExt;

            download(id, state).map(Some)
        }),
    )
}

async fn download<I: Copy>(id: I, state: State) -> ((I, Progress), State) {
    match state {
        State::Ready(url) => {
            let response = reqwest::get(&url).await;
            match response {
                Ok(response) => {
                    if let Some(total) = response.content_length() {
                        (
                            (id, Progress::Started),
                            State::Downloading {
                                #[cfg(target_arch = "wasm32")]
                                stream: response.bytes_stream().boxed_local(),
                                #[cfg(not(target_arch = "wasm32"))]
                                stream: response,
                                total,
                                downloaded: 0,
                            },
                        )
                    } else {
                        ((id, Progress::Errored), State::Finished)
                    }
                }
                Err(_) => ((id, Progress::Errored), State::Finished),
            }
        }
        State::Downloading {
            mut stream,
            total,
            downloaded,
        } => {
            #[cfg(target_arch = "wasm32")]
            let chunk = stream.next().await.map_or(Ok(None), |v| v.map(Some));
            #[cfg(not(target_arch = "wasm32"))]
            let chunk = stream.chunk().await;
            match chunk {
                Ok(Some(chunk)) => {
                    let downloaded = downloaded + chunk.len() as u64;

                    let percentage = (downloaded as f32 / total as f32) * 100.0;

                    (
                        (id, Progress::Advanced(percentage)),
                        State::Downloading {
                            stream,
                            total,
                            downloaded,
                        },
                    )
                }
                Ok(None) => ((id, Progress::Finished), State::Finished),
                Err(_) => ((id, Progress::Errored), State::Finished),
            }
        }
        State::Finished => {
            // We do not let the stream die, as it would start a
            // new download repeatedly if the user is not careful
            // in case of errors.
            iced::futures::future::pending().await
        }
    }
}

#[derive(Debug, Clone)]
pub enum Progress {
    Started,
    Advanced(f32),
    Finished,
    Errored,
}

pub enum State {
    Ready(String),
    Downloading {
        #[cfg(target_arch = "wasm32")]
        stream:
            Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>>>>,
        #[cfg(not(target_arch = "wasm32"))]
        stream: reqwest::Response,
        total: u64,
        downloaded: u64,
    },
    Finished,
}
