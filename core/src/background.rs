use crate::image::Handle;
use crate::Color;

/// The background of some element.
#[derive(Debug, Clone, PartialEq)]
pub enum Background {
    /// A solid color
    Color(Color),
    /// A background image
    Image(Handle, BackgroundImagePosition),
    // TODO: Add gradient variant
}

/// How a background image is resized to fit/fill its container
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BackgroundImagePosition {
    // There's currently only a single option.
    // In the future, we could want to also add the following:
    // /// Stretch (or shrink) the image so that it fits in its container in one dimension, the size of the other dimension being smaller or equal as the container
    // Fit,
    // /// Stretch (or shrink) the image so that it fills its container in one dimension, the other dimension being clipped
    // Fill,
    //
    // Or, we could even do more-or-less the same as what CSS provides, i.e.
    // * background-repeat
    // * background-origin
    // * background-position
    // * background-size
    /// Stretch (or shrink) the background so that it fills its container.
    /// The aspect ratio is not preserved
    Stretch,
}

impl From<Color> for Background {
    fn from(color: Color) -> Self {
        Background::Color(color)
    }
}

impl From<Color> for Option<Background> {
    fn from(color: Color) -> Self {
        Some(Background::from(color))
    }
}
