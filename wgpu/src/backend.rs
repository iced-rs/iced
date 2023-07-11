use crate::core::renderer::Effect;
use crate::core::{Color, Font, Point, Rectangle, Size};
use crate::graphics::backend;
use crate::graphics::color;
use crate::graphics::{Transformation, Viewport};
use crate::primitive::{self, Primitive};
use crate::triangle;
use crate::{blur, composite, core, layer, quad, text};
use crate::{Layer, Settings};

#[cfg(any(feature = "image", feature = "svg"))]
use crate::image;

use std::borrow::Cow;
use std::mem::ManuallyDrop;

/// A [`wgpu`] graphics backend for [`iced`].
///
/// [`wgpu`]: https://github.com/gfx-rs/wgpu-rs
/// [`iced`]: https://github.com/iced-rs/iced
#[allow(missing_debug_implementations)]
pub struct Backend {
    quad_pipeline: quad::Pipeline,
    text_pipeline: text::Pipeline,
    triangle_pipeline: triangle::Pipeline,
    blur_pipeline: blur::Pipeline,
    composite_pipeline: composite::Pipeline,

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
        let blur_pipeline = blur::Pipeline::new(device, format);
        let composite_pipeline = composite::Pipeline::new(device, format);

        #[cfg(any(feature = "image", feature = "svg"))]
        let image_pipeline = image::Pipeline::new(device, format);

        Self {
            quad_pipeline,
            text_pipeline,
            triangle_pipeline,

            blur_pipeline,
            composite_pipeline,
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
        format: wgpu::TextureFormat,
        frame: &wgpu::TextureView,
        primitives: &[Primitive],
        viewport: &Viewport,
        overlay_text: &[T],
    ) {
        log::debug!("Drawing");

        let viewport_size = viewport.physical_size();
        let scale_factor = viewport.scale_factor() as f32;
        let viewport_transform = viewport.projection();

        let mut layers = Layer::generate(primitives, viewport);
        layers.push(Layer::overlay(overlay_text, viewport));

        self.prepare(
            device,
            queue,
            encoder,
            format,
            scale_factor,
            viewport_size,
            viewport_transform,
            &layers,
        );

        self.render(
            device,
            encoder,
            frame,
            clear_color,
            viewport_size,
            scale_factor,
            &layers,
        );

        self.quad_pipeline.end_frame();
        self.text_pipeline.end_frame();
        self.triangle_pipeline.end_frame();

        #[cfg(any(feature = "image", feature = "svg"))]
        self.image_pipeline.end_frame();

        self.blur_pipeline.end_frame();
        self.composite_pipeline.end_frame();
    }

    fn prepare_text(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scale_factor: f32,
        surface_bounds: Rectangle,
        surface_size: Size<u32>,
        text: &[layer::Text<'_>],
    ) -> bool {
        if !text.is_empty()
            && !self.text_pipeline.prepare(
                device,
                queue,
                text,
                surface_bounds,
                scale_factor,
                surface_size,
            )
        {
            return false;
        }

        true
    }

    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _encoder: &mut wgpu::CommandEncoder,
        format: wgpu::TextureFormat,
        scale_factor: f32,
        viewport_size: Size<u32>,
        viewport_transform: Transformation,
        layers: &[Layer<'_>],
    ) {
        for layer in layers {
            let bounds = (layer.bounds * scale_factor).snap();

            if bounds.width < 1 || bounds.height < 1 {
                continue;
            }

            match &layer.kind {
                layer::Kind::Immediate => {
                    self.prepare_layer(
                        layer,
                        device,
                        queue,
                        _encoder,
                        scale_factor,
                        viewport_transform,
                        viewport_size,
                        layer.bounds,
                    );
                }
                layer::Kind::Deferred { effect } => {
                    let surface = match effect {
                        Effect::Blur { radius } => self
                            .blur_pipeline
                            .prepare(device, queue, format, *radius, bounds),
                    };

                    let surface_transform = surface.transform();
                    let surface_size = surface.texture_size();
                    let scale_factor = scale_factor * surface.ratio();

                    self.composite_pipeline.prepare(
                        device,
                        queue,
                        viewport_transform,
                        surface.clip.into(),
                        &surface.view,
                    );

                    self.prepare_layer(
                        layer,
                        device,
                        queue,
                        _encoder,
                        scale_factor,
                        surface_transform,
                        surface_size,
                        Rectangle::with_size(layer.bounds.size()),
                    );
                }
            }
        }
    }

    fn render(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        frame: &wgpu::TextureView,
        clear_color: Option<Color>,
        surface_size: Size<u32>,
        scale_factor: f32,
        layers: &[Layer<'_>],
    ) {
        let mut quad_count = 0;
        let mut mesh_count = 0;
        #[cfg(any(feature = "image", feature = "svg"))]
        let mut image_count = 0;

        let mut text_count = 0;
        let mut blur_count = 0;
        let mut composite_count = 0;

        let mut pass = Pass::new(encoder, frame, clear_color);

        for layer in layers {
            let bounds = (layer.bounds * scale_factor).snap();

            if bounds.width < 1 || bounds.height < 1 {
                return;
            }

            match &layer.kind {
                layer::Kind::Immediate => {
                    //Render directly to frame
                    if !layer.quads.is_empty() {
                        self.quad_pipeline.render(
                            quad_count,
                            bounds,
                            &layer.quads,
                            pass.inner(),
                        );

                        quad_count += 1;
                    }

                    #[cfg(any(feature = "image", feature = "svg"))]
                    {
                        if !layer.images.is_empty() {
                            self.image_pipeline.render(
                                image_count,
                                bounds,
                                pass.inner(),
                            );

                            image_count += 1;
                        }
                    }

                    if !layer.text.is_empty() {
                        self.text_pipeline.render(
                            text_count,
                            bounds,
                            pass.inner(),
                        );

                        text_count += 1;
                    }

                    if !layer.meshes.is_empty() {
                        pass.kill();

                        self.triangle_pipeline.render(
                            device,
                            encoder,
                            frame,
                            mesh_count,
                            surface_size,
                            &layer.meshes,
                            scale_factor,
                        );

                        mesh_count += 1;

                        pass = Pass::init(
                            encoder,
                            frame,
                            wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: true,
                            },
                            "post_mesh_frame_pass",
                        );
                    }
                }
                layer::Kind::Deferred { effect } => {
                    let surface = match effect {
                        Effect::Blur { .. } => {
                            self.blur_pipeline.surface(blur_count)
                        }
                    };

                    if let Some(surface) = surface {
                        pass.kill();

                        let surface_view = &surface.view;
                        let texture_bounds = surface.scissor();
                        let ratio = surface.ratio();

                        // recreate the pass targeting the new surface
                        pass = Pass::init(
                            encoder,
                            surface_view,
                            wgpu::Operations {
                                load: wgpu::LoadOp::Clear(
                                    wgpu::Color::TRANSPARENT,
                                ),
                                store: true,
                            },
                            "composite_surface_pass",
                        );

                        // render everything in this layer to the surface
                        if !layer.quads.is_empty() {
                            self.quad_pipeline.render(
                                quad_count,
                                texture_bounds,
                                &layer.quads,
                                pass.inner(),
                            );

                            quad_count += 1;
                        }

                        #[cfg(any(feature = "image", feature = "svg"))]
                        {
                            if !layer.images.is_empty() {
                                self.image_pipeline.render(
                                    image_count,
                                    texture_bounds,
                                    pass.inner(),
                                );

                                image_count += 1;
                            }
                        }

                        if !layer.text.is_empty() {
                            self.text_pipeline.render(
                                text_count,
                                texture_bounds,
                                pass.inner(),
                            );

                            text_count += 1;
                        }

                        pass.kill();

                        if !layer.meshes.is_empty() {
                            self.triangle_pipeline.render(
                                device,
                                encoder,
                                surface_view,
                                mesh_count,
                                surface.texture_size(),
                                &layer.meshes,
                                scale_factor * ratio,
                            );

                            mesh_count += 1;
                        }

                        // run effect pipeline with the surface
                        match effect {
                            Effect::Blur { .. } => {
                                self.blur_pipeline.render(blur_count, encoder);

                                blur_count += 1;
                            }
                        };

                        // recreate the render pass targeting the frame
                        pass = Pass::init(
                            encoder,
                            frame,
                            wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: true,
                            },
                            "post_composite_frame_pass",
                        );

                        //now we composite the surface on to the frame
                        self.composite_pipeline
                            .render(pass.inner(), composite_count);

                        composite_count += 1;
                    }
                }
            }
        }

        pass.kill();
    }

    fn prepare_layer(
        &mut self,
        layer: &Layer<'_>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _encoder: &mut wgpu::CommandEncoder,
        scale_factor: f32,
        transform: Transformation,
        surface_size: Size<u32>,
        surface_bounds: Rectangle,
    ) {
        if !layer.quads.is_empty() {
            self.quad_pipeline.prepare(
                device,
                queue,
                &layer.quads,
                transform,
                scale_factor,
            );
        }

        if !layer.meshes.is_empty() {
            //TODO matrix math -> shader
            let scaled =
                transform * Transformation::scale(scale_factor, scale_factor);

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
                //TODO matrix math -> shader
                let scaled = transform
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

        while !self.prepare_text(
            device,
            queue,
            scale_factor,
            surface_bounds,
            surface_size,
            &layer.text,
        ) {}
    }
}

#[derive(Debug)]
struct Pass<'b> {
    pass: ManuallyDrop<wgpu::RenderPass<'b>>,
}

impl<'a, 'b> Pass<'b> {
    pub fn new(
        encoder: &'a mut wgpu::CommandEncoder,
        frame: &'a wgpu::TextureView,
        clear_color: Option<Color>,
    ) -> Self
    where
        'a: 'b,
    {
        Self {
            pass: ManuallyDrop::new(encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("iced_wgpu.pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: frame,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: match clear_color {
                                    Some(background_color) => {
                                        wgpu::LoadOp::Clear({
                                            let [r, g, b, a] =
                                                color::pack(background_color)
                                                    .components();

                                            wgpu::Color {
                                                r: f64::from(r),
                                                g: f64::from(g),
                                                b: f64::from(b),
                                                a: f64::from(a),
                                            }
                                        })
                                    }
                                    None => wgpu::LoadOp::Load,
                                },
                                store: true,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                },
            )),
        }
    }

    pub fn inner(&mut self) -> &mut wgpu::RenderPass<'b> {
        &mut self.pass
    }

    pub fn kill(self) {
        let _ = ManuallyDrop::into_inner(self.pass);
    }

    pub fn init(
        encoder: &'a mut wgpu::CommandEncoder,
        target: &'a wgpu::TextureView,
        ops: wgpu::Operations<wgpu::Color>,
        label: &'static str,
    ) -> Self
    where
        'a: 'b,
    {
        Self {
            pass: ManuallyDrop::new(encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some(label),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: target,
                            resolve_target: None,
                            ops,
                        },
                    )],
                    depth_stencil_attachment: None,
                },
            )),
        }
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
