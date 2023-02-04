use crate::quad;
use crate::text;
use crate::triangle;
use crate::{Settings, Transformation};

use iced_graphics::backend;
use iced_graphics::layer::Layer;
use iced_graphics::{Primitive, Viewport};
use iced_native::{Font, Size};

#[cfg(feature = "tracing")]
use tracing::info_span;

#[cfg(any(feature = "image", feature = "svg"))]
use crate::image;

use std::borrow::Cow;

/// A [`wgpu`] graphics backend for [`iced`].
///
/// [`wgpu`]: https://github.com/gfx-rs/wgpu-rs
/// [`iced`]: https://github.com/iced-rs/iced
#[allow(missing_debug_implementations)]
pub struct Backend {
    quad_pipeline: quad::Pipeline,
    text_pipeline: text::Pipeline,
    triangle_pipeline: triangle::Pipeline,

    #[cfg(any(feature = "image", feature = "svg"))]
    image_pipeline: image::Pipeline,

    default_font: Font,
    default_text_size: f32,
}

impl Backend {
    /// Creates a new [`Backend`].
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        settings: Settings,
        format: wgpu::TextureFormat,
    ) -> Self {
        let text_pipeline = text::Pipeline::new(device, queue, format);
        let quad_pipeline = quad::Pipeline::new(device, format);
        let triangle_pipeline =
            triangle::Pipeline::new(device, format, settings.antialiasing);

        #[cfg(any(feature = "image", feature = "svg"))]
        let image_pipeline = image::Pipeline::new(device, format);

        Self {
            quad_pipeline,
            text_pipeline,
            triangle_pipeline,

            #[cfg(any(feature = "image", feature = "svg"))]
            image_pipeline,

            default_font: settings.default_font,
            default_text_size: settings.default_text_size,
        }
    }

    /// Draws the provided primitives in the given `TextureView`.
    ///
    /// The text provided as overlay will be rendered on top of the primitives.
    /// This is useful for rendering debug information.
    pub fn present<T: AsRef<str>>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        frame: &wgpu::TextureView,
        primitives: &[Primitive],
        viewport: &Viewport,
        overlay_text: &[T],
    ) {
        log::debug!("Drawing");
        #[cfg(feature = "tracing")]
        let _ = info_span!("Wgpu::Backend", "PRESENT").entered();

        let target_size = viewport.physical_size();
        let scale_factor = viewport.scale_factor() as f32;
        let transformation = viewport.projection();

        let mut layers = Layer::generate(primitives, viewport);
        layers.push(Layer::overlay(overlay_text, viewport));

        for layer in layers {
            self.flush(
                device,
                queue,
                scale_factor,
                transformation,
                &layer,
                staging_belt,
                encoder,
                frame,
                target_size,
            );
        }

        self.text_pipeline.end_frame();

        #[cfg(any(feature = "image", feature = "svg"))]
        self.image_pipeline.trim_cache(device, encoder);
    }

    fn flush(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scale_factor: f32,
        transformation: Transformation,
        layer: &Layer<'_>,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        target_size: Size<u32>,
    ) {
        let bounds = (layer.bounds * scale_factor).snap();

        if bounds.width < 1 || bounds.height < 1 {
            return;
        }

        if !layer.quads.is_empty() {
            self.quad_pipeline.draw(
                device,
                staging_belt,
                encoder,
                &layer.quads,
                transformation,
                scale_factor,
                bounds,
                target,
            );
        }

        if !layer.meshes.is_empty() {
            let scaled = transformation
                * Transformation::scale(scale_factor, scale_factor);

            self.triangle_pipeline.draw(
                device,
                staging_belt,
                encoder,
                target,
                target_size,
                scaled,
                scale_factor,
                &layer.meshes,
            );
        }

        #[cfg(any(feature = "image", feature = "svg"))]
        {
            if !layer.images.is_empty() {
                let scaled = transformation
                    * Transformation::scale(scale_factor, scale_factor);

                self.image_pipeline.draw(
                    device,
                    staging_belt,
                    encoder,
                    &layer.images,
                    scaled,
                    bounds,
                    target,
                    scale_factor,
                );
            }
        }

        if !layer.text.is_empty() {
            self.text_pipeline.prepare(
                device,
                queue,
                &layer.text,
                layer.bounds,
                scale_factor,
                target_size,
            );

            self.text_pipeline.render(encoder, target);
        }
    }
}

impl iced_graphics::Backend for Backend {
    fn trim_measurements(&mut self) {
        self.text_pipeline.trim_measurement_cache()
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
        contents: &str,
        size: f32,
        font: Font,
        bounds: Size,
    ) -> (f32, f32) {
        self.text_pipeline.measure(contents, size, font, bounds)
    }

    fn hit_test(
        &self,
        contents: &str,
        size: f32,
        font: Font,
        bounds: Size,
        point: iced_native::Point,
        nearest_only: bool,
    ) -> Option<text::Hit> {
        self.text_pipeline.hit_test(
            contents,
            size,
            font,
            bounds,
            point,
            nearest_only,
        )
    }

    fn load_font(&mut self, font: Cow<'static, [u8]>) {
        self.text_pipeline.load_font(font);
    }
}

#[cfg(feature = "image")]
impl backend::Image for Backend {
    fn dimensions(&self, handle: &iced_native::image::Handle) -> Size<u32> {
        self.image_pipeline.dimensions(handle)
    }
}

#[cfg(feature = "svg")]
impl backend::Svg for Backend {
    fn viewport_dimensions(
        &self,
        handle: &iced_native::svg::Handle,
    ) -> Size<u32> {
        self.image_pipeline.viewport_dimensions(handle)
    }
}
