//! Handle mouse input.

pub mod click;

mod button;
mod cursor;
mod interaction;

pub use button::Button;
pub use click::Click;
pub use cursor::Cursor;
pub use interaction::Interaction;

/// A scroll movement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollDelta {
    /// A line-based scroll movement.
    Lines {
        /// The number of horizontal lines scrolled.
        x: f32,

        /// The number of vertical lines scrolled.
        y: f32,
    },

    /// A pixel-based scroll movement.
    Pixels {
        /// The number of horizontal pixels scrolled.
        x: f32,

        /// The number of vertical pixels scrolled.
        y: f32,
    },
}
