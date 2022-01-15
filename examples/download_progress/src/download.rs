use futures::Stream;
use iced_futures::futures;
use iced_native::subscription;

use std::hash::Hash;

// Just a little utility function
pub fn file<I: 'static + Hash + Copy + Send, T: ToString>(
    id: I,
    url: T,
) -> iced::Subscription<(I, Progress)> {
    subscription::run(
        Download {
            id,
            url: url.to_string(),
        },
        download,
    )
}

#[derive(Debug, Hash, Clone)]
pub struct Download<I> {
    id: I,
    url: String,
}

fn download<I: Copy>(
    download: Download<I>,
) -> impl Stream<Item = (I, Progress)> {
    let id = download.id;

    futures::stream::unfold(
        State::Ready(download.url),
        move |state| async move {
            match state {
                State::Ready(url) => {
                    let response = reqwest::get(&url).await;

                    match response {
                        Ok(response) => {
                            if let Some(total) = response.content_length() {
                                Some((
                                    (id, Progress::Started),
                                    State::Downloading {
                                        response,
                                        total,
                                        downloaded: 0,
                                    },
                                ))
                            } else {
                                Some(((id, Progress::Errored), State::Finished))
                            }
                        }
                        Err(_) => {
                            Some(((id, Progress::Errored), State::Finished))
                        }
                    }
                }
                State::Downloading {
                    mut response,
                    total,
                    downloaded,
                } => match response.chunk().await {
                    Ok(Some(chunk)) => {
                        let downloaded = downloaded + chunk.len() as u64;

                        let percentage =
                            (downloaded as f32 / total as f32) * 100.0;

                        Some((
                            (id, Progress::Advanced(percentage)),
                            State::Downloading {
                                response,
                                total,
                                downloaded,
                            },
                        ))
                    }
                    Ok(None) => {
                        Some(((id, Progress::Finished), State::Finished))
                    }
                    Err(_) => Some(((id, Progress::Errored), State::Finished)),
                },
                State::Finished => {
                    // We do not let the stream die, as it would start a
                    // new download repeatedly if the user is not careful
                    // in case of errors.
                    let _: () = iced::futures::future::pending().await;

                    None
                }
            }
        },
    )
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
        response: reqwest::Response,
        total: u64,
        downloaded: u64,
    },
    Finished,
}
