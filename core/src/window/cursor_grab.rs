/// The behavior of cursor grabbing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorGrab {
    /// No grabbing of the cursor is performed.
    #[default]
    None,

    /// The cursor is confined to the window area.
    ///
    /// There's no guarantee that the cursor will be hidden. You should hide it by yourself if you
    /// want to do so.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** Not implemented.
    /// - **iOS / Android / Web:** Unsupported.
    Confined,

    /// The cursor is locked inside the window area to the certain position.
    ///
    /// There's no guarantee that the cursor will be hidden. You should hide it by yourself if you
    /// want to do so.
    ///
    /// ## Platform-specific
    ///
    /// - **X11 / Windows:** Not implemented.
    /// - **iOS / Android:** Unsupported.
    Locked,
}
