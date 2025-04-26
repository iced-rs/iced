//! Platform specific settings for macOS.

/// The platform specific window settings of an application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformSpecific {
    /// Hides the window title.
    pub title_hidden: bool,
    /// Makes the titlebar transparent and allows the content to appear behind it.
    pub titlebar_transparent: bool,
    /// Makes the window content appear behind the titlebar.
    pub fullsize_content_view: bool,
    /// Sets the window's blur radius.
    pub blur_radius: i64,
}

impl Default for PlatformSpecific {
    fn default() -> Self {
        Self {
            title_hidden: false,
            titlebar_transparent: false,
            fullsize_content_view: false,
            blur_radius: 80,
        }
    }
}
