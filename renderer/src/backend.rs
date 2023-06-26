use crate::core::text;
use crate::core::{Font, Point, Size};
use crate::graphics::backend;

use std::borrow::Cow;

#[allow(clippy::large_enum_variant)]
pub enum Backend {
    TinySkia(iced_tiny_skia::Backend),
    #[cfg(feature = "wgpu")]
    Wgpu(iced_wgpu::Backend),
}

macro_rules! delegate {
    ($backend:expr, $name:ident, $body:expr) => {
        match $backend {
            Self::TinySkia($name) => $body,
            #[cfg(feature = "wgpu")]
            Self::Wgpu($name) => $body,
        }
    };
}

impl backend::Text for Backend {
    const ICON_FONT: Font = Font::with_name("Iced-Icons");
    const CHECKMARK_ICON: char = '\u{f00c}';
    const ARROW_DOWN_ICON: char = '\u{e800}';

    fn default_font(&self) -> Font {
        delegate!(self, backend, backend.default_font())
    }

    fn default_size(&self) -> f32 {
        delegate!(self, backend, backend.default_size())
    }

    fn measure(
        &self,
        contents: &str,
        size: f32,
        line_height: text::LineHeight,
        font: Font,
        bounds: Size,
        shaping: text::Shaping,
    ) -> (f32, f32) {
        delegate!(
            self,
            backend,
            backend.measure(contents, size, line_height, font, bounds, shaping)
        )
    }

    fn hit_test(
        &self,
        contents: &str,
        size: f32,
        line_height: text::LineHeight,
        font: Font,
        bounds: Size,
        shaping: text::Shaping,
        position: Point,
        nearest_only: bool,
    ) -> Option<text::Hit> {
        delegate!(
            self,
            backend,
            backend.hit_test(
                contents,
                size,
                line_height,
                font,
                bounds,
                shaping,
                position,
                nearest_only,
            )
        )
    }

    fn load_font(&mut self, font: Cow<'static, [u8]>) {
        delegate!(self, backend, backend.load_font(font));
    }
}

#[cfg(feature = "image")]
impl backend::Image for Backend {
    fn dimensions(&self, handle: &crate::core::image::Handle) -> Size<u32> {
        delegate!(self, backend, backend.dimensions(handle))
    }
}

#[cfg(feature = "svg")]
impl backend::Svg for Backend {
    fn viewport_dimensions(
        &self,
        handle: &crate::core::svg::Handle,
    ) -> Size<u32> {
        delegate!(self, backend, backend.viewport_dimensions(handle))
    }
}
