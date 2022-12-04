//! Access the IME.
use std::fmt;

use crate::widget;

///
pub trait IME {
    ///
    fn set_ime_position(&self, x: i32, y: i32);

    /// id's widget will gain focus.
    ///
    /// enable IME will controlled.
    fn gain(&self, id: Option<widget::Id>);

    /// need to call if clicked position is not widget's region.
    ///
    /// used to determine disable ime.
    fn outside(&self, id: Option<widget::Id>);

    /// disable IME.
    ///
    fn password_mode(&self, id: Option<widget::Id>);
}

/// A null implementation of the [`IME`] trait.
#[derive(Debug, Clone, Copy)]
pub struct Null;

impl IME for Null {
    fn set_ime_position(&self, _x: i32, _y: i32) {}

    fn outside(&self, _id: Option<widget::Id>) {}

    fn password_mode(&self, _id: Option<widget::Id>) {}

    fn gain(&self, _id: Option<widget::Id>) {}
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
