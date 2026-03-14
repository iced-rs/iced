//! A renderer-agnostic native GUI runtime.
//!
//! ![The native path of the Iced ecosystem](https://github.com/iced-rs/iced/blob/master/docs/graphs/native.png?raw=true)
//!
//! `iced_runtime` takes [`iced_core`] and builds a native runtime on top of it.
//!
//! [`iced_core`]: https://github.com/iced-rs/iced/tree/master/core
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
pub mod clipboard;
pub mod font;
pub mod image;
pub mod keyboard;
pub mod system;
pub mod task;
pub mod user_interface;
pub mod widget;
pub mod window;

pub use iced_core as core;
pub use iced_futures as futures;

pub use task::Task;
pub use user_interface::UserInterface;
pub use window::Window;

use crate::core::Event;

use std::fmt;

/// An action that the iced runtime can perform.
pub enum Action<T> {
    /// Output some value.
    Output(T),

    /// Run a widget operation.
    Widget(Box<dyn core::widget::Operation>),

    /// Run a clipboard action.
    Clipboard(clipboard::Action),

    /// Run a window action.
    Window(window::Action),

    /// Run a system action.
    System(system::Action),

    /// Run a font action.
    Font(font::Action),

    /// Run an image action.
    Image(image::Action),

    /// Produce an event.
    Event {
        /// The [`window::Id`](core::window::Id) of the event.
        window: core::window::Id,
        /// The [`Event`] to be produced.
        event: Event,
    },

    /// Poll any resources that may have pending computations.
    Tick,

    /// Recreate all user interfaces and redraw all windows.
    Reload,

    /// Announce a message to assistive technology via a live region.
    ///
    /// The text will be spoken by screen readers using an assertive
    /// live-region announcement on the next accessibility tree update.
    Announce(String),

    /// Exits the runtime.
    ///
    /// This will normally close any application windows and
    /// terminate the runtime loop.
    Exit,
}

impl<T> Action<T> {
    /// Creates a new [`Action::Widget`] with the given [`widget::Operation`](core::widget::Operation).
    pub fn widget(operation: impl core::widget::Operation + 'static) -> Self {
        Self::Widget(Box::new(operation))
    }

    fn output<O>(self) -> Result<T, Action<O>> {
        match self {
            Action::Output(output) => Ok(output),
            Action::Widget(operation) => Err(Action::Widget(operation)),
            Action::Clipboard(action) => Err(Action::Clipboard(action)),
            Action::Window(action) => Err(Action::Window(action)),
            Action::System(action) => Err(Action::System(action)),
            Action::Font(action) => Err(Action::Font(action)),
            Action::Image(action) => Err(Action::Image(action)),
            Action::Event { window, event } => Err(Action::Event { window, event }),
            Action::Tick => Err(Action::Tick),
            Action::Reload => Err(Action::Reload),
            Action::Exit => Err(Action::Exit),
            Action::Announce(text) => Err(Action::Announce(text)),
        }
    }
}

impl<T> fmt::Debug for Action<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Output(output) => write!(f, "Action::Output({output:?})"),
            Action::Widget { .. } => {
                write!(f, "Action::Widget")
            }
            Action::Clipboard(action) => {
                write!(f, "Action::Clipboard({action:?})")
            }
            Action::Window(_) => write!(f, "Action::Window"),
            Action::System(action) => write!(f, "Action::System({action:?})"),
            Action::Font(action) => {
                write!(f, "Action::Font({action:?})")
            }
            Action::Image(_) => write!(f, "Action::Image"),
            Action::Event { window, event } => write!(
                f,
                "Action::Event {{ window: {window:?}, event: {event:?} }}"
            ),
            Action::Tick => write!(f, "Action::Tick"),
            Action::Reload => write!(f, "Action::Reload"),
            Action::Exit => write!(f, "Action::Exit"),
            Action::Announce(text) => {
                write!(f, "Action::Announce({text:?})")
            }
        }
    }
}

/// Creates a [`Task`] that announces the given text to assistive
/// technology via a live region.
///
/// Screen readers will speak the text using an assertive announcement.
#[cfg(feature = "a11y")]
#[cfg_attr(docsrs, doc(cfg(feature = "a11y")))]
pub fn announce<T>(text: impl Into<String>) -> Task<T> {
    task::effect(Action::Announce(text.into()))
}

/// Creates a [`Task`] that exits the iced runtime.
///
/// This will normally close any application windows and
/// terminate the runtime loop.
pub fn exit<T>() -> Task<T> {
    task::effect(Action::Exit)
}
