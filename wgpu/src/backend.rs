use crate::quad;
use crate::text;
use crate::triangle;
use crate::{Settings, Transformation};
use iced_graphics::backend;
use iced_graphics::font;
use iced_graphics::layer::Layer;
use iced_graphics::{Primitive, Viewport};
use iced_native::mouse;
use iced_native::{Font, HorizontalAlignment, Size, VerticalAlignment};

#[cfg(any(feature = "image_rs", feature = "svg"))]
use crate::image;

/// A [`wgpu`] graphics backend for [`iced`].
///
/// [`wgpu`]: https://github.com/gfx-rs/wgpu-rs
/// [`iced`]: https://github.com/hecrj/iced
#[derive(Debug)]
pub struct Backend {
    quad_pipeline: quad::Pipeline,
    text_pipeline: text::Pipeline,
    triangle_pipeline: triangle::Pipeline,

    #[cfg(any(feature = "image_rs", feature = "svg"))]
    image_pipeline: image::Pipeline,

    default_text_size: u16,
}

impl Backend {
    /// Creates a new [`Backend`].
    pub fn new(
        device: &wgpu::Device,
        settings: Settings,
        format: wgpu::TextureFormat,
    ) -> Self {
        let text_pipeline = text::Pipeline::new(
            device,
            format,
            settings.default_font,
            settings.text_multithreading,
        );

        let quad_pipeline = quad::Pipeline::new(device, format);
        let triangle_pipeline =
            triangle::Pipeline::new(device, format, settings.antialiasing);

        #[cfg(any(feature = "image_rs", feature = "svg"))]
        let image_pipeline = image::Pipeline::new(device, format);

        Self {
            quad_pipeline,
            text_pipeline,
            triangle_pipeline,

            #[cfg(any(feature = "image_rs", feature = "svg"))]
            image_pipeline,

            default_text_size: settings.default_text_size,
        }
    }

    /// Draws the provided primitives in the given `TextureView`.
    ///
    /// The text provided as overlay will be rendered on top of the primitives.
    /// This is useful for rendering debug information.
    pub fn draw<T: AsRef<str>>(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        frame: &wgpu::TextureView,
        viewport: &Viewport,
        (primitive, mouse_interaction): &(Primitive, mouse::Interaction),
        overlay_text: &[T],
    ) -> mouse::Interaction {
        log::debug!("Drawing");

        let target_size = viewport.physical_size();
        let scale_factor = viewport.scale_factor() as f32;
        let transformation = viewport.projection();

        let mut layers = Layer::generate(primitive, viewport);
        layers.push(Layer::overlay(overlay_text, viewport));

        for layer in layers {
            self.flush(
                device,
                scale_factor,
                transformation,
                &layer,
                staging_belt,
                encoder,
                &frame,
                target_size.width,
                target_size.height,
            );
        }

        #[cfg(any(feature = "image_rs", feature = "svg"))]
        self.image_pipeline.trim_cache();

        *mouse_interaction
    }

    fn flush(
        &mut self,
        device: &wgpu::Device,
        scale_factor: f32,
        transformation: Transformation,
        layer: &Layer<'_>,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        target_width: u32,
        target_height: u32,
    ) {
        let bounds = (layer.bounds * scale_factor).snap();

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
                target_width,
                target_height,
                scaled,
                scale_factor,
                &layer.meshes,
            );
        }

        #[cfg(any(feature = "image_rs", feature = "svg"))]
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
            for text in layer.text.iter() {
                // Target physical coordinates directly to avoid blurry text
                let text = wgpu_glyph::Section {
                    // TODO: We `round` here to avoid rerasterizing text when
                    // its position changes slightly. This can make text feel a
                    // bit "jumpy". We may be able to do better once we improve
                    // our text rendering/caching pipeline.
                    screen_position: (
                        (text.bounds.x * scale_factor).round(),
                        (text.bounds.y * scale_factor).round(),
                    ),
                    // TODO: Fix precision issues with some scale factors.
                    //
                    // The `ceil` here can cause some words to render on the
                    // same line when they should not.
                    //
                    // Ideally, `wgpu_glyph` should be able to compute layout
                    // using logical positions, and then apply the proper
                    // scaling when rendering. This would ensure that both
                    // measuring and rendering follow the same layout rules.
                    bounds: (
                        (text.bounds.width * scale_factor).ceil(),
                        (text.bounds.height * scale_factor).ceil(),
                    ),
                    text: vec![wgpu_glyph::Text {
                        text: text.content,
                        scale: wgpu_glyph::ab_glyph::PxScale {
                            x: text.size * scale_factor,
                            y: text.size * scale_factor,
                        },
                        font_id: self.text_pipeline.find_font(text.font),
                        extra: wgpu_glyph::Extra {
                            color: text.color,
                            z: 0.0,
                        },
                    }],
                    layout: wgpu_glyph::Layout::default()
                        .h_align(match text.horizontal_alignment {
                            HorizontalAlignment::Left => {
                                wgpu_glyph::HorizontalAlign::Left
                            }
                            HorizontalAlignment::Center => {
                                wgpu_glyph::HorizontalAlign::Center
                            }
                            HorizontalAlignment::Right => {
                                wgpu_glyph::HorizontalAlign::Right
                            }
                        })
                        .v_align(match text.vertical_alignment {
                            VerticalAlignment::Top => {
                                wgpu_glyph::VerticalAlign::Top
                            }
                            VerticalAlignment::Center => {
                                wgpu_glyph::VerticalAlign::Center
                            }
                            VerticalAlignment::Bottom => {
                                wgpu_glyph::VerticalAlign::Bottom
                            }
                        }),
                    ..Default::default()
                };

                self.text_pipeline.queue(text);
            }

            self.text_pipeline.draw_queued(
                device,
                staging_belt,
                encoder,
                target,
                transformation,
                wgpu_glyph::Region {
                    x: bounds.x,
                    y: bounds.y,
                    width: bounds.width,
                    height: bounds.height,
                },
            );
        }
    }
}

impl iced_graphics::Backend for Backend {
    fn trim_measurements(&mut self) {
        self.text_pipeline.trim_measurement_cache()
    }
}

impl backend::Text for Backend {
    const ICON_FONT: Font = font::ICONS;
    const CHECKMARK_ICON: char = font::CHECKMARK_ICON;
    const ARROW_DOWN_ICON: char = font::ARROW_DOWN_ICON;

    fn default_size(&self) -> u16 {
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
}

#[cfg(feature = "image_rs")]
impl backend::Image for Backend {
    fn dimensions(&self, handle: &iced_native::image::Handle) -> (u32, u32) {
        self.image_pipeline.dimensions(handle)
    }
}

#[cfg(feature = "svg")]
impl backend::Svg for Backend {
    fn viewport_dimensions(
        &self,
        handle: &iced_native::svg::Handle,
    ) -> (u32, u32) {
        self.image_pipeline.viewport_dimensions(handle)
    }
}
