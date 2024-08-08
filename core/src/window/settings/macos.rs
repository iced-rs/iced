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
    /// Activation policy for the application.
    pub activation_policy: ActivationPolicy,
    /// Used to prevent the application from automatically activating when launched if
    /// another application is already active.
    ///
    /// The default behavior is to ignore other applications and activate when launched.
    pub activate_ignoring_other_apps: bool,
}

impl Default for PlatformSpecific {
    fn default() -> Self {
        Self {
            title_hidden: false,
            titlebar_transparent: false,
            fullsize_content_view: false,
            activation_policy: Default::default(), // Regular
            activate_ignoring_other_apps: true,
        }
    }
}

/// Corresponds to `NSApplicationActivationPolicy`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActivationPolicy {
    /// Corresponds to `NSApplicationActivationPolicyRegular`.
    #[default]
    Regular,
    /// Corresponds to `NSApplicationActivationPolicyAccessory`.
    Accessory,
    /// Corresponds to `NSApplicationActivationPolicyProhibited`.
    Prohibited,
}
