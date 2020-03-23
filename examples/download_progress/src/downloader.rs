use iced_futures::futures;

// Just a little utility function
pub fn file<T: ToString>(url: T) -> iced::Subscription<DownloadMessage> {
    iced::Subscription::from_recipe(Downloader {
        url: url.to_string(),
    })
}

pub struct Downloader {
    url: String,
}

// Make sure iced can use our download stream
impl<H, I> iced_native::subscription::Recipe<H, I> for Downloader
where
    H: std::hash::Hasher,
{
    type Output = DownloadMessage;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        use isahc::prelude::*;

        Box::pin(futures::stream::unfold(
            DownloadState::Ready(self.url),
            |state| async move {
                match state {
                    DownloadState::Ready(url) => {
                        let resp = Request::get(&url)
                            .metrics(true)
                            .body(())
                            .unwrap()
                            .send_async()
                            .await
                            .unwrap();
                        let metrics = resp.metrics().unwrap().clone();
                        // If you actually want to download:
                        /*let file = async_std::fs::File::create("download.bin")
                        .await
                        .unwrap();*/

                        async_std::task::spawn(async_std::io::copy(
                            resp.into_body(),
                            async_std::io::sink(), //file
                        ));

                        Some((
                            DownloadMessage::DownloadStarted,
                            DownloadState::Downloading(metrics),
                        ))
                    }
                    DownloadState::Downloading(metrics) => {
                        async_std::task::sleep(
                            std::time::Duration::from_millis(100),
                        )
                        .await;

                        let percentage = metrics.download_progress().0 * 100
                            / metrics.download_progress().1;

                        if percentage == 100 {
                            Some((
                                DownloadMessage::Done,
                                DownloadState::Finished,
                            ))
                        } else {
                            Some((
                                DownloadMessage::Downloading(percentage),
                                DownloadState::Downloading(metrics),
                            ))
                        }
                    }
                    DownloadState::Finished => None,
                }
            },
        ))
    }
}

#[derive(Debug)]
pub enum DownloadMessage {
    DownloadStarted,
    Downloading(u64),
    Done,
}

pub enum DownloadState {
    Ready(String),
    Downloading(isahc::Metrics),
    Finished,
}
