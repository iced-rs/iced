//! Configure a renderer.
pub use crate::Antialiasing;

/// The settings of a [`Renderer`].
///
/// [`Renderer`]: ../struct.Renderer.html
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    /// The output format of the [`Renderer`].
    ///
    /// [`Renderer`]: ../struct.Renderer.html
    pub format: wgpu::TextureFormat,

    /// The bytes of the font that will be used by default.
    ///
    /// If `None` is provided, a default system font will be chosen.
    pub default_font: Option<&'static [u8]>,

    /// The antialiasing strategy that will be used for triangle primitives.
    pub antialiasing: Option<Antialiasing>,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            default_font: None,
            antialiasing: None,
        }
    }
}
