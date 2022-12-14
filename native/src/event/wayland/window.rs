#![allow(missing_docs)]

/// window events
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowEvent {
    /// window manager capabilities
    WmCapabilities(Vec<u32>),
    /// window state
    State(WindowState),
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// the state of the window
pub enum WindowState {
    Maximized,
    Fullscreen,
    Activated,
    TiledLeft,
    TiledRight,
    TiledTop,
    TiledBottom,
}
