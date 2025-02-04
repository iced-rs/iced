use iced::highlighter;
use iced::time::{self, milliseconds};
use iced::widget::{
    self, center_x, horizontal_space, hover, image, markdown, pop, right, row,
    scrollable, text_editor, toggler,
};
use iced::{Element, Fill, Font, Subscription, Task, Theme};

use tokio::task;

use std::collections::HashMap;
use std::io;
use std::sync::Arc;

pub fn main() -> iced::Result {
    iced::application("Markdown - Iced", Markdown::update, Markdown::view)
        .subscription(Markdown::subscription)
        .theme(Markdown::theme)
        .run_with(Markdown::new)
}

struct Markdown {
    content: text_editor::Content,
    images: HashMap<markdown::Url, Image>,
    mode: Mode,
    theme: Theme,
}

enum Mode {
    Preview(Vec<markdown::Item>),
    Stream {
        pending: String,
        parsed: markdown::Content,
    },
}

enum Image {
    Loading,
    Ready(image::Handle),
    #[allow(dead_code)]
    Errored(Error),
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    LinkClicked(markdown::Url),
    ImageShown(markdown::Url),
    ImageDownloaded(markdown::Url, Result<image::Handle, Error>),
    ToggleStream(bool),
    NextToken,
}

impl Markdown {
    fn new() -> (Self, Task<Message>) {
        const INITIAL_CONTENT: &str = include_str!("../overview.md");

        let theme = Theme::TokyoNight;

        (
            Self {
                content: text_editor::Content::with_text(INITIAL_CONTENT),
                images: HashMap::new(),
                mode: Mode::Preview(markdown::parse(INITIAL_CONTENT).collect()),
                theme,
            },
            widget::focus_next(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Edit(action) => {
                let is_edit = action.is_edit();

                self.content.perform(action);

                if is_edit {
                    self.mode = Mode::Preview(
                        markdown::parse(&self.content.text()).collect(),
                    );
                }

                Task::none()
            }
            Message::LinkClicked(link) => {
                let _ = open::that_in_background(link.to_string());

                Task::none()
            }
            Message::ImageShown(url) => {
                if self.images.contains_key(&url) {
                    return Task::none();
                }

                let _ = self.images.insert(url.clone(), Image::Loading);

                Task::perform(download_image(url.clone()), move |result| {
                    Message::ImageDownloaded(url.clone(), result)
                })
            }
            Message::ImageDownloaded(url, result) => {
                let _ = self.images.insert(
                    url,
                    result.map(Image::Ready).unwrap_or_else(Image::Errored),
                );

                Task::none()
            }
            Message::ToggleStream(enable_stream) => {
                if enable_stream {
                    self.mode = Mode::Stream {
                        pending: self.content.text(),
                        parsed: markdown::Content::new(),
                    };

                    scrollable::snap_to(
                        "preview",
                        scrollable::RelativeOffset::END,
                    )
                } else {
                    self.mode = Mode::Preview(
                        markdown::parse(&self.content.text()).collect(),
                    );

                    Task::none()
                }
            }
            Message::NextToken => {
                match &mut self.mode {
                    Mode::Preview(_) => {}
                    Mode::Stream { pending, parsed } => {
                        if pending.is_empty() {
                            self.mode = Mode::Preview(parsed.items().to_vec());
                        } else {
                            let mut tokens = pending.split(' ');

                            if let Some(token) = tokens.next() {
                                parsed.push_str(&format!("{token} "));
                            }

                            *pending = tokens.collect::<Vec<_>>().join(" ");
                        }
                    }
                }

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let editor = text_editor(&self.content)
            .placeholder("Type your Markdown here...")
            .on_action(Message::Edit)
            .height(Fill)
            .padding(10)
            .font(Font::MONOSPACE)
            .highlight("markdown", highlighter::Theme::Base16Ocean);

        let items = match &self.mode {
            Mode::Preview(items) => items.as_slice(),
            Mode::Stream { parsed, .. } => parsed.items(),
        };

        let preview = markdown::view_with(
            &MarkdownViewer {
                images: &self.images,
            },
            &self.theme,
            items,
        );

        row![
            editor,
            hover(
                scrollable(preview)
                    .spacing(10)
                    .width(Fill)
                    .height(Fill)
                    .id("preview"),
                right(
                    toggler(matches!(self.mode, Mode::Stream { .. }))
                        .label("Stream")
                        .on_toggle(Message::ToggleStream)
                )
                .padding([0, 20])
            )
        ]
        .spacing(10)
        .padding(10)
        .into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn subscription(&self) -> Subscription<Message> {
        match self.mode {
            Mode::Preview(_) => Subscription::none(),
            Mode::Stream { .. } => {
                time::every(milliseconds(10)).map(|_| Message::NextToken)
            }
        }
    }
}

struct MarkdownViewer<'a> {
    images: &'a HashMap<markdown::Url, Image>,
}

impl<'a> markdown::Viewer<'a, Message> for MarkdownViewer<'a> {
    fn on_link_clicked(url: markdown::Url) -> Message {
        Message::LinkClicked(url)
    }

    fn image(
        &self,
        _settings: markdown::Settings,
        _title: &markdown::Text,
        url: &'a markdown::Url,
    ) -> Element<'a, Message> {
        if let Some(Image::Ready(handle)) = self.images.get(url) {
            center_x(image(handle)).into()
        } else {
            pop(horizontal_space().width(0))
                .key(url.as_str())
                .on_show(|_size| Message::ImageShown(url.clone()))
                .into()
        }
    }
}

async fn download_image(url: markdown::Url) -> Result<image::Handle, Error> {
    use std::io;
    use tokio::task;

    let client = reqwest::Client::new();

    let bytes = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let image = task::spawn_blocking(move || {
        Ok::<_, Error>(
            ::image::ImageReader::new(io::Cursor::new(bytes))
                .with_guessed_format()?
                .decode()?
                .to_rgba8(),
        )
    })
    .await??;

    Ok(image::Handle::from_rgba(
        image.width(),
        image.height(),
        image.into_raw(),
    ))
}

#[derive(Debug, Clone)]
pub enum Error {
    RequestFailed(Arc<reqwest::Error>),
    IOFailed(Arc<io::Error>),
    JoinFailed(Arc<task::JoinError>),
    ImageDecodingFailed(Arc<::image::ImageError>),
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

impl From<::image::ImageError> for Error {
    fn from(error: ::image::ImageError) -> Self {
        Self::ImageDecodingFailed(Arc::new(error))
    }
}
