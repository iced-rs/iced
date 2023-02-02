use std::hash::{Hash, Hasher};

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

impl Hash for Font {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        match self {
            Self::Default => {
                0.hash(hasher);
            }
            Self::External { name, .. } => {
                1.hash(hasher);
                name.hash(hasher);
            }
        }
    }
}
