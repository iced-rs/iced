//! A simple gallery that displays the daily featured images of Civitai.
//!
//! Showcases lazy loading of images in the background, as well as
//! some smooth animations.
mod civitai;

use crate::civitai::{Error, Id, Image, Rgba, Size};

use iced::animation;
use iced::time::{Instant, milliseconds};
use iced::widget::{
    button, container, float, grid, horizontal_space, image, mouse_area,
    opaque, pop, scrollable, stack,
};
use iced::window;
use iced::{
    Animation, Color, ContentFit, Element, Fill, Function, Shadow,
    Subscription, Task, Theme, color,
};

use std::collections::HashMap;

fn main() -> iced::Result {
    iced::application::timed(
        Gallery::new,
        Gallery::update,
        Gallery::subscription,
        Gallery::view,
    )
    .window_size((Preview::WIDTH as f32 * 4.0, Preview::HEIGHT as f32 * 2.5))
    .theme(Gallery::theme)
    .run()
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
    BlurhashDecoded(Id, civitai::Blurhash),
    Open(Id),
    Close,
    Animate,
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
            .any(|preview| preview.is_animating(self.now))
            || self.viewer.is_animating(self.now);

        if is_animating {
            window::frames().map(|_| Message::Animate)
        } else {
            Subscription::none()
        }
    }

    pub fn update(&mut self, message: Message, now: Instant) -> Task<Message> {
        self.now = now;

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

                Task::sip(
                    image.download(Size::Thumbnail {
                        width: Preview::WIDTH,
                        height: Preview::HEIGHT,
                    }),
                    Message::BlurhashDecoded.with(id),
                    Message::ThumbnailDownloaded.with(id),
                )
            }
            Message::ImageDownloaded(Ok(rgba)) => {
                self.viewer.show(rgba, self.now);

                Task::none()
            }
            Message::ThumbnailDownloaded(id, Ok(rgba)) => {
                let thumbnail = if let Some(preview) = self.previews.remove(&id)
                {
                    preview.load(rgba, self.now)
                } else {
                    Preview::ready(rgba, self.now)
                };

                let _ = self.previews.insert(id, thumbnail);

                Task::none()
            }
            Message::ThumbnailHovered(id, is_hovered) => {
                if let Some(preview) = self.previews.get_mut(&id) {
                    preview.toggle_zoom(is_hovered, self.now);
                }

                Task::none()
            }
            Message::BlurhashDecoded(id, blurhash) => {
                if !self.previews.contains_key(&id) {
                    let _ = self
                        .previews
                        .insert(id, Preview::loading(blurhash.rgba, self.now));
                }

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

                self.viewer.open(self.now);

                Task::perform(
                    image.download(Size::Original),
                    Message::ImageDownloaded,
                )
            }
            Message::Close => {
                self.viewer.close(self.now);

                Task::none()
            }
            Message::Animate => Task::none(),
            Message::ImagesListed(Err(error))
            | Message::ImageDownloaded(Err(error))
            | Message::ThumbnailDownloaded(_, Err(error)) => {
                dbg!(error);

                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let images = self
            .images
            .iter()
            .map(|image| card(image, self.previews.get(&image.id), self.now))
            .chain((self.images.len()..=Image::LIMIT).map(|_| placeholder()));

        let gallery = grid(images)
            .fluid(Preview::WIDTH)
            .height(grid::aspect_ratio(Preview::WIDTH, Preview::HEIGHT))
            .spacing(10);

        let content = container(scrollable(gallery).spacing(10)).padding(10);
        let viewer = self.viewer.view(self.now);

        stack![content, viewer].into()
    }
}

fn card<'a>(
    metadata: &'a Image,
    preview: Option<&'a Preview>,
    now: Instant,
) -> Element<'a, Message> {
    let image = if let Some(preview) = preview {
        let thumbnail: Element<'_, _> =
            if let Preview::Ready { thumbnail, .. } = &preview {
                float(
                    image(&thumbnail.handle)
                        .width(Fill)
                        .content_fit(ContentFit::Cover)
                        .opacity(thumbnail.fade_in.interpolate(0.0, 1.0, now)),
                )
                .scale(thumbnail.zoom.interpolate(1.0, 1.1, now))
                .translate(move |bounds, viewport| {
                    bounds.zoom(1.1).offset(&viewport.shrink(10))
                        * thumbnail.zoom.interpolate(0.0, 1.0, now)
                })
                .style(move |_theme| float::Style {
                    shadow: Shadow {
                        color: Color::BLACK.scale_alpha(
                            thumbnail.zoom.interpolate(0.0, 1.0, now),
                        ),
                        blur_radius: thumbnail.zoom.interpolate(0.0, 20.0, now),
                        ..Shadow::default()
                    },
                    ..float::Style::default()
                })
                .into()
            } else {
                horizontal_space().into()
            };

        if let Some(blurhash) = preview.blurhash(now) {
            let blurhash = image(&blurhash.handle)
                .width(Fill)
                .content_fit(ContentFit::Cover)
                .opacity(blurhash.fade_in.interpolate(0.0, 1.0, now));

            stack![blurhash, thumbnail].into()
        } else {
            thumbnail
        }
    } else {
        horizontal_space().into()
    };

    let card = mouse_area(container(image).style(container::dark))
        .on_enter(Message::ThumbnailHovered(metadata.id, true))
        .on_exit(Message::ThumbnailHovered(metadata.id, false));

    if let Some(preview) = preview {
        let is_thumbnail = matches!(preview, Preview::Ready { .. });

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
    container(horizontal_space()).style(container::dark).into()
}

enum Preview {
    Loading {
        blurhash: Blurhash,
    },
    Ready {
        blurhash: Option<Blurhash>,
        thumbnail: Thumbnail,
    },
}

struct Blurhash {
    handle: image::Handle,
    fade_in: Animation<bool>,
}

struct Thumbnail {
    handle: image::Handle,
    fade_in: Animation<bool>,
    zoom: Animation<bool>,
}

impl Preview {
    const WIDTH: u32 = 320;
    const HEIGHT: u32 = 410;

    fn loading(rgba: Rgba, now: Instant) -> Self {
        Self::Loading {
            blurhash: Blurhash {
                fade_in: Animation::new(false)
                    .duration(milliseconds(700))
                    .easing(animation::Easing::EaseIn)
                    .go(true, now),
                handle: image::Handle::from_rgba(
                    rgba.width,
                    rgba.height,
                    rgba.pixels,
                ),
            },
        }
    }

    fn ready(rgba: Rgba, now: Instant) -> Self {
        Self::Ready {
            blurhash: None,
            thumbnail: Thumbnail::new(rgba, now),
        }
    }

    fn load(self, rgba: Rgba, now: Instant) -> Self {
        let Self::Loading { blurhash } = self else {
            return self;
        };

        Self::Ready {
            blurhash: Some(blurhash),
            thumbnail: Thumbnail::new(rgba, now),
        }
    }

    fn toggle_zoom(&mut self, enabled: bool, now: Instant) {
        if let Self::Ready { thumbnail, .. } = self {
            thumbnail.zoom.go_mut(enabled, now);
        }
    }

    fn is_animating(&self, now: Instant) -> bool {
        match &self {
            Self::Loading { blurhash } => blurhash.fade_in.is_animating(now),
            Self::Ready { thumbnail, .. } => {
                thumbnail.fade_in.is_animating(now)
                    || thumbnail.zoom.is_animating(now)
            }
        }
    }

    fn blurhash(&self, now: Instant) -> Option<&Blurhash> {
        match self {
            Self::Loading { blurhash, .. } => Some(blurhash),
            Self::Ready {
                blurhash: Some(blurhash),
                thumbnail,
                ..
            } if thumbnail.fade_in.is_animating(now) => Some(blurhash),
            Self::Ready { .. } => None,
        }
    }
}

impl Thumbnail {
    pub fn new(rgba: Rgba, now: Instant) -> Self {
        Self {
            handle: image::Handle::from_rgba(
                rgba.width,
                rgba.height,
                rgba.pixels,
            ),
            fade_in: Animation::new(false).slow().go(true, now),
            zoom: Animation::new(false)
                .quick()
                .easing(animation::Easing::EaseInOut),
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

    fn open(&mut self, now: Instant) {
        self.image = None;
        self.background_fade_in.go_mut(true, now);
    }

    fn show(&mut self, rgba: Rgba, now: Instant) {
        self.image = Some(image::Handle::from_rgba(
            rgba.width,
            rgba.height,
            rgba.pixels,
        ));
        self.background_fade_in.go_mut(true, now);
        self.image_fade_in.go_mut(true, now);
    }

    fn close(&mut self, now: Instant) {
        self.background_fade_in.go_mut(false, now);
        self.image_fade_in.go_mut(false, now);
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
