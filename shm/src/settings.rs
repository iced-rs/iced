//! Configure a [`Renderer`].
//!
//! [`Renderer`]: struct.Renderer.html

/// The settings of a [`Renderer`].
///
/// [`Renderer`]: ../struct.Renderer.html
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    /// The bytes of the font that will be used by default.
    ///
    /// If `None` is provided, a default system font will be chosen.
    pub default_font: Option<&'static [u8]>,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings { default_font: None }
    }
}
