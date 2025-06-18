mod icon;

use iced::animation;
use iced::clipboard;
use iced::highlighter;
use iced::time::{self, Instant, milliseconds};
use iced::widget::{
    self, button, center_x, container, horizontal_space, hover, image,
    markdown, pop, right, row, scrollable, text_editor, toggler,
};
use iced::window;
use iced::{
    Animation, Element, Fill, Font, Function, Subscription, Task, Theme,
};

use std::collections::HashMap;
use std::io;
use std::sync::Arc;

pub fn main() -> iced::Result {
    iced::application::timed(
        Markdown::new,
        Markdown::update,
        Markdown::subscription,
        Markdown::view,
    )
    .font(icon::FONT)
    .theme(Markdown::theme)
    .run()
}

struct Markdown {
    content: markdown::Content,
    raw: text_editor::Content,
    images: HashMap<markdown::Url, Image>,
    mode: Mode,
    theme: Theme,
    now: Instant,
}

enum Mode {
    Preview,
    Stream { pending: String },
}

enum Image {
    Loading,
    Ready {
        handle: image::Handle,
        fade_in: Animation<bool>,
    },
    #[allow(dead_code)]
    Errored(Error),
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    Copy(String),
    LinkClicked(markdown::Url),
    ImageShown(markdown::Url),
    ImageDownloaded(markdown::Url, Result<image::Handle, Error>),
    ToggleStream(bool),
    NextToken,
    Tick,
}

impl Markdown {
    fn new() -> (Self, Task<Message>) {
        const INITIAL_CONTENT: &str = include_str!("../overview.md");

        (
            Self {
                content: markdown::Content::parse(INITIAL_CONTENT),
                raw: text_editor::Content::with_text(INITIAL_CONTENT),
                images: HashMap::new(),
                mode: Mode::Preview,
                theme: Theme::TokyoNight,
                now: Instant::now(),
            },
            widget::focus_next(),
        )
    }

    fn update(&mut self, message: Message, now: Instant) -> Task<Message> {
        self.now = now;

        match message {
            Message::Edit(action) => {
                let is_edit = action.is_edit();

                self.raw.perform(action);

                if is_edit {
                    self.content = markdown::Content::parse(&self.raw.text());
                    self.mode = Mode::Preview;
                }

                Task::none()
            }
            Message::Copy(content) => clipboard::write(content),
            Message::LinkClicked(link) => {
                let _ = open::that_in_background(link.to_string());

                Task::none()
            }
            Message::ImageShown(url) => {
                if self.images.contains_key(&url) {
                    return Task::none();
                }

                let _ = self.images.insert(url.clone(), Image::Loading);

                Task::perform(
                    download_image(url.clone()),
                    Message::ImageDownloaded.with(url),
                )
            }
            Message::ImageDownloaded(url, result) => {
                let _ = self.images.insert(
                    url,
                    result
                        .map(|handle| Image::Ready {
                            handle,
                            fade_in: Animation::new(false)
                                .quick()
                                .easing(animation::Easing::EaseInOut)
                                .go(true, self.now),
                        })
                        .unwrap_or_else(Image::Errored),
                );

                Task::none()
            }
            Message::ToggleStream(enable_stream) => {
                if enable_stream {
                    self.content = markdown::Content::new();

                    self.mode = Mode::Stream {
                        pending: self.raw.text(),
                    };

                    scrollable::snap_to(
                        "preview",
                        scrollable::RelativeOffset::END,
                    )
                } else {
                    self.mode = Mode::Preview;

                    Task::none()
                }
            }
            Message::NextToken => {
                match &mut self.mode {
                    Mode::Preview => {}
                    Mode::Stream { pending } => {
                        if pending.is_empty() {
                            self.mode = Mode::Preview;
                        } else {
                            let mut tokens = pending.split(' ');

                            if let Some(token) = tokens.next() {
                                self.content.push_str(&format!("{token} "));
                            }

                            *pending = tokens.collect::<Vec<_>>().join(" ");
                        }
                    }
                }

                Task::none()
            }
            Message::Tick => Task::none(),
        }
    }

    fn view(&self) -> Element<Message> {
        let editor = text_editor(&self.raw)
            .placeholder("Type your Markdown here...")
            .on_action(Message::Edit)
            .height(Fill)
            .padding(10)
            .font(Font::MONOSPACE)
            .highlight("markdown", highlighter::Theme::Base16Ocean);

        let preview = markdown::view_with(
            self.content.items(),
            &self.theme,
            &CustomViewer {
                images: &self.images,
                now: self.now,
            },
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
        let listen_stream = match self.mode {
            Mode::Preview => Subscription::none(),
            Mode::Stream { .. } => {
                time::every(milliseconds(10)).map(|_| Message::NextToken)
            }
        };

        let animate = {
            let is_animating = self.images.values().any(|image| match image {
                Image::Ready { fade_in, .. } => fade_in.is_animating(self.now),
                _ => false,
            });

            if is_animating {
                window::frames().map(|_| Message::Tick)
            } else {
                Subscription::none()
            }
        };

        Subscription::batch([listen_stream, animate])
    }
}

struct CustomViewer<'a> {
    images: &'a HashMap<markdown::Url, Image>,
    now: Instant,
}

impl<'a> markdown::Viewer<'a, Message> for CustomViewer<'a> {
    fn on_link_click(url: markdown::Url) -> Message {
        Message::LinkClicked(url)
    }

    fn image(
        &self,
        _settings: markdown::Settings,
        url: &'a markdown::Url,
        _title: &'a str,
        _alt: &markdown::Text,
    ) -> Element<'a, Message> {
        if let Some(Image::Ready { handle, fade_in }) = self.images.get(url) {
            center_x(
                image(handle)
                    .opacity(fade_in.interpolate(0.0, 1.0, self.now))
                    .scale(fade_in.interpolate(1.2, 1.0, self.now)),
            )
            .into()
        } else {
            pop(horizontal_space())
                .key_ref(url.as_str())
                .delay(milliseconds(500))
                .on_show(|_size| Message::ImageShown(url.clone()))
                .into()
        }
    }

    fn code_block(
        &self,
        settings: markdown::Settings,
        _language: Option<&'a str>,
        code: &'a str,
        lines: &'a [markdown::Text],
    ) -> Element<'a, Message> {
        let code_block =
            markdown::code_block(settings, lines, Message::LinkClicked);

        let copy = button(icon::copy().size(12))
            .padding(2)
            .on_press_with(|| Message::Copy(code.to_owned()))
            .style(button::text);

        hover(
            code_block,
            right(container(copy).style(container::dark))
                .padding(settings.spacing / 2),
        )
    }
}

async fn download_image(url: markdown::Url) -> Result<image::Handle, Error> {
    use std::io;
    use tokio::task;

    println!("Trying to download image: {url}");

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
    JoinFailed(Arc<tokio::task::JoinError>),
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

impl From<tokio::task::JoinError> for Error {
    fn from(error: tokio::task::JoinError) -> Self {
        Self::JoinFailed(Arc::new(error))
    }
}

impl From<::image::ImageError> for Error {
    fn from(error: ::image::ImageError) -> Self {
        Self::ImageDecodingFailed(Arc::new(error))
    }
}
