/// The behavior of cursor grabbing.
///
/// Use this enum with [`Window::set_cursor_grab`] to grab the cursor.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CursorGrabMode {
    /// No grabbing of the cursor is performed.
    None,

    /// The cursor is confined to the window area.
    ///
    /// There's no guarantee that the cursor will be hidden. You should hide it by yourself if you
    /// want to do so.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** Not implemented. Always returns [`ExternalError::NotSupported`] for now.
    /// - **iOS / Android / Web:** Always returns an [`ExternalError::NotSupported`].
    Confined,

    /// The cursor is locked inside the window area to the certain position.
    ///
    /// There's no guarantee that the cursor will be hidden. You should hide it by yourself if you
    /// want to do so.
    ///
    /// ## Platform-specific
    ///
    /// - **X11 / Windows:** Not implemented. Always returns [`ExternalError::NotSupported`] for now.
    /// - **iOS / Android:** Always returns an [`ExternalError::NotSupported`].
    Locked,
}

/// Describes the appearance of the mouse cursor.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CursorIcon {
    /// The platform-dependent default cursor.
    Default,
    /// A simple crosshair.
    Crosshair,
    /// A hand (often used to indicate links in web browsers).
    Hand,
    /// Self explanatory.
    Arrow,
    /// Indicates something is to be moved.
    Move,
    /// Indicates text that may be selected or edited.
    Text,
    /// Program busy indicator.
    Wait,
    /// Help indicator (often rendered as a "?")
    Help,
    /// Progress indicator. Shows that processing is being done. But in contrast
    /// with "Wait" the user may still interact with the program. Often rendered
    /// as a spinning beach ball, or an arrow with a watch or hourglass.
    Progress,

    /// Cursor showing that something cannot be done.
    NotAllowed,
    ContextMenu,
    Cell,
    VerticalText,
    Alias,
    Copy,
    NoDrop,
    /// Indicates something can be grabbed.
    Grab,
    /// Indicates something is grabbed.
    Grabbing,
    AllScroll,
    ZoomIn,
    ZoomOut,

    /// Indicate that some edge is to be moved. For example, the 'SeResize' cursor
    /// is used when the movement starts from the south-east corner of the box.
    EResize,
    NResize,
    NeResize,
    NwResize,
    SResize,
    SeResize,
    SwResize,
    WResize,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ColResize,
    RowResize,
}

impl Default for CursorIcon {
    fn default() -> Self {
        CursorIcon::Default
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

/// ## Platform-specific
///
/// - **X11:** Sets the WM's `XUrgencyHint`. No distinction between [`Critical`] and [`Informational`].
///
/// [`Critical`]: Self::Critical
/// [`Informational`]: Self::Informational
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserAttentionType {
    /// ## Platform-specific
    ///
    /// - **macOS:** Bounces the dock icon until the application is in focus.
    /// - **Windows:** Flashes both the window and the taskbar button until the application is in focus.
    Critical,
    /// ## Platform-specific
    ///
    /// - **macOS:** Bounces the dock icon once.
    /// - **Windows:** Flashes the taskbar button until the application is in focus.
    Informational,
}

impl Default for UserAttentionType {
    fn default() -> Self {
        UserAttentionType::Informational
    }
}
