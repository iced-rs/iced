use crate::core;
use crate::core::{Color, Font, Point, Size};
use crate::graphics::backend;
use crate::graphics::color;
use crate::graphics::{Transformation, Viewport};
use crate::primitive::{self, Primitive};
use crate::quad;
use crate::text;
use crate::triangle;
use crate::{Layer, Settings};

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
        encoder: &mut wgpu::CommandEncoder,
        clear_color: Option<Color>,
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

        self.prepare(
            device,
            queue,
            encoder,
            scale_factor,
            target_size,
            transformation,
            &layers,
        );

        self.render(
            device,
            encoder,
            frame,
            clear_color,
            scale_factor,
            target_size,
            &layers,
        );

        self.quad_pipeline.end_frame();
        self.text_pipeline.end_frame();
        self.triangle_pipeline.end_frame();

        #[cfg(any(feature = "image", feature = "svg"))]
        self.image_pipeline.end_frame();
    }

    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _encoder: &mut wgpu::CommandEncoder,
        scale_factor: f32,
        target_size: Size<u32>,
        transformation: Transformation,
        layers: &[Layer<'_>],
    ) {
        for layer in layers {
            let bounds = (layer.bounds * scale_factor).snap();

            if bounds.width < 1 || bounds.height < 1 {
                continue;
            }

            if !layer.quads.is_empty() {
                self.quad_pipeline.prepare(
                    device,
                    queue,
                    &layer.quads,
                    transformation,
                    scale_factor,
                );
            }

            if !layer.meshes.is_empty() {
                let scaled = transformation
                    * Transformation::scale(scale_factor, scale_factor);

                self.triangle_pipeline.prepare(
                    device,
                    queue,
                    &layer.meshes,
                    scaled,
                );
            }

            #[cfg(any(feature = "image", feature = "svg"))]
            {
                if !layer.images.is_empty() {
                    let scaled = transformation
                        * Transformation::scale(scale_factor, scale_factor);

                    self.image_pipeline.prepare(
                        device,
                        queue,
                        _encoder,
                        &layer.images,
                        scaled,
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
            }
        }
    }

    fn render(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clear_color: Option<Color>,
        scale_factor: f32,
        target_size: Size<u32>,
        layers: &[Layer<'_>],
    ) {
        use std::mem::ManuallyDrop;

        let mut quad_layer = 0;
        let mut triangle_layer = 0;
        #[cfg(any(feature = "image", feature = "svg"))]
        let mut image_layer = 0;
        let mut text_layer = 0;

        let mut render_pass = ManuallyDrop::new(encoder.begin_render_pass(
            &wgpu::RenderPassDescriptor {
                label: Some("iced_wgpu::quad render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: match clear_color {
                            Some(background_color) => wgpu::LoadOp::Clear({
                                let [r, g, b, a] =
                                    color::pack(background_color).components();

                                wgpu::Color {
                                    r: f64::from(r),
                                    g: f64::from(g),
                                    b: f64::from(b),
                                    a: f64::from(a),
                                }
                            }),
                            None => wgpu::LoadOp::Load,
                        },
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            },
        ));

        for layer in layers {
            let bounds = (layer.bounds * scale_factor).snap();

            if bounds.width < 1 || bounds.height < 1 {
                continue;
            }

            if !layer.quads.is_empty() {
                self.quad_pipeline.render(
                    quad_layer,
                    bounds,
                    &layer.quads,
                    &mut render_pass,
                );

                quad_layer += 1;
            }

            if !layer.meshes.is_empty() {
                let _ = ManuallyDrop::into_inner(render_pass);

                self.triangle_pipeline.render(
                    device,
                    encoder,
                    target,
                    triangle_layer,
                    target_size,
                    &layer.meshes,
                    scale_factor,
                );

                triangle_layer += 1;

                render_pass = ManuallyDrop::new(encoder.begin_render_pass(
                    &wgpu::RenderPassDescriptor {
                        label: Some("iced_wgpu::quad render pass"),
                        color_attachments: &[Some(
                            wgpu::RenderPassColorAttachment {
                                view: target,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: true,
                                },
                            },
                        )],
                        depth_stencil_attachment: None,
                    },
                ));
            }

            #[cfg(any(feature = "image", feature = "svg"))]
            {
                if !layer.images.is_empty() {
                    self.image_pipeline.render(
                        image_layer,
                        bounds,
                        &mut render_pass,
                    );

                    image_layer += 1;
                }
            }

            if !layer.text.is_empty() {
                self.text_pipeline
                    .render(text_layer, bounds, &mut render_pass);

                text_layer += 1;
            }
        }

        let _ = ManuallyDrop::into_inner(render_pass);
    }
}

impl crate::graphics::Backend for Backend {
    type Primitive = primitive::Custom;

    fn trim_measurements(&mut self) {
        self.text_pipeline.trim_measurements();
    }
}

impl backend::Text for Backend {
    const ICON_FONT: Font = Font::with_name("Iced-Icons");
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
        line_height: core::text::LineHeight,
        font: Font,
        bounds: Size,
        shaping: core::text::Shaping,
    ) -> Size {
        self.text_pipeline.measure(
            contents,
            size,
            line_height,
            font,
            bounds,
            shaping,
        )
    }

    fn hit_test(
        &self,
        contents: &str,
        size: f32,
        line_height: core::text::LineHeight,
        font: Font,
        bounds: Size,
        shaping: core::text::Shaping,
        point: Point,
        nearest_only: bool,
    ) -> Option<core::text::Hit> {
        self.text_pipeline.hit_test(
            contents,
            size,
            line_height,
            font,
            bounds,
            shaping,
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
    fn dimensions(&self, handle: &core::image::Handle) -> Size<u32> {
        self.image_pipeline.dimensions(handle)
    }
}

#[cfg(feature = "svg")]
impl backend::Svg for Backend {
    fn viewport_dimensions(&self, handle: &core::svg::Handle) -> Size<u32> {
        self.image_pipeline.viewport_dimensions(handle)
    }
}
