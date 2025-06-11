//! A renderer-agnostic native GUI runtime.
//!
//! ![The native path of the Iced ecosystem](https://github.com/iced-rs/iced/blob/master/docs/graphs/native.png?raw=true)
//!
//! `iced_runtime` takes [`iced_core`] and builds a native runtime on top of it.
//!
//! [`iced_core`]: https://github.com/iced-rs/iced/tree/0.13/core
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
pub mod clipboard;
pub mod font;
pub mod keyboard;
pub mod overlay;
pub mod system;
pub mod task;
pub mod user_interface;
pub mod window;

pub use iced_core as core;
pub use iced_debug as debug;
pub use iced_futures as futures;

pub use task::Task;
pub use user_interface::UserInterface;

use crate::core::widget;
use crate::futures::futures::channel::oneshot;

use std::borrow::Cow;
use std::fmt;

/// An action that the iced runtime can perform.
pub enum Action<T> {
    /// Output some value.
    Output(T),

    /// Load a font from its bytes.
    LoadFont {
        /// The bytes of the font to load.
        bytes: Cow<'static, [u8]>,
        /// The channel to send back the load result.
        channel: oneshot::Sender<Result<(), font::Error>>,
    },

    /// Run a widget operation.
    Widget(Box<dyn widget::Operation>),

    /// Run a clipboard action.
    Clipboard(clipboard::Action),

    /// Run a window action.
    Window(window::Action),

    /// Run a system action.
    System(system::Action),

    /// Exits the runtime.
    ///
    /// This will normally close any application windows and
    /// terminate the runtime loop.
    Exit,
}

impl<T> Action<T> {
    /// Creates a new [`Action::Widget`] with the given [`widget::Operation`].
    pub fn widget(operation: impl widget::Operation + 'static) -> Self {
        Self::Widget(Box::new(operation))
    }

    fn output<O>(self) -> Result<T, Action<O>> {
        match self {
            Action::Output(output) => Ok(output),
            Action::LoadFont { bytes, channel } => {
                Err(Action::LoadFont { bytes, channel })
            }
            Action::Widget(operation) => Err(Action::Widget(operation)),
            Action::Clipboard(action) => Err(Action::Clipboard(action)),
            Action::Window(action) => Err(Action::Window(action)),
            Action::System(action) => Err(Action::System(action)),
            Action::Exit => Err(Action::Exit),
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
            Action::LoadFont { .. } => {
                write!(f, "Action::LoadFont")
            }
            Action::Widget { .. } => {
                write!(f, "Action::Widget")
            }
            Action::Clipboard(action) => {
                write!(f, "Action::Clipboard({action:?})")
            }
            Action::Window(_) => write!(f, "Action::Window"),
            Action::System(action) => write!(f, "Action::System({action:?})"),
            Action::Exit => write!(f, "Action::Exit"),
        }
    }
}

/// Creates a [`Task`] that exits the iced runtime.
///
/// This will normally close any application windows and
/// terminate the runtime loop.
pub fn exit<T>() -> Task<T> {
    task::effect(Action::Exit)
}
