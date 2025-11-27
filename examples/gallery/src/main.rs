//! A simple gallery that displays the daily featured images of Civitai.
//!
//! Showcases lazy loading of images in the background, as well as
//! some smooth animations.
mod civitai;

use crate::civitai::{Bytes, Error, Id, Image, Rgba, Size};

use iced::animation;
use iced::border;
use iced::time::{Instant, milliseconds};
use iced::widget::{
    button, container, float, grid, image, mouse_area, opaque, scrollable,
    sensor, space, stack,
};
use iced::window;
use iced::{
    Animation, Color, ContentFit, Element, Fill, Function, Shadow,
    Subscription, Task, Theme, color,
};

use std::collections::{HashMap, HashSet};

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
    visible: HashSet<Id>,
    downloaded: HashSet<Id>,
    viewer: Viewer,
    now: Instant,
}

#[derive(Debug, Clone)]
enum Message {
    ImagesListed(Result<Vec<Image>, Error>),
    ImagePoppedIn(Id),
    ImagePoppedOut(Id),
    ImageDownloaded(Result<image::Allocation, Error>),
    ThumbnailDownloaded(Id, Result<Bytes, Error>),
    ThumbnailAllocated(Id, Result<image::Allocation, image::Error>),
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
                visible: HashSet::new(),
                downloaded: HashSet::new(),
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

                let _ = self.visible.insert(id);

                if self.downloaded.contains(&id) {
                    let Some(Preview::Ready {
                        thumbnail,
                        blurhash,
                    }) = self.previews.get_mut(&id)
                    else {
                        return Task::none();
                    };

                    if let Some(blurhash) = blurhash {
                        blurhash.show(now);
                    }

                    return to_rgba(thumbnail.bytes.clone())
                        .then(image::allocate)
                        .map(Message::ThumbnailAllocated.with(id));
                }

                let _ = self.downloaded.insert(id);

                Task::sip(
                    image.download(Size::Thumbnail {
                        width: Preview::WIDTH,
                        height: Preview::HEIGHT,
                    }),
                    Message::BlurhashDecoded.with(id),
                    Message::ThumbnailDownloaded.with(id),
                )
            }
            Message::ImagePoppedOut(id) => {
                let _ = self.visible.remove(&id);

                if let Some(Preview::Ready {
                    thumbnail,
                    blurhash,
                }) = self.previews.get_mut(&id)
                {
                    thumbnail.reset();

                    if let Some(blurhash) = blurhash {
                        blurhash.reset();
                    }
                }

                Task::none()
            }
            Message::ImageDownloaded(Ok(allocation)) => {
                self.viewer.show(allocation, self.now);

                Task::none()
            }
            Message::ThumbnailDownloaded(id, Ok(bytes)) => {
                let preview = if let Some(preview) = self.previews.remove(&id) {
                    preview.load(bytes.clone())
                } else {
                    Preview::ready(bytes.clone())
                };

                let _ = self.previews.insert(id, preview);

                to_rgba(bytes)
                    .then(image::allocate)
                    .map(Message::ThumbnailAllocated.with(id))
            }
            Message::ThumbnailAllocated(id, Ok(allocation)) => {
                if !self.visible.contains(&id) {
                    return Task::none();
                }

                let Some(Preview::Ready { thumbnail, .. }) =
                    self.previews.get_mut(&id)
                else {
                    return Task::none();
                };

                thumbnail.show(allocation, now);

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

                Task::future(image.download(Size::Original))
                    .and_then(|bytes| {
                        image::allocate(image::Handle::from_bytes(bytes))
                            .map_err(|_| Error::ImageDecodingFailed)
                    })
                    .map(Message::ImageDownloaded)
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
            Message::ThumbnailAllocated(_, Err(error)) => {
                dbg!(error);

                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let images = self
            .images
            .iter()
            .map(|image| {
                card(
                    image,
                    if self.visible.contains(&image.id) {
                        self.previews.get(&image.id)
                    } else {
                        None
                    },
                    self.now,
                )
            })
            .chain(
                if self.images.is_empty() {
                    0..Image::LIMIT
                } else {
                    0..0
                }
                .map(|_| placeholder()),
            );

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
            if let Preview::Ready { thumbnail, .. } = &preview
                && let Some(allocation) = &thumbnail.allocation
            {
                float(
                    image(allocation.handle())
                        .width(Fill)
                        .content_fit(ContentFit::Cover)
                        .opacity(thumbnail.fade_in.interpolate(0.0, 1.0, now))
                        .border_radius(BORDER_RADIUS),
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
                    shadow_border_radius: border::radius(BORDER_RADIUS),
                })
                .into()
            } else {
                space::horizontal().into()
            };

        if let Some(blurhash) = preview.blurhash(now) {
            let blurhash = image(&blurhash.handle)
                .width(Fill)
                .content_fit(ContentFit::Cover)
                .opacity(blurhash.fade_in.interpolate(0.0, 1.0, now))
                .border_radius(BORDER_RADIUS);

            stack![blurhash, thumbnail].into()
        } else {
            thumbnail
        }
    } else {
        space::horizontal().into()
    };

    let card = mouse_area(container(image).style(rounded))
        .on_enter(Message::ThumbnailHovered(metadata.id, true))
        .on_exit(Message::ThumbnailHovered(metadata.id, false));

    let card: Element<'_, _> = if let Some(preview) = preview {
        let is_thumbnail = matches!(preview, Preview::Ready { .. });

        button(card)
            .on_press_maybe(is_thumbnail.then_some(Message::Open(metadata.id)))
            .padding(0)
            .style(button::text)
            .into()
    } else {
        card.into()
    };

    sensor(card)
        .on_show(|_| Message::ImagePoppedIn(metadata.id))
        .on_hide(Message::ImagePoppedOut(metadata.id))
        .into()
}

fn placeholder<'a>() -> Element<'a, Message> {
    container(space()).style(rounded).into()
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

impl Blurhash {
    pub fn show(&mut self, now: Instant) {
        self.fade_in.go_mut(true, now);
    }

    pub fn reset(&mut self) {
        self.fade_in = Animation::new(false)
            .easing(animation::Easing::EaseIn)
            .very_quick();
    }
}

struct Thumbnail {
    bytes: Bytes,
    allocation: Option<image::Allocation>,
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

    fn ready(bytes: Bytes) -> Self {
        Self::Ready {
            blurhash: None,
            thumbnail: Thumbnail::new(bytes),
        }
    }

    fn load(self, bytes: Bytes) -> Self {
        let Self::Loading { blurhash } = self else {
            return self;
        };

        Self::Ready {
            blurhash: Some(blurhash),
            thumbnail: Thumbnail::new(bytes),
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
            Self::Ready {
                thumbnail,
                blurhash,
            } => {
                thumbnail.fade_in.is_animating(now)
                    || thumbnail.zoom.is_animating(now)
                    || blurhash.as_ref().is_some_and(|blurhash| {
                        blurhash.fade_in.is_animating(now)
                    })
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
            } if !thumbnail.fade_in.value()
                || thumbnail.fade_in.is_animating(now) =>
            {
                Some(blurhash)
            }
            Self::Ready { .. } => None,
        }
    }
}

impl Thumbnail {
    pub fn new(bytes: Bytes) -> Self {
        Self {
            bytes,
            allocation: None,
            fade_in: Animation::new(false)
                .easing(animation::Easing::EaseIn)
                .slow(),
            zoom: Animation::new(false)
                .quick()
                .easing(animation::Easing::EaseInOut),
        }
    }

    pub fn reset(&mut self) {
        self.allocation = None;
        self.fade_in = Animation::new(false)
            .easing(animation::Easing::EaseIn)
            .quick();
    }

    pub fn show(&mut self, allocation: image::Allocation, now: Instant) {
        self.allocation = Some(allocation);
        self.fade_in.go_mut(true, now);
    }
}

struct Viewer {
    image: Option<image::Allocation>,
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

    fn show(&mut self, allocation: image::Allocation, now: Instant) {
        self.image = Some(allocation);
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

    fn view(&self, now: Instant) -> Option<Element<'_, Message>> {
        let opacity = self.background_fade_in.interpolate(0.0, 0.8, now);

        if opacity <= 0.0 {
            return None;
        }

        let image = self.image.as_ref().map(|allocation| {
            image(allocation.handle())
                .width(Fill)
                .height(Fill)
                .opacity(self.image_fade_in.interpolate(0.0, 1.0, now))
                .scale(self.image_fade_in.interpolate(1.5, 1.0, now))
        });

        Some(opaque(
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
        ))
    }
}

fn to_rgba(bytes: Bytes) -> Task<image::Handle> {
    Task::future(async move {
        tokio::task::spawn_blocking(move || {
            match ::image::load_from_memory(bytes.as_slice()) {
                Ok(image) => {
                    let rgba = image.to_rgba8();

                    image::Handle::from_rgba(
                        rgba.width(),
                        rgba.height(),
                        rgba.into_raw(),
                    )
                }
                _ => image::Handle::from_bytes(bytes),
            }
        })
        .await
        .unwrap()
    })
}

fn rounded(theme: &Theme) -> container::Style {
    container::dark(theme).border(border::rounded(BORDER_RADIUS))
}

const BORDER_RADIUS: u32 = 10;
