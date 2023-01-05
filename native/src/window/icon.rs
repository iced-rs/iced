//! Attach an icon to the window of your application.

/// The icon of a window.
#[derive(Debug, Clone)]
pub struct Icon {
    /// The __rgba__ color data of the window [`Icon`].
    pub rgba: Vec<u8>,
    /// The width of the window [`Icon`].
    pub width: u32,
    /// The height of the window [`Icon`].
    pub height: u32,
}
