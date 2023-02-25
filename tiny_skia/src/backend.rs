use crate::{Color, Font, Settings, Size, Viewport};

use iced_graphics::backend;
use iced_graphics::text;
use iced_graphics::Primitive;

use std::borrow::Cow;

pub struct Backend {
    default_font: Font,
    default_text_size: f32,
}

impl Backend {
    pub fn new(settings: Settings) -> Self {
        Self {
            default_font: settings.default_font,
            default_text_size: settings.default_text_size,
        }
    }

    pub fn draw<T: AsRef<str>>(
        &mut self,
        pixels: &mut tiny_skia::Pixmap,
        _primitives: &[Primitive],
        _viewport: &Viewport,
        background_color: Color,
        _overlay: &[T],
    ) {
        pixels.fill(into_color(background_color));
    }
}

fn into_color(color: Color) -> tiny_skia::Color {
    tiny_skia::Color::from_rgba(color.r, color.g, color.b, color.a)
        .expect("Convert color from iced to tiny_skia")
}

impl iced_graphics::Backend for Backend {
    fn trim_measurements(&mut self) {
        // TODO
    }
}

impl backend::Text for Backend {
    const ICON_FONT: Font = Font::Name("Iced-Icons");
    const CHECKMARK_ICON: char = '\u{f00c}';
    const ARROW_DOWN_ICON: char = '\u{e800}';

    fn default_font(&self) -> Font {
        self.default_font
    }

    fn default_size(&self) -> f32 {
        self.default_text_size
    }

    fn measure(
        &self,
        _contents: &str,
        _size: f32,
        _font: Font,
        _bounds: Size,
    ) -> (f32, f32) {
        // TODO
        (0.0, 0.0)
    }

    fn hit_test(
        &self,
        _contents: &str,
        _size: f32,
        _font: Font,
        _bounds: Size,
        _point: iced_native::Point,
        _nearest_only: bool,
    ) -> Option<text::Hit> {
        // TODO
        None
    }

    fn load_font(&mut self, _font: Cow<'static, [u8]>) {
        // TODO
    }
}

#[cfg(feature = "image")]
impl backend::Image for Backend {
    fn dimensions(&self, _handle: &iced_native::image::Handle) -> Size<u32> {
        // TODO
        Size::new(0, 0)
    }
}

#[cfg(feature = "svg")]
impl backend::Svg for Backend {
    fn viewport_dimensions(
        &self,
        _handle: &iced_native::svg::Handle,
    ) -> Size<u32> {
        // TODO
        Size::new(0, 0)
    }
}
