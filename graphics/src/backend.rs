use iced_native::image;
use iced_native::svg;
use iced_native::{Font, Size};

pub trait Backend {
    fn trim_measurements(&mut self) {}
}

pub trait Text {
    const ICON_FONT: Font;
    const CHECKMARK_ICON: char;

    fn measure(
        &self,
        contents: &str,
        size: f32,
        font: Font,
        bounds: Size,
    ) -> (f32, f32);
}

pub trait Image {
    fn dimensions(&self, handle: &image::Handle) -> (u32, u32);
}

pub trait Svg {
    fn viewport_dimensions(&self, handle: &svg::Handle) -> (u32, u32);
}
