//! Platform specific settings for macOS.

/// The platform specific application settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformSpecific {
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
            activation_policy: ActivationPolicy::default(),
            activate_ignoring_other_apps: true,
        }
    }
}

/// Activation policies that control whether and how an app may be activated.
/// Corresponds to [`NSApplicationActivationPolicy`].
///
/// [`NSApplicationActivationPolicy`]: https://developer.apple.com/documentation/appkit/nsapplication/activationpolicy-swift.enum?language=objc
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
