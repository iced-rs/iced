//! Platform specific settings for Windows.

/// The platform specific window settings of an application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformSpecific {
    /// Drag and drop support
    pub drag_and_drop: bool,

    /// Whether show or hide the window icon in the taskbar.
    pub skip_taskbar: bool,

    /// Shows or hides the background drop shadow for undecorated windows.
    ///
    /// The shadow is hidden by default.
    /// Enabling the shadow causes a thin 1px line to appear on the top of the window.
    pub undecorated_shadow: bool,

    /// Sets the preferred style of the window corners.
    ///
    /// Supported starting with Windows 11 Build 22000.
    pub corner_preference: CornerPreference,
}

impl Default for PlatformSpecific {
    fn default() -> Self {
        Self {
            drag_and_drop: true,
            skip_taskbar: false,
            undecorated_shadow: false,
            corner_preference: Default::default(),
        }
    }
}

/// Describes how the corners of a window should look like.
#[repr(i32)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CornerPreference {
    /// Corresponds to `DWMWCP_DEFAULT`.
    ///
    /// Let the system decide when to round window corners.
    #[default]
    Default = 0,

    /// Corresponds to `DWMWCP_DONOTROUND`.
    ///
    /// Never round window corners.
    DoNotRound = 1,

    /// Corresponds to `DWMWCP_ROUND`.
    ///
    /// Round the corners, if appropriate.
    Round = 2,

    /// Corresponds to `DWMWCP_ROUNDSMALL`.
    ///
    /// Round the corners if appropriate, with a small radius.
    RoundSmall = 3,
}
