//! Platform specific settings for WebAssembly.

/// The platform specific window settings of an application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformSpecific {
    /// The identifier of a DOM element that will be replaced with the
    /// application.
    ///
    /// If set to `None`, the application will be appended to the HTML body.
    ///
    /// By default, it is set to `"iced"`.
    pub target: Option<String>,
}

impl Default for PlatformSpecific {
    fn default() -> Self {
        Self {
            target: Some(String::from("iced")),
        }
    }
}
