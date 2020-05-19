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

    /// The antialiasing strategy that will be used for triangle primitives.
    pub antialiasing: Option<Antialiasing>,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            default_font: None,
            antialiasing: None,
        }
    }
}

/// An antialiasing strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Antialiasing {
    /// Multisample AA with 2 samples
    MSAAx2,
    /// Multisample AA with 4 samples
    MSAAx4,
    /// Multisample AA with 8 samples
    MSAAx8,
    /// Multisample AA with 16 samples
    MSAAx16,
}

impl Antialiasing {
    pub(crate) fn sample_count(self) -> u32 {
        match self {
            Antialiasing::MSAAx2 => 2,
            Antialiasing::MSAAx4 => 4,
            Antialiasing::MSAAx8 => 8,
            Antialiasing::MSAAx16 => 16,
        }
    }
}
