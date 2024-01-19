//! Platform specific settings for macOS.

/// The platform specific window settings of an application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlatformSpecific {
    /// Hides the window title.
    pub title_hidden: bool,
    /// Makes the titlebar transparent and allows the content to appear behind it.
    pub titlebar_transparent: bool,
    /// Makes the window content appear behind the titlebar.
    pub fullsize_content_view: bool,
}
