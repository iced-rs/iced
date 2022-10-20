//! Access to winit ime related things.
pub use iced_native::clipboard::Action;

use crate::command::{self, Command};

#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

use winit::window::Window;

#[cfg(target_arch = "wasm32")]
pub struct IME {
    window: Rc<Window>,
}

#[cfg(not(target_arch = "wasm32"))]
/// ime related setting interface.
#[derive(Debug)]
pub struct IME {
    window: Arc<Window>,
}
#[cfg(target_arch = "wasm32")]

/// ime related setting interface.
#[derive(Debug)]
impl IME {
    /// connect to winit
    pub fn connect(window: Rc<Window>) -> Self {
        Self { window }
    }
}
#[cfg(not(target_arch = "wasm32"))]
impl IME {
    /// connect to winit
    pub fn connect(window: Arc<Window>) -> Self {
        Self { window }
    }
}
impl IME {
    /// allow input by ime or not.
    pub fn set_ime_allowed(&self, allowed: bool) {
        self.window.set_ime_allowed(allowed)
    }
    /// set the logical position of IME candidate window.
    pub fn set_ime_position(&self, x: i32, y: i32) {
        self.window
            .set_ime_position(winit::dpi::LogicalPosition { x, y })
    }
}

impl iced_native::ime::IME for IME {
    fn set_ime_allowed(&self, allowed: bool) {
        self.set_ime_allowed(allowed)
    }

    fn set_ime_position(&self, x: i32, y: i32) {
        self.set_ime_position(x, y)
    }
}

/// allow input by ime or not.
pub fn set_ime_allowed<Message>(allowed: bool) -> Command<Message> {
    Command::single(command::Action::IME(iced_native::ime::Action::Allow(
        allowed,
    )))
}

/// set the logical position of IME candidate window.
pub fn set_position<Message>(x: i32, y: i32) -> Command<Message> {
    Command::single(command::Action::IME(iced_native::ime::Action::Position(
        x, y,
    )))
}
