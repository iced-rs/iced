use crate::core::{Font, Pixels};
use crate::Backend;

/// The settings of a renderer.
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    /// The default [`Font`] to use.
    pub default_font: Font,

    /// The default size of text.
    ///
    /// By default, it will be set to `16.0`.
    pub default_text_size: Pixels,

    /// The graphical backend to use.
    ///
    /// It defaults to [`Backend::Best`].
    pub backend: Backend,

    /// If set to true, the renderer will try to perform antialiasing for some
    /// primitives.
    ///
    /// Enabling it can produce a smoother result in some widgets, like the
    /// `Canvas`, at a performance cost.
    ///
    /// By default, it is disabled.
    pub antialiasing: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            backend: Backend::default(),
            default_font: Font::default(),
            default_text_size: Pixels(16.0),
            antialiasing: false,
        }
    }
}
