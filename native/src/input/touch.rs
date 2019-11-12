//! Build touch events.
/// The touch of a mobile device.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Touch {
    /// The touch cursor was started
    Started {
        /// The X coordinate of the touch position
        x: f32,

        /// The Y coordinate of the touch position
        y: f32,
    },
    /// The touch cursor was ended
    Ended {
        /// The X coordinate of the touch position
        x: f32,

        /// The Y coordinate of the touch position
        y: f32,
    },

    /// The touch was moved.
    Moved {
        /// The X coordinate of the touch position
        x: f32,

        /// The Y coordinate of the touch position
        y: f32,
    },

    /// Some canceled button.
    Cancelled {
        /// The X coordinate of the touch position
        x: f32,

        /// The Y coordinate of the touch position
        y: f32,
    },
}
