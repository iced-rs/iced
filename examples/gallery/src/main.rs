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

fn main() -> iced::Result {
    iced::application("Gallery - Iced", Gallery::update, Gallery::view)
        .subscription(Gallery::subscription)
        .theme(Gallery::theme)
        .run_with(Gallery::new)
}

struct Gallery {
    images: Vec<Image>,
    thumbnails: HashMap<Id, Thumbnail>,
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
    Open(Id),
    Close,
    Animate(Instant),
}

impl Gallery {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                images: Vec::new(),
                thumbnails: HashMap::new(),
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
            .thumbnails
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

                Task::perform(image.download(Size::Thumbnail), move |result| {
                    Message::ThumbnailDownloaded(id, result)
                })
            }
            Message::ImageDownloaded(Ok(rgba)) => {
                self.viewer.show(rgba);

                Task::none()
            }
            Message::ThumbnailDownloaded(id, Ok(rgba)) => {
                let thumbnail = Thumbnail::new(rgba);
                let _ = self.thumbnails.insert(id, thumbnail);

                Task::none()
            }
            Message::ThumbnailHovered(id, is_hovered) => {
                if let Some(thumbnail) = self.thumbnails.get_mut(&id) {
                    thumbnail.zoom.go_mut(is_hovered);
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
            | Message::ThumbnailDownloaded(_, Err(error)) => {
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
                card(image, self.thumbnails.get(&image.id), self.now)
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
    thumbnail: Option<&'a Thumbnail>,
    now: Instant,
) -> Element<'a, Message> {
    let image: Element<'_, _> = if let Some(thumbnail) = thumbnail {
        image(&thumbnail.handle)
            .width(Fill)
            .height(Fill)
            .content_fit(ContentFit::Cover)
            .opacity(thumbnail.fade_in.interpolate(0.0, 1.0, now))
            .scale(thumbnail.zoom.interpolate(1.0, 1.1, now))
            .into()
    } else {
        horizontal_space().into()
    };

    let card = mouse_area(
        container(image)
            .width(Thumbnail::WIDTH)
            .height(Thumbnail::HEIGHT)
            .style(container::dark),
    )
    .on_enter(Message::ThumbnailHovered(metadata.id, true))
    .on_exit(Message::ThumbnailHovered(metadata.id, false));

    if thumbnail.is_some() {
        button(card)
            .on_press(Message::Open(metadata.id))
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
        .width(Thumbnail::WIDTH)
        .height(Thumbnail::HEIGHT)
        .style(container::dark)
        .into()
}

struct Thumbnail {
    handle: image::Handle,
    fade_in: Animation<bool>,
    zoom: Animation<bool>,
}

impl Thumbnail {
    const WIDTH: u16 = 320;
    const HEIGHT: u16 = 410;

    fn new(rgba: Rgba) -> Self {
        Self {
            handle: image::Handle::from_rgba(
                rgba.width,
                rgba.height,
                rgba.pixels,
            ),
            fade_in: Animation::new(false).slow().go(true),
            zoom: Animation::new(false)
                .quick()
                .easing(animation::Easing::EaseInOut),
        }
    }

    fn is_animating(&self, now: Instant) -> bool {
        self.fade_in.is_animating(now) || self.zoom.is_animating(now)
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
