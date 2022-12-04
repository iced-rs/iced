//! Access to winit ime related things.
use std::{collections::HashMap, sync::RwLock};

use crate::command::{self, Command};
pub use iced_native::clipboard::Action;
use iced_native::widget;

use winit::window::Window;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum RequestKind {
    Outside,
    Gain,
    Password,
}

/// IME related setting interface.
///
/// This is the wrapper of winit window reference so youd don't have to care about cost of initialize this struct.
#[derive(Debug)]
pub struct IME<'a> {
    window: &'a Window,

    requests: RwLock<HashMap<widget::Id, RequestKind>>,
}

impl<'a> IME<'a> {
    /// connect to winit
    pub fn connect(window: &'a Window) -> Self {
        Self {
            window,
            requests: RwLock::new(HashMap::new()),
        }
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

    fn ime_disable_or_enable(&self) {
        if let Ok(allowed) = self.requests.read().map(|guard| {
            guard
                .iter()
                .map(|(_, request)| request)
                .map(|request_kind| match request_kind {
                    RequestKind::Outside => false,
                    RequestKind::Gain => true,
                    RequestKind::Password => false,
                })
                .fold(false, |or, this| or | this)
        }) {
            self.window.set_ime_allowed(allowed);
        }
    }
}

impl<'a> iced_native::ime::IME for IME<'a> {
    fn set_ime_position(&self, x: i32, y: i32) {
        self.set_ime_position(x, y)
    }

    /// id's widget will gain focus.
    ///
    /// enable IME will controlled.
    fn gain(&self, id: Option<widget::Id>) {
        if let Some(id) = id {
            if let Ok(mut guard) = self.requests.write() {
                let _ = guard.insert(id, RequestKind::Gain);
            };
        }
        self.ime_disable_or_enable()
    }

    /// need to call if clicked position is not widget's region.
    ///
    /// used to determine disable ime.
    fn outside(&self, id: Option<widget::Id>) {
        if let Some(id) = id {
            if let Ok(mut guard) = self.requests.write() {
                let _ = guard.insert(id, RequestKind::Outside);
            };
        }
        self.ime_disable_or_enable()
    }

    /// disable IME.
    ///
    fn password_mode(&self, id: Option<widget::Id>) {
        if let Some(id) = id {
            if let Ok(mut guard) = self.requests.write() {
                let _ = guard.insert(id, RequestKind::Password);
            };
        }
        self.ime_disable_or_enable()
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
