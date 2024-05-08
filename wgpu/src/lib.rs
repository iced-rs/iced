//! A [`wgpu`] renderer for [Iced].
//!
//! ![The native path of the Iced ecosystem](https://github.com/iced-rs/iced/blob/0525d76ff94e828b7b21634fa94a747022001c83/docs/graphs/native.png?raw=true)
//!
//! [`wgpu`] supports most modern graphics backends: Vulkan, Metal, DX11, and
//! DX12 (OpenGL and WebGL are still WIP). Additionally, it will support the
//! incoming [WebGPU API].
//!
//! Currently, `iced_wgpu` supports the following primitives:
//! - Text, which is rendered using [`glyphon`].
//! - Quads or rectangles, with rounded borders and a solid background color.
//! - Clip areas, useful to implement scrollables or hide overflowing content.
//! - Images and SVG, loaded from memory or the file system.
//! - Meshes of triangles, useful to draw geometry freely.
//!
//! [Iced]: https://github.com/iced-rs/iced
//! [`wgpu`]: https://github.com/gfx-rs/wgpu-rs
//! [WebGPU API]: https://gpuweb.github.io/gpuweb/
//! [`glyphon`]: https://github.com/grovesNL/glyphon
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![allow(missing_docs)]
pub mod layer;
pub mod primitive;
pub mod settings;
pub mod window;

#[cfg(feature = "geometry")]
pub mod geometry;

mod buffer;
mod color;
mod engine;
mod quad;
mod text;
mod triangle;

#[cfg(any(feature = "image", feature = "svg"))]
#[path = "image/mod.rs"]
mod image;

#[cfg(not(any(feature = "image", feature = "svg")))]
#[path = "image/null.rs"]
mod image;

use buffer::Buffer;

pub use iced_graphics as graphics;
pub use iced_graphics::core;

pub use wgpu;

pub use engine::Engine;
pub use layer::Layer;
pub use primitive::Primitive;
pub use settings::Settings;

#[cfg(feature = "geometry")]
pub use geometry::Geometry;

use crate::core::{
    Background, Color, Font, Pixels, Point, Rectangle, Size, Transformation,
    Vector,
};
use crate::graphics::text::{Editor, Paragraph};
use crate::graphics::Viewport;

/// A [`wgpu`] graphics renderer for [`iced`].
///
/// [`wgpu`]: https://github.com/gfx-rs/wgpu-rs
/// [`iced`]: https://github.com/iced-rs/iced
#[allow(missing_debug_implementations)]
pub struct Renderer {
    default_font: Font,
    default_text_size: Pixels,
    layers: layer::Stack,

    triangle_storage: triangle::Storage,
    text_storage: text::Storage,
    text_viewport: text::Viewport,

    // TODO: Centralize all the image feature handling
    #[cfg(any(feature = "svg", feature = "image"))]
    image_cache: std::cell::RefCell<image::Cache>,
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        engine: &Engine,
        default_font: Font,
        default_text_size: Pixels,
    ) -> Self {
        Self {
            default_font,
            default_text_size,
            layers: layer::Stack::new(),

            triangle_storage: triangle::Storage::new(),
            text_storage: text::Storage::new(),
            text_viewport: engine.text_pipeline.create_viewport(device),

            #[cfg(any(feature = "svg", feature = "image"))]
            image_cache: std::cell::RefCell::new(
                engine.create_image_cache(device),
            ),
        }
    }

    pub fn present<T: AsRef<str>>(
        &mut self,
        engine: &mut Engine,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        clear_color: Option<Color>,
        format: wgpu::TextureFormat,
        frame: &wgpu::TextureView,
        viewport: &Viewport,
        overlay: &[T],
    ) {
        self.draw_overlay(overlay, viewport);
        self.prepare(engine, device, queue, format, encoder, viewport);
        self.render(engine, encoder, frame, clear_color, viewport);

        self.triangle_storage.trim();
        self.text_storage.trim();

        #[cfg(any(feature = "svg", feature = "image"))]
        self.image_cache.borrow_mut().trim();
    }

    fn prepare(
        &mut self,
        engine: &mut Engine,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _format: wgpu::TextureFormat,
        encoder: &mut wgpu::CommandEncoder,
        viewport: &Viewport,
    ) {
        let scale_factor = viewport.scale_factor() as f32;

        self.text_viewport.update(queue, viewport.physical_size());

        for layer in self.layers.iter_mut() {
            if !layer.quads.is_empty() {
                engine.quad_pipeline.prepare(
                    device,
                    encoder,
                    &mut engine.staging_belt,
                    &layer.quads,
                    viewport.projection(),
                    scale_factor,
                );
            }

            if !layer.triangles.is_empty() {
                engine.triangle_pipeline.prepare(
                    device,
                    encoder,
                    &mut engine.staging_belt,
                    &mut self.triangle_storage,
                    &layer.triangles,
                    Transformation::scale(scale_factor),
                    viewport.physical_size(),
                );
            }

            if !layer.primitives.is_empty() {
                for instance in &layer.primitives {
                    instance.primitive.prepare(
                        device,
                        queue,
                        engine.format,
                        &mut engine.primitive_storage,
                        &instance.bounds,
                        viewport,
                    );
                }
            }

            if !layer.text.is_empty() {
                engine.text_pipeline.prepare(
                    device,
                    queue,
                    &self.text_viewport,
                    encoder,
                    &mut self.text_storage,
                    &layer.text,
                    layer.bounds,
                    Transformation::scale(scale_factor),
                );
            }

            #[cfg(any(feature = "svg", feature = "image"))]
            if !layer.images.is_empty() {
                engine.image_pipeline.prepare(
                    device,
                    encoder,
                    &mut engine.staging_belt,
                    &mut self.image_cache.borrow_mut(),
                    &layer.images,
                    viewport.projection(),
                    scale_factor,
                );
            }
        }
    }

    fn render(
        &mut self,
        engine: &mut Engine,
        encoder: &mut wgpu::CommandEncoder,
        frame: &wgpu::TextureView,
        clear_color: Option<Color>,
        viewport: &Viewport,
    ) {
        use std::mem::ManuallyDrop;

        let mut render_pass = ManuallyDrop::new(encoder.begin_render_pass(
            &wgpu::RenderPassDescriptor {
                label: Some("iced_wgpu render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: frame,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: match clear_color {
                            Some(background_color) => wgpu::LoadOp::Clear({
                                let [r, g, b, a] =
                                    graphics::color::pack(background_color)
                                        .components();

                                wgpu::Color {
                                    r: f64::from(r),
                                    g: f64::from(g),
                                    b: f64::from(b),
                                    a: f64::from(a),
                                }
                            }),
                            None => wgpu::LoadOp::Load,
                        },
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            },
        ));

        let mut quad_layer = 0;
        let mut mesh_layer = 0;
        let mut text_layer = 0;

        #[cfg(any(feature = "svg", feature = "image"))]
        let mut image_layer = 0;
        #[cfg(any(feature = "svg", feature = "image"))]
        let image_cache = self.image_cache.borrow();

        let scale_factor = viewport.scale_factor() as f32;
        let physical_bounds = Rectangle::<f32>::from(Rectangle::with_size(
            viewport.physical_size(),
        ));

        let scale = Transformation::scale(scale_factor);

        for layer in self.layers.iter() {
            let Some(physical_bounds) =
                physical_bounds.intersection(&(layer.bounds * scale))
            else {
                continue;
            };

            let Some(scissor_rect) = physical_bounds.snap() else {
                continue;
            };

            if !layer.quads.is_empty() {
                engine.quad_pipeline.render(
                    quad_layer,
                    scissor_rect,
                    &layer.quads,
                    &mut render_pass,
                );

                quad_layer += 1;
            }

            if !layer.triangles.is_empty() {
                let _ = ManuallyDrop::into_inner(render_pass);

                mesh_layer += engine.triangle_pipeline.render(
                    encoder,
                    frame,
                    &self.triangle_storage,
                    mesh_layer,
                    &layer.triangles,
                    physical_bounds,
                    scale,
                );

                render_pass = ManuallyDrop::new(encoder.begin_render_pass(
                    &wgpu::RenderPassDescriptor {
                        label: Some("iced_wgpu render pass"),
                        color_attachments: &[Some(
                            wgpu::RenderPassColorAttachment {
                                view: frame,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: wgpu::StoreOp::Store,
                                },
                            },
                        )],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    },
                ));
            }

            if !layer.primitives.is_empty() {
                let _ = ManuallyDrop::into_inner(render_pass);

                for instance in &layer.primitives {
                    if let Some(clip_bounds) = (instance.bounds * scale)
                        .intersection(&physical_bounds)
                        .and_then(Rectangle::snap)
                    {
                        instance.primitive.render(
                            encoder,
                            &engine.primitive_storage,
                            frame,
                            &clip_bounds,
                        );
                    }
                }

                render_pass = ManuallyDrop::new(encoder.begin_render_pass(
                    &wgpu::RenderPassDescriptor {
                        label: Some("iced_wgpu render pass"),
                        color_attachments: &[Some(
                            wgpu::RenderPassColorAttachment {
                                view: frame,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: wgpu::StoreOp::Store,
                                },
                            },
                        )],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    },
                ));
            }

            if !layer.text.is_empty() {
                text_layer += engine.text_pipeline.render(
                    &self.text_viewport,
                    &self.text_storage,
                    text_layer,
                    &layer.text,
                    scissor_rect,
                    &mut render_pass,
                );
            }

            #[cfg(any(feature = "svg", feature = "image"))]
            if !layer.images.is_empty() {
                engine.image_pipeline.render(
                    &image_cache,
                    image_layer,
                    scissor_rect,
                    &mut render_pass,
                );

                image_layer += 1;
            }
        }

        let _ = ManuallyDrop::into_inner(render_pass);
    }

    fn draw_overlay(
        &mut self,
        overlay: &[impl AsRef<str>],
        viewport: &Viewport,
    ) {
        use crate::core::alignment;
        use crate::core::text::Renderer as _;
        use crate::core::Renderer as _;

        self.with_layer(
            Rectangle::with_size(viewport.logical_size()),
            |renderer| {
                for (i, line) in overlay.iter().enumerate() {
                    let text = crate::core::Text {
                        content: line.as_ref().to_owned(),
                        bounds: viewport.logical_size(),
                        size: Pixels(20.0),
                        line_height: core::text::LineHeight::default(),
                        font: Font::MONOSPACE,
                        horizontal_alignment: alignment::Horizontal::Left,
                        vertical_alignment: alignment::Vertical::Top,
                        shaping: core::text::Shaping::Basic,
                    };

                    renderer.fill_text(
                        text.clone(),
                        Point::new(11.0, 11.0 + 25.0 * i as f32),
                        Color::new(0.9, 0.9, 0.9, 1.0),
                        Rectangle::with_size(Size::INFINITY),
                    );

                    renderer.fill_text(
                        text,
                        Point::new(11.0, 11.0 + 25.0 * i as f32)
                            + Vector::new(-1.0, -1.0),
                        Color::BLACK,
                        Rectangle::with_size(Size::INFINITY),
                    );
                }
            },
        );
    }
}

impl core::Renderer for Renderer {
    fn start_layer(&mut self, bounds: Rectangle) {
        self.layers.push_clip(bounds);
    }

    fn end_layer(&mut self) {
        self.layers.pop_clip();
    }

    fn start_transformation(&mut self, transformation: Transformation) {
        self.layers.push_transformation(transformation);
    }

    fn end_transformation(&mut self) {
        self.layers.pop_transformation();
    }

    fn fill_quad(
        &mut self,
        quad: core::renderer::Quad,
        background: impl Into<Background>,
    ) {
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_quad(quad, background.into(), transformation);
    }

    fn clear(&mut self) {
        self.layers.clear();
    }
}

impl core::text::Renderer for Renderer {
    type Font = Font;
    type Paragraph = Paragraph;
    type Editor = Editor;

    const ICON_FONT: Font = Font::with_name("Iced-Icons");
    const CHECKMARK_ICON: char = '\u{f00c}';
    const ARROW_DOWN_ICON: char = '\u{e800}';

    fn default_font(&self) -> Self::Font {
        self.default_font
    }

    fn default_size(&self) -> Pixels {
        self.default_text_size
    }

    fn fill_paragraph(
        &mut self,
        text: &Self::Paragraph,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        let (layer, transformation) = self.layers.current_mut();

        layer.draw_paragraph(
            text,
            position,
            color,
            clip_bounds,
            transformation,
        );
    }

    fn fill_editor(
        &mut self,
        editor: &Self::Editor,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_editor(editor, position, color, clip_bounds, transformation);
    }

    fn fill_text(
        &mut self,
        text: core::Text,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_text(text, position, color, clip_bounds, transformation);
    }
}

#[cfg(feature = "image")]
impl core::image::Renderer for Renderer {
    type Handle = core::image::Handle;

    fn measure_image(&self, handle: &Self::Handle) -> Size<u32> {
        self.image_cache.borrow_mut().measure_image(handle)
    }

    fn draw_image(
        &mut self,
        handle: Self::Handle,
        filter_method: core::image::FilterMethod,
        bounds: Rectangle,
        rotation: core::Radians,
        opacity: f32,
    ) {
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_image(
            handle,
            filter_method,
            bounds,
            transformation,
            rotation,
            opacity,
        );
    }
}

#[cfg(feature = "svg")]
impl core::svg::Renderer for Renderer {
    fn measure_svg(&self, handle: &core::svg::Handle) -> Size<u32> {
        self.image_cache.borrow_mut().measure_svg(handle)
    }

    fn draw_svg(
        &mut self,
        handle: core::svg::Handle,
        color_filter: Option<Color>,
        bounds: Rectangle,
        rotation: core::Radians,
        opacity: f32,
    ) {
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_svg(
            handle,
            color_filter,
            bounds,
            transformation,
            rotation,
            opacity,
        );
    }
}

impl graphics::mesh::Renderer for Renderer {
    fn draw_mesh(&mut self, mesh: graphics::Mesh) {
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_mesh(mesh, transformation);
    }
}

#[cfg(feature = "geometry")]
impl graphics::geometry::Renderer for Renderer {
    type Geometry = Geometry;
    type Frame = geometry::Frame;

    fn new_frame(&self, size: Size) -> Self::Frame {
        geometry::Frame::new(size)
    }

    fn draw_geometry(&mut self, geometry: Self::Geometry) {
        let (layer, transformation) = self.layers.current_mut();

        match geometry {
            Geometry::Live { meshes, text } => {
                layer.draw_mesh_group(meshes, transformation);
                layer.draw_text_group(text, transformation);
            }
            Geometry::Cached(cache) => {
                if let Some(meshes) = cache.meshes {
                    layer.draw_mesh_cache(meshes, transformation);
                }

                if let Some(text) = cache.text {
                    layer.draw_text_cache(text, transformation);
                }
            }
        }
    }
}

impl primitive::Renderer for Renderer {
    fn draw_primitive(&mut self, bounds: Rectangle, primitive: impl Primitive) {
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_primitive(bounds, Box::new(primitive), transformation);
    }
}

impl graphics::compositor::Default for crate::Renderer {
    type Compositor = window::Compositor;
}
