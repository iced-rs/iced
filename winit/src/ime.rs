//! Access to winit ime related things.
pub use iced_native::clipboard::Action;

use crate::command::{self, Command};

use winit::window::Window;

/// IME related setting interface.
///
/// This is the wrapper of winit window reference so youd don't have to care about cost of initialize this struct.
#[derive(Debug)]
pub struct IME<'a> {
    window: &'a Window,
}

impl<'a> IME<'a> {
    /// connect to winit
    pub fn connect(window: &'a Window) -> Self {
        Self { window }
    }
}
impl<'a> IME<'a> {
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

impl<'a> iced_native::ime::IME for IME<'a> {
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
