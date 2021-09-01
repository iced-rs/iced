/// An operation to be performed on some window.
#[derive(Debug)]
pub enum Action {
    /// Resize the window.
    Resize {
        /// The new logical width of the window
        width: u32,
        /// The new logical height of the window
        height: u32,
    },
    /// Move the window.
    Move {
        /// The new logical x location of the window
        x: i32,
        /// The new logical y location of the window
        y: i32,
    },
}
