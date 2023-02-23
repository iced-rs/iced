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

impl From<PlatformSpecific> for iced_winit::settings::PlatformSpecific {
    fn from(platform_specific: PlatformSpecific) -> Self {
        Self {
            title_hidden: platform_specific.title_hidden,
            titlebar_transparent: platform_specific.titlebar_transparent,
            fullsize_content_view: platform_specific.fullsize_content_view,
        }
    }
}
