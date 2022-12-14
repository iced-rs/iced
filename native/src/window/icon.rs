//! Attach an icon to the window of your application.

/// The icon of a window.
#[derive(Debug, Clone)]
pub struct Icon {
    /// TODO(derezzedex)
    pub rgba: Vec<u8>,
    /// TODO(derezzedex)
    pub width: u32,
    /// TODO(derezzedex)
    pub height: u32,
}
