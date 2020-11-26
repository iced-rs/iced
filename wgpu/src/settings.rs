//! Configure a renderer.
pub use crate::Antialiasing;

/// The settings of a [`Backend`].
///
/// [`Backend`]: crate::Backend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    /// The output format of the [`Backend`].
    ///
    /// [`Backend`]: crate::Backend
    pub format: wgpu::TextureFormat,

    /// The present mode of the [`Backend`].
    ///
    /// [`Backend`]: crate::Backend
    pub present_mode: wgpu::PresentMode,

    /// The bytes of the font that will be used by default.
    ///
    /// If `None` is provided, a default system font will be chosen.
    pub default_font: Option<&'static [u8]>,

    /// The default size of text.
    ///
    /// By default, it will be set to 20.
    pub default_text_size: u16,

    /// The antialiasing strategy that will be used for triangle primitives.
    pub antialiasing: Option<Antialiasing>,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            present_mode: wgpu::PresentMode::Mailbox,
            default_font: None,
            default_text_size: 20,
            antialiasing: None,
        }
    }
}
