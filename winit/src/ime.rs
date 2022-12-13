//! Access to winit ime related things.
use std::sync::RwLock;

use crate::command::{self, Command};
pub use iced_native::clipboard::Action;

use winit::window::Window;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum RequestKind {
    Outside,
    Inside,
    Password,
}

/// IME related setting interface.
///
/// This is the wrapper of winit window reference so youd don't have to care about cost of initialize this struct.
///
/// need to recreate every frame.
///
/// when application::update and UserInterface::update finished ,call change_ime_enabled_or_disable.

#[derive(Debug, Default)]
pub struct IME {
    requests: RwLock<Vec<RequestKind>>,
    force: RwLock<Option<bool>>,
    position: RwLock<Option<(i32, i32)>>,
}

impl IME {
    /// create manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Send IME enable or disable position update message to winit.
    ///
    ///
    pub fn apply_request(&self, window: &Window) {
        if let Ok(force) = self.force.read() {
            match *force {
                Some(allowed) => {
                    window.set_ime_allowed(allowed);
                }
                None => {
                    if let Ok(mut requests) = self.requests.write() {
                        if !requests.is_empty() {
                            let allowed =
                                requests.drain(..).fold(false, |sum, x| {
                                    sum | matches!(x, RequestKind::Inside)
                                });
                            window.set_ime_allowed(allowed);
                        }
                    }
                }
            }
        }

        if let Ok(mut position) = self.position.write() {
            if let Some((x, y)) = position.take() {
                window.set_ime_position(winit::dpi::LogicalPosition { x, y });
            }
        }
    }
}
impl IME {
    /// allow input by ime or not.
    pub fn set_ime_allowed(&self, allowed: bool) {
        if let Ok(mut guard) = self.force.write() {
            *guard = Some(allowed);
        }
    }

    /// set the logical position of IME candidate window.
    pub fn set_ime_position(&self, x: i32, y: i32) {
        if let Ok(mut guard) = self.position.write() {
            *guard = Some((x, y));
        }
    }

    /// remove the rquest of set_ime_allowed
    pub fn unlock_set_ime_allowed(&self) {
        if let Ok(mut guard) = self.force.write() {
            *guard = None;
        }
    }
}

impl iced_native::ime::IME for IME {
    fn set_ime_position(&self, x: i32, y: i32) {
        self.set_ime_position(x, y)
    }

    fn inside(&self) {
        if let Ok(mut guard) = self.requests.write() {
            guard.push(RequestKind::Inside);
        };
    }

    fn outside(&self) {
        if let Ok(mut guard) = self.requests.write() {
            guard.push(RequestKind::Outside);
        };
    }

    /// disable IME.
    ///
    fn password_mode(&self) {
        if let Ok(mut guard) = self.requests.write() {
            guard.push(RequestKind::Password);
        };
    }

    fn force_set_ime_allowed(&self, allowed: bool) {
        self.set_ime_allowed(allowed);
    }

    fn unlock_set_ime_allowed(&self) {
        self.unlock_set_ime_allowed();
    }
}

/// allow input by ime or not.
pub fn set_ime_allowed<Message>(allowed: bool) -> Command<Message> {
    Command::single(command::Action::IME(iced_native::ime::Action::Allow(
        allowed,
    )))
}

/// allow input by ime or not.
pub fn unlock_set_ime_allowed<Message>() -> Command<Message> {
    Command::single(command::Action::IME(iced_native::ime::Action::Unlock))
}

/// set the logical position of IME candidate window.
pub fn set_position<Message>(x: i32, y: i32) -> Command<Message> {
    Command::single(command::Action::IME(iced_native::ime::Action::Position(
        x, y,
    )))
}
