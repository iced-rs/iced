/// A window-related event.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Event {
    /// A window was resized
    Resized {
        /// The new width of the window (in units)
        width: u32,

        /// The new height of the window (in units)
        height: u32,
    },
}
