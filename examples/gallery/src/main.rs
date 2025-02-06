//! A simple gallery that displays the daily featured images of Civitai.
//!
//! Showcases lazy loading of images in the background, as well as
//! some smooth animations.
mod civitai;

use crate::civitai::{Error, Id, Image, Rgba, Size};

use iced::animation;
use iced::time::Instant;
use iced::widget::{
    button, center_x, container, horizontal_space, image, mouse_area, opaque,
    pop, row, scrollable, stack,
};
use iced::window;
use iced::{
    color, Animation, ContentFit, Element, Fill, Subscription, Task, Theme,
};

use std::collections::HashMap;
use std::time::Duration;

fn main() -> iced::Result {
    iced::application("Gallery - Iced", Gallery::update, Gallery::view)
        .subscription(Gallery::subscription)
        .theme(Gallery::theme)
        .run_with(Gallery::new)
}

struct Gallery {
    images: Vec<Image>,
    previews: HashMap<Id, Preview>,
    viewer: Viewer,
    now: Instant,
}

#[derive(Debug, Clone)]
enum Message {
    ImagesListed(Result<Vec<Image>, Error>),
    ImagePoppedIn(Id),
    ImageDownloaded(Result<Rgba, Error>),
    ThumbnailDownloaded(Id, Result<Rgba, Error>),
    ThumbnailHovered(Id, bool),
    BlurhashDecoded(Id, Result<Rgba, Error>),
    Open(Id),
    Close,
    Animate(Instant),
}

impl Gallery {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                images: Vec::new(),
                previews: HashMap::new(),
                viewer: Viewer::new(),
                now: Instant::now(),
            },
            Task::perform(Image::list(), Message::ImagesListed),
        )
    }

    pub fn theme(&self) -> Theme {
        Theme::TokyoNight
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let is_animating = self
            .previews
            .values()
            .any(|thumbnail| thumbnail.is_animating(self.now))
            || self.viewer.is_animating(self.now);

        if is_animating {
            window::frames().map(Message::Animate)
        } else {
            Subscription::none()
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ImagesListed(Ok(images)) => {
                self.images = images;

                Task::none()
            }
            Message::ImagePoppedIn(id) => {
                let Some(image) = self
                    .images
                    .iter()
                    .find(|candidate| candidate.id == id)
                    .cloned()
                else {
                    return Task::none();
                };

                Task::batch(vec![
                    Task::perform(
                        image.clone().blurhash(
                            Preview::WIDTH as u32,
                            Preview::HEIGHT as u32,
                        ),
                        move |result| Message::BlurhashDecoded(id, result),
                    ),
                    Task::perform(
                        image.download(Size::Thumbnail {
                            width: Preview::WIDTH,
                        }),
                        move |result| Message::ThumbnailDownloaded(id, result),
                    ),
                ])
            }
            Message::ImageDownloaded(Ok(rgba)) => {
                self.viewer.show(rgba);

                Task::none()
            }
            Message::ThumbnailDownloaded(id, Ok(rgba)) => {
                let blurhash = match self.previews.remove(&id) {
                    Some(Preview::Blurhash(blurhash)) => Some(blurhash),
                    _ => None,
                };

                let _ = self
                    .previews
                    .insert(id, Preview::thumbnail(self.now, blurhash, rgba));

                Task::none()
            }
            Message::ThumbnailHovered(id, is_hovered) => {
                if let Some(Preview::Thumbnail { zoom, .. }) =
                    self.previews.get_mut(&id)
                {
                    zoom.go_mut(is_hovered);
                }

                Task::none()
            }
            Message::BlurhashDecoded(id, Ok(rgba)) => {
                let _ = self.previews.insert(id, Preview::blurhash(rgba));

                Task::none()
            }
            Message::Open(id) => {
                let Some(image) = self
                    .images
                    .iter()
                    .find(|candidate| candidate.id == id)
                    .cloned()
                else {
                    return Task::none();
                };

                self.viewer.open();

                Task::perform(
                    image.download(Size::Original),
                    Message::ImageDownloaded,
                )
            }
            Message::Close => {
                self.viewer.close();

                Task::none()
            }
            Message::Animate(now) => {
                self.now = now;

                Task::none()
            }
            Message::ImagesListed(Err(error))
            | Message::ImageDownloaded(Err(error))
            | Message::ThumbnailDownloaded(_, Err(error))
            | Message::BlurhashDecoded(_, Err(error)) => {
                dbg!(error);

                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let gallery = if self.images.is_empty() {
            row((0..=Image::LIMIT).map(|_| placeholder()))
        } else {
            row(self.images.iter().map(|image| {
                card(image, self.previews.get(&image.id), self.now)
            }))
        }
        .spacing(10)
        .wrap();

        let content =
            container(scrollable(center_x(gallery)).spacing(10)).padding(10);

        let viewer = self.viewer.view(self.now);

        stack![content, viewer].into()
    }
}

fn card<'a>(
    metadata: &'a Image,
    preview: Option<&'a Preview>,
    now: Instant,
) -> Element<'a, Message> {
    let image: Element<'_, _> = match preview {
        Some(Preview::Blurhash(Blurhash { handle, fade_in })) => image(handle)
            .width(Fill)
            .height(Fill)
            .content_fit(ContentFit::Cover)
            .opacity(fade_in.interpolate(0.0, Blurhash::MAX_OPACITY, now))
            .into(),
        // Blurhash still needs to fade all the way in
        Some(Preview::Thumbnail {
            blurhash: Some(blurhash),
            ..
        }) if blurhash.fade_in.is_animating(now) => image(&blurhash.handle)
            .width(Fill)
            .height(Fill)
            .content_fit(ContentFit::Cover)
            .opacity(blurhash.fade_in.interpolate(
                0.0,
                Blurhash::MAX_OPACITY,
                now,
            ))
            .into(),
        Some(Preview::Thumbnail {
            blurhash,
            thumbnail,
            fade_in,
            zoom,
        }) => stack![]
            // Transition between blurhash & thumbnail over the fade-in period
            .push_maybe(
                blurhash.as_ref().filter(|_| fade_in.is_animating(now)).map(
                    |blurhash| {
                        image(&blurhash.handle)
                            .width(Fill)
                            .height(Fill)
                            .content_fit(ContentFit::Cover)
                            .opacity(fade_in.interpolate(
                                Blurhash::MAX_OPACITY,
                                0.0,
                                now,
                            ))
                    },
                ),
            )
            .push(
                image(thumbnail)
                    .width(Fill)
                    .height(Fill)
                    .content_fit(ContentFit::Cover)
                    .opacity(fade_in.interpolate(0.0, 1.0, now))
                    .scale(zoom.interpolate(1.0, 1.1, now)),
            )
            .into(),
        None => horizontal_space().into(),
    };

    let card = mouse_area(
        container(image)
            .width(Preview::WIDTH)
            .height(Preview::HEIGHT)
            .style(container::dark),
    )
    .on_enter(Message::ThumbnailHovered(metadata.id, true))
    .on_exit(Message::ThumbnailHovered(metadata.id, false));

    if let Some(preview) = preview {
        let is_thumbnail = matches!(preview, Preview::Thumbnail { .. });

        button(card)
            .on_press_maybe(is_thumbnail.then_some(Message::Open(metadata.id)))
            .padding(0)
            .style(button::text)
            .into()
    } else {
        pop(card)
            .on_show(|_| Message::ImagePoppedIn(metadata.id))
            .into()
    }
}

fn placeholder<'a>() -> Element<'a, Message> {
    container(horizontal_space())
        .width(Preview::WIDTH)
        .height(Preview::HEIGHT)
        .style(container::dark)
        .into()
}

struct Blurhash {
    fade_in: Animation<bool>,
    handle: image::Handle,
}

impl Blurhash {
    const FADE_IN: Duration = Duration::from_millis(200);
    const MAX_OPACITY: f32 = 0.6;
}

enum Preview {
    Blurhash(Blurhash),
    Thumbnail {
        blurhash: Option<Blurhash>,
        thumbnail: image::Handle,
        fade_in: Animation<bool>,
        zoom: Animation<bool>,
    },
}

impl Preview {
    const WIDTH: u16 = 320;
    const HEIGHT: u16 = 410;

    fn blurhash(rgba: Rgba) -> Self {
        Self::Blurhash(Blurhash {
            fade_in: Animation::new(false).duration(Blurhash::FADE_IN).go(true),
            handle: image::Handle::from_rgba(
                rgba.width,
                rgba.height,
                rgba.pixels,
            ),
        })
    }

    fn thumbnail(now: Instant, blurhash: Option<Blurhash>, rgba: Rgba) -> Self {
        // Delay the thumbnail fade in until blurhash is fully
        // faded in itself
        let delay = blurhash
            .as_ref()
            .map(|blurhash| {
                Duration::from_secs_f32(blurhash.fade_in.interpolate(
                    0.0,
                    Blurhash::FADE_IN.as_secs_f32(),
                    now,
                ))
            })
            .unwrap_or_default();

        Self::Thumbnail {
            blurhash,
            thumbnail: image::Handle::from_rgba(
                rgba.width,
                rgba.height,
                rgba.pixels,
            ),
            fade_in: Animation::new(false).very_slow().delay(delay).go(true),
            zoom: Animation::new(false)
                .quick()
                .easing(animation::Easing::EaseInOut),
        }
    }

    fn is_animating(&self, now: Instant) -> bool {
        match self {
            Preview::Blurhash(Blurhash { fade_in, .. }) => {
                fade_in.is_animating(now)
            }
            Preview::Thumbnail { fade_in, zoom, .. } => {
                fade_in.is_animating(now) || zoom.is_animating(now)
            }
        }
    }
}

struct Viewer {
    image: Option<image::Handle>,
    background_fade_in: Animation<bool>,
    image_fade_in: Animation<bool>,
}

impl Viewer {
    fn new() -> Self {
        Self {
            image: None,
            background_fade_in: Animation::new(false)
                .very_slow()
                .easing(animation::Easing::EaseInOut),
            image_fade_in: Animation::new(false)
                .very_slow()
                .easing(animation::Easing::EaseInOut),
        }
    }

    fn open(&mut self) {
        self.image = None;
        self.background_fade_in.go_mut(true);
    }

    fn show(&mut self, rgba: Rgba) {
        self.image = Some(image::Handle::from_rgba(
            rgba.width,
            rgba.height,
            rgba.pixels,
        ));
        self.background_fade_in.go_mut(true);
        self.image_fade_in.go_mut(true);
    }

    fn close(&mut self) {
        self.background_fade_in.go_mut(false);
        self.image_fade_in.go_mut(false);
    }

    fn is_animating(&self, now: Instant) -> bool {
        self.background_fade_in.is_animating(now)
            || self.image_fade_in.is_animating(now)
    }

    fn view(&self, now: Instant) -> Element<'_, Message> {
        let opacity = self.background_fade_in.interpolate(0.0, 0.8, now);

        let image: Element<'_, _> = if let Some(handle) = &self.image {
            image(handle)
                .width(Fill)
                .height(Fill)
                .opacity(self.image_fade_in.interpolate(0.0, 1.0, now))
                .scale(self.image_fade_in.interpolate(1.5, 1.0, now))
                .into()
        } else {
            horizontal_space().into()
        };

        if opacity > 0.0 {
            opaque(
                mouse_area(
                    container(image)
                        .center(Fill)
                        .style(move |_theme| {
                            container::Style::default()
                                .background(color!(0x000000, opacity))
                        })
                        .padding(20),
                )
                .on_press(Message::Close),
            )
        } else {
            horizontal_space().into()
        }
    }
}
