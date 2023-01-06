pub use iced_native::text::Hit;

#[derive(Debug)]
pub struct Pipeline;

impl Pipeline {
    pub fn new(
        _device: &wgpu::Device,
        _format: wgpu::TextureFormat,
        _default_font: Option<&[u8]>,
        _multithreading: bool,
    ) -> Self {
        Pipeline
    }

    pub fn measure(
        &self,
        _content: &str,
        _size: f32,
        _font: iced_native::Font,
        _bounds: iced_native::Size,
    ) -> (f32, f32) {
        (0.0, 0.0)
    }

    pub fn hit_test(
        &self,
        _content: &str,
        _size: f32,
        _font: iced_native::Font,
        _bounds: iced_native::Size,
        _point: iced_native::Point,
        _nearest_only: bool,
    ) -> Option<Hit> {
        None
    }

    pub fn trim_measurement_cache(&mut self) {}
}
