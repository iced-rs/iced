/// The settings of a [`Renderer`].
///
/// [`Renderer`]: struct.Renderer.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Settings {
    /// The bytes of the font that will be used by default.
    ///
    /// If `None` is provided, a default system font will be chosen.
    pub default_font: Option<&'static [u8]>,

    /// The antialiasing strategy that will be used for triangle primitives.
    pub antialiasing: Option<MSAA>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MSAA {
    X2,
    X4,
    X8,
    X16,
}

impl MSAA {
    pub(crate) fn sample_count(&self) -> u32 {
        match self {
            MSAA::X2 => 2,
            MSAA::X4 => 4,
            MSAA::X8 => 8,
            MSAA::X16 => 16,
        }
    }
}
