//! Wayland-specific platform actions.
//!
//! This module provides actions for Wayland-specific features like
//! popup surfaces that extend outside the parent window.

use std::fmt;

/// Popup surface actions
pub mod popup;

/// Wayland-specific actions.
#[derive(Clone)]
pub enum Action {
    /// Popup surface action
    Popup(popup::Action),
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Popup(action) => f.debug_tuple("Popup").field(action).finish(),
        }
    }
}
