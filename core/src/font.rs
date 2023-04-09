/// A font.
#[derive(Debug, Clone, Copy)]
pub enum Font {
    /// The default font.
    ///
    /// This is normally a font configured in a renderer or loaded from the
    /// system.
    Default,

    /// An external font.
    External {
        /// The name of the external font
        name: &'static str,

        /// The bytes of the external font
        bytes: &'static [u8],
    },
}

impl Default for Font {
    fn default() -> Font {
        Font::Default
    }
}
