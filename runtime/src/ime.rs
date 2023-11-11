//! Access the IME

use std::fmt;

/// A IME action to be performed by some [`Command`].
///
/// [`Command`]: crate::Command
pub enum Action {
    ///
    Allow(bool),

    ///
    Position(i32, i32),
    ///
    Unlock,
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Allow(_) => {
                write!(f, "Action::Allow")
            }
            Action::Position(_, _) => write!(f, "Action::SetPosition"),
            Action::Unlock => write!(f, "Action::Unlock"),
        }
    }
}
