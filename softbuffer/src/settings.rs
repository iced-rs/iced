//! Configure a renderer.
pub use iced_graphics::Antialiasing;

/// The settings of a [`Backend`].
///
/// [`Backend`]: crate::Backend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    /// The bytes of the font that will be used by default.
    ///
    /// If `None` is provided, a default system font will be chosen.
    pub default_font: Option<&'static [u8]>,

    /// The default size of text.
    ///
    /// By default, it will be set to 20.
    pub default_text_size: u16,

    /// If enabled, spread text workload in multiple threads when multiple cores
    /// are available.
    ///
    /// By default, it is disabled.
    pub text_multithreading: bool,

    /// The antialiasing strategy that will be used for triangle primitives.
    ///
    /// By default, it is `None`.
    pub antialiasing: Option<Antialiasing>,
}

impl Settings {
    /// Creates new [`Settings`] using environment configuration.
    pub fn from_env() -> Self {
        Settings::default()
    }
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            default_font: None,
            default_text_size: 20,
            text_multithreading: false,
            antialiasing: None,
        }
    }
}
