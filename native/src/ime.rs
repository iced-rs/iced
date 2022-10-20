//! Access the IME.
use std::fmt;

///
pub trait IME {
    ///
    fn set_ime_allowed(&self, allowed: bool);

    ///
    fn set_ime_position(&self, x: i32, y: i32);
}

/// A null implementation of the [`IME`] trait.
#[derive(Debug, Clone, Copy)]
pub struct Null;

impl IME for Null {
    fn set_ime_allowed(&self, _allowed: bool) {}

    fn set_ime_position(&self, _x: i32, _y: i32) {}
}

/// A IME action to be performed by some [`Command`].
///
/// [`Command`]: crate::Command
pub enum Action {
    ///
    Allow(bool),

    ///
    Position(i32, i32),
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Allow(_) => {
                write!(f, "Action::Allow")
            }
            Action::Position(_, _) => write!(f, "Action::SetPosition"),
        }
    }
}
