use crate::{Font, Point, Size};

use iced_graphics::backend;
use iced_graphics::text;

use std::borrow::Cow;

pub enum Backend {
    Wgpu(iced_wgpu::Backend),
}

impl iced_graphics::Backend for Backend {}

impl backend::Text for Backend {
    const ICON_FONT: Font = Font::Name("Iced-Icons");
    const CHECKMARK_ICON: char = '\u{f00c}';
    const ARROW_DOWN_ICON: char = '\u{e800}';

    fn default_font(&self) -> Font {
        match self {
            Self::Wgpu(backend) => backend.default_font(),
        }
    }

    fn default_size(&self) -> f32 {
        match self {
            Self::Wgpu(backend) => backend.default_size(),
        }
    }

    fn measure(
        &self,
        contents: &str,
        size: f32,
        font: Font,
        bounds: Size,
    ) -> (f32, f32) {
        match self {
            Self::Wgpu(backend) => {
                backend.measure(contents, size, font, bounds)
            }
        }
    }

    fn hit_test(
        &self,
        contents: &str,
        size: f32,
        font: Font,
        bounds: Size,
        position: Point,
        nearest_only: bool,
    ) -> Option<text::Hit> {
        match self {
            Self::Wgpu(backend) => backend.hit_test(
                contents,
                size,
                font,
                bounds,
                position,
                nearest_only,
            ),
        }
    }

    fn load_font(&mut self, font: Cow<'static, [u8]>) {
        match self {
            Self::Wgpu(backend) => {
                backend.load_font(font);
            }
        }
    }
}

#[cfg(feature = "image")]
impl backend::Image for Backend {
    fn dimensions(&self, handle: &iced_native::image::Handle) -> Size<u32> {
        match self {
            Self::Wgpu(backend) => backend.dimensions(handle),
        }
    }
}

#[cfg(feature = "svg")]
impl backend::Svg for Backend {
    fn viewport_dimensions(
        &self,
        handle: &iced_native::svg::Handle,
    ) -> Size<u32> {
        match self {
            Self::Wgpu(backend) => backend.viewport_dimensions(handle),
        }
    }
}