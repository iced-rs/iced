//! Platform specific settings for WebAssembly.

/// The platform specific window settings of an application.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PlatformSpecific {
    /// The identifier of a DOM element that will be replaced with the
    /// application.
    ///
    /// If set to `None`, the application will be appended to the HTML body.
    pub target: Option<String>,
}
