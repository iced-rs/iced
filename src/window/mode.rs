/// The mode of a window-based application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// The application appears in its own window.
    Windowed,

    /// The application takes the whole screen of its current monitor.
    Fullscreen,
}
