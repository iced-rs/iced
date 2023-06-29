//! Write a graphics backend.
use iced_core::image;
use iced_core::svg;
use iced_core::text;
use iced_core::{Font, Point, Size};

use std::borrow::Cow;

/// The graphics backend of a [`Renderer`].
///
/// [`Renderer`]: crate::Renderer
pub trait Backend {
    /// The custom kind of primitives this [`Backend`] supports.
    type Primitive;

    /// Trims the measurements cache.
    ///
    /// This method is currently necessary to properly trim the text cache in
    /// `iced_wgpu` and `iced_glow` because of limitations in the text rendering
    /// pipeline. It will be removed in the future.
    fn trim_measurements(&mut self) {}
}

/// A graphics backend that supports text rendering.
pub trait Text {
    /// The icon font of the backend.
    const ICON_FONT: Font;

    /// The `char` representing a ✔ icon in the [`ICON_FONT`].
    ///
    /// [`ICON_FONT`]: Self::ICON_FONT
    const CHECKMARK_ICON: char;

    /// The `char` representing a ▼ icon in the built-in [`ICON_FONT`].
    ///
    /// [`ICON_FONT`]: Self::ICON_FONT
    const ARROW_DOWN_ICON: char;

    /// Returns the default [`Font`].
    fn default_font(&self) -> Font;

    /// Returns the default size of text.
    fn default_size(&self) -> f32;

    /// Measures the text contents with the given size and font,
    /// returning the size of a laid out paragraph that fits in the provided
    /// bounds.
    fn measure(
        &self,
        contents: &str,
        size: f32,
        line_height: text::LineHeight,
        font: Font,
        bounds: Size,
        shaping: text::Shaping,
    ) -> Size;

    /// Tests whether the provided point is within the boundaries of [`Text`]
    /// laid out with the given parameters, returning information about
    /// the nearest character.
    ///
    /// If nearest_only is true, the hit test does not consider whether the
    /// the point is interior to any glyph bounds, returning only the character
    /// with the nearest centeroid.
    fn hit_test(
        &self,
        contents: &str,
        size: f32,
        line_height: text::LineHeight,
        font: Font,
        bounds: Size,
        shaping: text::Shaping,
        point: Point,
        nearest_only: bool,
    ) -> Option<text::Hit>;

    /// Loads a [`Font`] from its bytes.
    fn load_font(&mut self, font: Cow<'static, [u8]>);
}

/// A graphics backend that supports image rendering.
pub trait Image {
    /// Returns the dimensions of the provided image.
    fn dimensions(&self, handle: &image::Handle) -> Size<u32>;
}

/// A graphics backend that supports SVG rendering.
pub trait Svg {
    /// Returns the viewport dimensions of the provided SVG.
    fn viewport_dimensions(&self, handle: &svg::Handle) -> Size<u32>;
}
