use std::path::PathBuf;

/// A window-related event.
#[derive(PartialEq, Clone, Debug)]
pub enum Event {
    /// A window was moved.
    Moved {
        /// The new logical x location of the window
        x: i32,
        /// The new logical y location of the window
        y: i32,
    },

    /// A window was resized.
    Resized {
        /// The new width of the window (in units)
        width: u32,

        /// The new height of the window (in units)
        height: u32,
    },

    /// The user has requested for the window to close.
    ///
    /// Usually, you will want to terminate the execution whenever this event
    /// occurs.
    CloseRequested,

    /// A window was focused.
    Focused,

    /// A window was unfocused.
    Unfocused,

    /// A file is being hovered over the window.
    ///
    /// When the user hovers multiple files at once, this event will be emitted
    /// for each file separately.
    FileHovered(PathBuf),

    /// A file has beend dropped into the window.
    ///
    /// When the user drops multiple files at once, this event will be emitted
    /// for each file separately.
    FileDropped(PathBuf),

    /// A file was hovered, but has exited the window.
    ///
    /// There will be a single `FilesHoveredLeft` event triggered even if
    /// multiple files were hovered.
    FilesHoveredLeft,
}
