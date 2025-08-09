//! Test your `iced` applications in headless mode.
//!
//! # Basic Usage
//! Let's assume we want to test [the classical counter interface].
//!
//! First, we will want to create a [`Simulator`] of our interface:
//!
//! ```rust,no_run
//! # struct Counter { value: i64 }
//! # impl Counter {
//! #    pub fn view(&self) -> iced_runtime::core::Element<(), iced_runtime::core::Theme, iced_renderer::Renderer> { unimplemented!() }
//! # }
//! use iced_test::simulator;
//!
//! let mut counter = Counter { value: 0 };
//! let mut ui = simulator(counter.view());
//! ```
//!
//! Now we can simulate a user interacting with our interface. Let's use [`Simulator::click`] to click
//! the counter buttons:
//!
//! ```rust,no_run
//! # struct Counter { value: i64 }
//! # impl Counter {
//! #    pub fn view(&self) -> iced_runtime::core::Element<(), iced_runtime::core::Theme, iced_renderer::Renderer> { unimplemented!() }
//! # }
//! use iced_test::selector::text;
//! # use iced_test::simulator;
//! #
//! # let mut counter = Counter { value: 0 };
//! # let mut ui = simulator(counter.view());
//!
//! let _ = ui.click(text("+"));
//! let _ = ui.click(text("+"));
//! let _ = ui.click(text("-"));
//! ```
//!
//! [`Simulator::click`] takes a [`Selector`]. A [`Selector`] describes a way to query the widgets of an interface. In this case,
//! [`selector::text`] lets us select a widget by the text it contains.
//!
//! We can now process any messages produced by these interactions and then assert that the final value of our counter is
//! indeed `1`!
//!
//! ```rust,no_run
//! # struct Counter { value: i64 }
//! # impl Counter {
//! #    pub fn update(&mut self, message: ()) {}
//! #    pub fn view(&self) -> iced_runtime::core::Element<(), iced_runtime::core::Theme, iced_renderer::Renderer> { unimplemented!() }
//! # }
//! # use iced_test::selector::text;
//! # use iced_test::simulator;
//! #
//! # let mut counter = Counter { value: 0 };
//! # let mut ui = simulator(counter.view());
//! #
//! # let _ = ui.click(text("+"));
//! # let _ = ui.click(text("+"));
//! # let _ = ui.click(text("-"));
//! #
//! for message in ui.into_messages() {
//!     counter.update(message);
//! }
//!
//! assert_eq!(counter.value, 1);
//! ```
//!
//! We can even rebuild the interface to make sure the counter _displays_ the proper value with [`Simulator::find`]:
//!
//! ```rust,no_run
//! # struct Counter { value: i64 }
//! # impl Counter {
//! #    pub fn view(&self) -> iced_runtime::core::Element<(), iced_runtime::core::Theme, iced_renderer::Renderer> { unimplemented!() }
//! # }
//! # use iced_test::selector::text;
//! # use iced_test::simulator;
//! #
//! # let mut counter = Counter { value: 0 };
//! let mut ui = simulator(counter.view());
//!
//! assert!(ui.find(text("1")).is_ok(), "Counter should display 1!");
//! ```
//!
//! And that's it! That's the gist of testing `iced` applications!
//!
//! [`Simulator`] contains additional operations you can use to simulate more interactions—like [`tap_key`](Simulator::tap_key) or
//! [`typewrite`](Simulator::typewrite)—and even perform [_snapshot testing_](Simulator::snapshot)!
//!
//! [the classical counter interface]: https://book.iced.rs/architecture.html#dissecting-an-interface
pub mod selector;

pub use selector::Selector;

use iced_renderer as renderer;
use iced_runtime as runtime;
use iced_runtime::core;

use crate::core::clipboard;
use crate::core::event;
use crate::core::keyboard;
use crate::core::mouse;
use crate::core::theme;
use crate::core::time;
use crate::core::widget;
use crate::core::window;
use crate::core::{
    Element, Event, Font, Point, Rectangle, Settings, Size, SmolStr,
};
use crate::runtime::UserInterface;
use crate::runtime::user_interface;

use std::borrow::Cow;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Creates a new [`Simulator`].
///
/// This is just a function version of [`Simulator::new`].
pub fn simulator<'a, Message, Theme, Renderer>(
    element: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Simulator<'a, Message, Theme, Renderer>
where
    Theme: theme::Base,
    Renderer: core::Renderer + core::renderer::Headless,
{
    Simulator::new(element)
}

/// A user interface that can be interacted with and inspected programmatically.
#[allow(missing_debug_implementations)]
pub struct Simulator<
    'a,
    Message,
    Theme = core::Theme,
    Renderer = renderer::Renderer,
> {
    raw: UserInterface<'a, Message, Theme, Renderer>,
    renderer: Renderer,
    size: Size,
    cursor: mouse::Cursor,
    messages: Vec<Message>,
}

/// A specific area of a [`Simulator`], normally containing a widget.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Target {
    /// The bounds of the area.
    pub bounds: Rectangle,
}

impl<'a, Message, Theme, Renderer> Simulator<'a, Message, Theme, Renderer>
where
    Theme: theme::Base,
    Renderer: core::Renderer + core::renderer::Headless,
{
    /// Creates a new [`Simulator`] with default [`Settings`] and a default size (1024x768).
    pub fn new(
        element: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self::with_settings(Settings::default(), element)
    }

    /// Creates a new [`Simulator`] with the given [`Settings`] and a default size (1024x768).
    pub fn with_settings(
        settings: Settings,
        element: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self::with_size(settings, window::Settings::default().size, element)
    }

    /// Creates a new [`Simulator`] with the given [`Settings`] and size.
    pub fn with_size(
        settings: Settings,
        size: impl Into<Size>,
        element: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        let size = size.into();

        let default_font = match settings.default_font {
            Font::DEFAULT => Font::with_name("Fira Sans"),
            _ => settings.default_font,
        };

        for font in settings.fonts {
            load_font(font).expect("Font must be valid");
        }

        let mut renderer = {
            let backend = env::var("ICED_TEST_BACKEND").ok();

            iced_runtime::futures::futures::executor::block_on(Renderer::new(
                default_font,
                settings.default_text_size,
                backend.as_deref(),
            ))
            .expect("Create new headless renderer")
        };

        let raw = UserInterface::build(
            element,
            size,
            user_interface::Cache::default(),
            &mut renderer,
        );

        Simulator {
            raw,
            renderer,
            size,
            cursor: mouse::Cursor::Unavailable,
            messages: Vec::new(),
        }
    }

    /// Finds the [`Target`] of the given widget [`Selector`] in the [`Simulator`].
    pub fn find(
        &mut self,
        selector: impl Into<Selector>,
    ) -> Result<Target, Error> {
        let selector = selector.into();

        match &selector {
            Selector::Id(id) => {
                struct FindById<'a> {
                    id: &'a widget::Id,
                    target: Option<Target>,
                }

                impl widget::Operation for FindById<'_> {
                    fn container(
                        &mut self,
                        id: Option<&widget::Id>,
                        bounds: Rectangle,
                        operate_on_children: &mut dyn FnMut(
                            &mut dyn widget::Operation<()>,
                        ),
                    ) {
                        if self.target.is_some() {
                            return;
                        }

                        if Some(self.id) == id {
                            self.target = Some(Target { bounds });
                            return;
                        }

                        operate_on_children(self);
                    }

                    fn scrollable(
                        &mut self,
                        id: Option<&widget::Id>,
                        bounds: Rectangle,
                        _content_bounds: Rectangle,
                        _translation: core::Vector,
                        _state: &mut dyn widget::operation::Scrollable,
                    ) {
                        if self.target.is_some() {
                            return;
                        }

                        if Some(self.id) == id {
                            self.target = Some(Target { bounds });
                        }
                    }

                    fn text_input(
                        &mut self,
                        id: Option<&widget::Id>,
                        bounds: Rectangle,
                        _state: &mut dyn widget::operation::TextInput,
                    ) {
                        if self.target.is_some() {
                            return;
                        }

                        if Some(self.id) == id {
                            self.target = Some(Target { bounds });
                        }
                    }

                    fn text(
                        &mut self,
                        id: Option<&widget::Id>,
                        bounds: Rectangle,
                        _text: &str,
                    ) {
                        if self.target.is_some() {
                            return;
                        }

                        if Some(self.id) == id {
                            self.target = Some(Target { bounds });
                        }
                    }

                    fn custom(
                        &mut self,
                        id: Option<&widget::Id>,
                        bounds: Rectangle,
                        _state: &mut dyn std::any::Any,
                    ) {
                        if self.target.is_some() {
                            return;
                        }

                        if Some(self.id) == id {
                            self.target = Some(Target { bounds });
                        }
                    }
                }

                let mut find = FindById { id, target: None };
                self.raw.operate(&self.renderer, &mut find);

                find.target.ok_or(Error::NotFound(selector))
            }
            Selector::Text(text) => {
                struct FindByText<'a> {
                    text: &'a str,
                    target: Option<Target>,
                }

                impl widget::Operation for FindByText<'_> {
                    fn container(
                        &mut self,
                        _id: Option<&widget::Id>,
                        _bounds: Rectangle,
                        operate_on_children: &mut dyn FnMut(
                            &mut dyn widget::Operation<()>,
                        ),
                    ) {
                        if self.target.is_some() {
                            return;
                        }

                        operate_on_children(self);
                    }

                    fn text(
                        &mut self,
                        _id: Option<&widget::Id>,
                        bounds: Rectangle,
                        text: &str,
                    ) {
                        if self.target.is_some() {
                            return;
                        }

                        if self.text == text {
                            self.target = Some(Target { bounds });
                        }
                    }
                }

                let mut find = FindByText { text, target: None };
                self.raw.operate(&self.renderer, &mut find);

                find.target.ok_or(Error::NotFound(selector))
            }
        }
    }

    /// Points the mouse cursor at the given position in the [`Simulator`].
    ///
    /// This does _not_ produce mouse movement events!
    pub fn point_at(&mut self, position: impl Into<Point>) {
        self.cursor = mouse::Cursor::Available(position.into());
    }

    /// Clicks the [`Target`] found by the given [`Selector`], if any.
    ///
    /// This consists in:
    /// - Pointing the mouse cursor at the center of the [`Target`].
    /// - Simulating a [`click`].
    pub fn click(
        &mut self,
        selector: impl Into<Selector>,
    ) -> Result<Target, Error> {
        let target = self.find(selector)?;
        self.point_at(target.bounds.center());

        let _ = self.simulate(click());

        Ok(target)
    }

    /// Simulates a key press, followed by a release, in the [`Simulator`].
    pub fn tap_key(&mut self, key: impl Into<keyboard::Key>) -> event::Status {
        self.simulate(tap_key(key, None))
            .first()
            .copied()
            .unwrap_or(event::Status::Ignored)
    }

    /// Simulates a user typing in the keyboard the given text in the [`Simulator`].
    pub fn typewrite(&mut self, text: &str) -> event::Status {
        let statuses = self.simulate(typewrite(text));

        statuses
            .into_iter()
            .fold(event::Status::Ignored, event::Status::merge)
    }

    /// Simulates the given raw sequence of events in the [`Simulator`].
    pub fn simulate(
        &mut self,
        events: impl IntoIterator<Item = Event>,
    ) -> Vec<event::Status> {
        let events: Vec<Event> = events.into_iter().collect();

        let (_state, statuses) = self.raw.update(
            &events,
            self.cursor,
            &mut self.renderer,
            &mut clipboard::Null,
            &mut self.messages,
        );

        statuses
    }

    /// Draws and takes a [`Snapshot`] of the interface in the [`Simulator`].
    pub fn snapshot(&mut self, theme: &Theme) -> Result<Snapshot, Error> {
        let base = theme.base();

        let _ = self.raw.update(
            &[Event::Window(window::Event::RedrawRequested(
                time::Instant::now(),
            ))],
            self.cursor,
            &mut self.renderer,
            &mut clipboard::Null,
            &mut self.messages,
        );

        self.raw.draw(
            &mut self.renderer,
            theme,
            &core::renderer::Style {
                text_color: base.text_color,
            },
            self.cursor,
        );

        let scale_factor = 2.0;

        let physical_size = Size::new(
            (self.size.width * scale_factor).round() as u32,
            (self.size.height * scale_factor).round() as u32,
        );

        let rgba = self.renderer.screenshot(
            physical_size,
            scale_factor,
            base.background_color,
        );

        Ok(Snapshot {
            screenshot: window::Screenshot::new(
                rgba,
                physical_size,
                f64::from(scale_factor),
            ),
            renderer: self.renderer.name(),
        })
    }

    /// Turns the [`Simulator`] into the sequence of messages produced by any interactions.
    pub fn into_messages(
        self,
    ) -> impl Iterator<Item = Message> + use<Message, Theme, Renderer> {
        self.messages.into_iter()
    }
}

/// A frame of a user interface rendered by a [`Simulator`].
#[derive(Debug, Clone)]
pub struct Snapshot {
    screenshot: window::Screenshot,
    renderer: String,
}

impl Snapshot {
    /// Compares the [`Snapshot`] with the PNG image found in the given path, returning
    /// `true` if they are identical.
    ///
    /// If the PNG image does not exist, it will be created by the [`Snapshot`] for future
    /// testing and `true` will be returned.
    pub fn matches_image(&self, path: impl AsRef<Path>) -> Result<bool, Error> {
        let path = self.path(path, "png");

        if path.exists() {
            let file = fs::File::open(&path)?;
            let decoder = png::Decoder::new(file);

            let mut reader = decoder.read_info()?;
            let mut bytes = vec![0; reader.output_buffer_size()];
            let info = reader.next_frame(&mut bytes)?;

            Ok(self.screenshot.bytes == bytes[..info.buffer_size()])
        } else {
            if let Some(directory) = path.parent() {
                fs::create_dir_all(directory)?;
            }

            let file = fs::File::create(path)?;

            let mut encoder = png::Encoder::new(
                file,
                self.screenshot.size.width,
                self.screenshot.size.height,
            );
            encoder.set_color(png::ColorType::Rgba);

            let mut writer = encoder.write_header()?;
            writer.write_image_data(&self.screenshot.bytes)?;
            writer.finish()?;

            Ok(true)
        }
    }

    /// Compares the [`Snapshot`] with the SHA-256 hash file found in the given path, returning
    /// `true` if they are identical.
    ///
    /// If the hash file does not exist, it will be created by the [`Snapshot`] for future
    /// testing and `true` will be returned.
    pub fn matches_hash(&self, path: impl AsRef<Path>) -> Result<bool, Error> {
        use sha2::{Digest, Sha256};

        let path = self.path(path, "sha256");

        let hash = {
            let mut hasher = Sha256::new();
            hasher.update(&self.screenshot.bytes);
            format!("{:x}", hasher.finalize())
        };

        if path.exists() {
            let saved_hash = fs::read_to_string(&path)?;

            Ok(hash == saved_hash)
        } else {
            if let Some(directory) = path.parent() {
                fs::create_dir_all(directory)?;
            }

            fs::write(path, hash)?;
            Ok(true)
        }
    }

    fn path(&self, path: impl AsRef<Path>, extension: &str) -> PathBuf {
        let path = path.as_ref();

        path.with_file_name(format!(
            "{name}-{renderer}",
            name = path
                .file_stem()
                .map(std::ffi::OsStr::to_string_lossy)
                .unwrap_or_default(),
            renderer = self.renderer
        ))
        .with_extension(extension)
    }
}

/// Returns the sequence of events of a click.
pub fn click() -> impl Iterator<Item = Event> {
    [
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
    ]
    .into_iter()
}

/// Returns the sequence of events of a "key tap" (i.e. pressing and releasing a key).
pub fn tap_key(
    key: impl Into<keyboard::Key>,
    text: Option<SmolStr>,
) -> impl Iterator<Item = Event> {
    let key = key.into();

    [
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key.clone(),
            physical_key: keyboard::key::Physical::Unidentified(
                keyboard::key::NativeCode::Unidentified,
            ),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::default(),
            text,
        }),
        Event::Keyboard(keyboard::Event::KeyReleased {
            key: key.clone(),
            modified_key: key,
            physical_key: keyboard::key::Physical::Unidentified(
                keyboard::key::NativeCode::Unidentified,
            ),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::default(),
        }),
    ]
    .into_iter()
}

/// Returns the sequence of events of typewriting the given text in a keyboard.
pub fn typewrite(text: &str) -> impl Iterator<Item = Event> + '_ {
    text.chars()
        .map(|c| SmolStr::new_inline(&c.to_string()))
        .flat_map(|c| tap_key(keyboard::Key::Character(c.clone()), Some(c)))
}

/// A test error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    /// No matching widget was found for the [`Selector`].
    #[error("no matching widget was found for the selector: {0:?}")]
    NotFound(Selector),
    /// An IO operation failed.
    #[error("an IO operation failed: {0}")]
    IOFailed(Arc<io::Error>),
    /// The decoding of some PNG image failed.
    #[error("the decoding of some PNG image failed: {0}")]
    PngDecodingFailed(Arc<png::DecodingError>),
    /// The encoding of some PNG image failed.
    #[error("the encoding of some PNG image failed: {0}")]
    PngEncodingFailed(Arc<png::EncodingError>),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::IOFailed(Arc::new(error))
    }
}

impl From<png::DecodingError> for Error {
    fn from(error: png::DecodingError) -> Self {
        Self::PngDecodingFailed(Arc::new(error))
    }
}

impl From<png::EncodingError> for Error {
    fn from(error: png::EncodingError) -> Self {
        Self::PngEncodingFailed(Arc::new(error))
    }
}

fn load_font(font: impl Into<Cow<'static, [u8]>>) -> Result<(), Error> {
    renderer::graphics::text::font_system()
        .write()
        .expect("Write to font system")
        .load_font(font.into());

    Ok(())
}
