use iced_native::subscription;

use std::hash::Hash;

// Just a little utility function
pub fn file<I: 'static + Hash + Copy + Send + Sync, T: ToString>(
    id: I,
    url: T,
) -> iced::Subscription<(I, Progress)> {
    subscription::unfold(id, State::Ready(url.to_string()), move |state| {
        download(id, state)
    })
}

#[derive(Debug, Hash, Clone)]
pub struct Download<I> {
    id: I,
    url: String,
}

async fn download<I: Copy>(
    id: I,
    state: State,
) -> (Option<(I, Progress)>, State) {
    match state {
        State::Ready(url) => {
            let response = reqwest::get(&url).await;

            match response {
                Ok(response) => {
                    if let Some(total) = response.content_length() {
                        (
                            Some((id, Progress::Started)),
                            State::Downloading {
                                response,
                                total,
                                downloaded: 0,
                            },
                        )
                    } else {
                        (Some((id, Progress::Errored)), State::Finished)
                    }
                }
                Err(_) => (Some((id, Progress::Errored)), State::Finished),
            }
        }
        State::Downloading {
            mut response,
            total,
            downloaded,
        } => match response.chunk().await {
            Ok(Some(chunk)) => {
                let downloaded = downloaded + chunk.len() as u64;

                let percentage = (downloaded as f32 / total as f32) * 100.0;

                (
                    Some((id, Progress::Advanced(percentage))),
                    State::Downloading {
                        response,
                        total,
                        downloaded,
                    },
                )
            }
            Ok(None) => (Some((id, Progress::Finished)), State::Finished),
            Err(_) => (Some((id, Progress::Errored)), State::Finished),
        },
        State::Finished => {
            // We do not let the stream die, as it would start a
            // new download repeatedly if the user is not careful
            // in case of errors.
            let _: () = iced::futures::future::pending().await;

            unreachable!()
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
        response: reqwest::Response,
        total: u64,
        downloaded: u64,
    },
    Finished,
}
