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
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(missing_docs)]
pub mod blur;
pub mod cached_scale;
pub mod gradient_fade;
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

use iced_debug as debug;
pub use iced_graphics as graphics;
pub use iced_graphics::core;

pub use wgpu;

pub use engine::Engine;
pub use layer::Layer;
pub use primitive::Primitive;
pub use settings::Settings;

#[cfg(feature = "geometry")]
pub use geometry::Geometry;

use crate::core::renderer;
use crate::core::{Background, Color, Font, Pixels, Point, Rectangle, Size, Transformation};
use crate::graphics::mesh;
use crate::graphics::text::{Editor, Paragraph};
use crate::graphics::{Shell, Viewport};

/// A [`wgpu`] graphics renderer for [`iced`].
///
/// [`wgpu`]: https://github.com/gfx-rs/wgpu-rs
/// [`iced`]: https://github.com/iced-rs/iced
pub struct Renderer {
    engine: Engine,

    default_font: Font,
    default_text_size: Pixels,
    layers: layer::Stack,
    scale_factor: Option<f32>,
    /// Stack of opacity values (multiplied together for nested opacity)
    opacity_stack: Vec<f32>,

    quad: quad::State,
    triangle: triangle::State,
    text: text::State,
    text_viewport: text::Viewport,

    #[cfg(any(feature = "svg", feature = "image"))]
    image: image::State,

    // TODO: Centralize all the image feature handling
    #[cfg(any(feature = "svg", feature = "image"))]
    image_cache: std::cell::RefCell<image::Cache>,

    /// Gradient fade state for offscreen rendering
    gradient_fade: gradient_fade::State,

    /// Cached scale state for GPU-accelerated scale animations
    cached_scale_state: cached_scale::State,

    /// Backdrop blur state
    blur_state: blur::State,
    /// Backdrop blur texture cache
    blur_cache: blur::TextureCache,

    staging_belt: wgpu::util::StagingBelt,
}

impl Renderer {
    pub fn new(engine: Engine, default_font: Font, default_text_size: Pixels) -> Self {
        Self {
            default_font,
            default_text_size,
            layers: layer::Stack::new(),
            scale_factor: None,
            opacity_stack: vec![1.0],

            quad: quad::State::new(),
            triangle: triangle::State::new(&engine.device, &engine.triangle_pipeline),
            text: text::State::new(),
            text_viewport: engine.text_pipeline.create_viewport(&engine.device),

            #[cfg(any(feature = "svg", feature = "image"))]
            image: image::State::new(),

            #[cfg(any(feature = "svg", feature = "image"))]
            image_cache: std::cell::RefCell::new(engine.create_image_cache()),

            gradient_fade: gradient_fade::State::new(),

            cached_scale_state: cached_scale::State::new(),

            blur_state: blur::State::new(),
            blur_cache: blur::TextureCache::new(),

            // TODO: Resize belt smartly (?)
            // It would be great if the `StagingBelt` API exposed methods
            // for introspection to detect when a resize may be worth it.
            staging_belt: wgpu::util::StagingBelt::new(
                engine.device.clone(),
                buffer::MAX_WRITE_SIZE as u64,
            ),

            engine,
        }
    }

    /// Returns the current combined opacity value from the opacity stack.
    #[inline]
    fn current_opacity(&self) -> f32 {
        *self.opacity_stack.last().unwrap_or(&1.0)
    }

    #[allow(unused_variables)]
    fn draw(
        &mut self,
        clear_color: Option<Color>,
        target: &wgpu::TextureView,
        target_texture: Option<&wgpu::Texture>,
        viewport: &Viewport,
    ) -> wgpu::CommandEncoder {
        let mut encoder =
            self.engine
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("iced_wgpu encoder"),
                });

        self.prepare(&mut encoder, viewport);

        // Platform-specific blur handling:
        // - Native: render to swapchain, copy from swapchain, blur (COPY_SRC supported)
        // - WASM: render to offscreen texture, blur from that (no COPY_SRC on swapchain)
        #[cfg(target_arch = "wasm32")]
        {
            if self.blur_state.has_regions() {
                // WASM path: render background to scene_copy, blur, then render to swapchain
                self.draw_with_offscreen_blur(&mut encoder, target, clear_color, viewport);
            } else {
                // No blur needed, render directly to swapchain
                self.render(&mut encoder, target, clear_color, viewport);
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Native path: render to swapchain (gradient fades applied inline
            // at the correct z-position during render), then blur
            self.render(&mut encoder, target, clear_color, viewport);
            self.apply_backdrop_blurs(&mut encoder, target, target_texture, viewport);
        }

        // Process gradient fade regions after main render (WASM no-blur path only;
        // blur path handles gradient fades inside draw_with_offscreen_blur)
        #[cfg(target_arch = "wasm32")]
        self.apply_gradient_fades(&mut encoder, target, viewport);

        self.quad.trim();
        self.triangle.trim();
        self.text.trim();

        // TODO: Provide window id (?)
        self.engine.trim();

        #[cfg(any(feature = "svg", feature = "image"))]
        {
            self.image.trim();
            self.image_cache.borrow_mut().trim();
        }

        encoder
    }

    pub fn present(
        &mut self,
        clear_color: Option<Color>,
        _format: wgpu::TextureFormat,
        frame: &wgpu::TextureView,
        viewport: &Viewport,
    ) -> wgpu::SubmissionIndex {
        let encoder = self.draw(clear_color, frame, None, viewport);

        self.staging_belt.finish();
        let submission = self.engine.queue.submit([encoder.finish()]);
        self.staging_belt.recall();
        submission
    }

    /// Present with access to the swapchain texture for advanced effects like backdrop blur.
    ///
    /// The `frame_texture` allows copying the rendered scene for blur effects.
    pub fn present_with_texture(
        &mut self,
        clear_color: Option<Color>,
        _format: wgpu::TextureFormat,
        frame: &wgpu::TextureView,
        frame_texture: &wgpu::Texture,
        viewport: &Viewport,
    ) -> wgpu::SubmissionIndex {
        let encoder = self.draw(clear_color, frame, Some(frame_texture), viewport);

        self.staging_belt.finish();
        let submission = self.engine.queue.submit([encoder.finish()]);
        self.staging_belt.recall();
        submission
    }

    /// Renders the current surface to an offscreen buffer.
    ///
    /// Returns RGBA bytes of the texture data.
    pub fn screenshot(&mut self, viewport: &Viewport, background_color: Color) -> Vec<u8> {
        #[derive(Clone, Copy, Debug)]
        struct BufferDimensions {
            width: u32,
            height: u32,
            unpadded_bytes_per_row: usize,
            padded_bytes_per_row: usize,
        }

        impl BufferDimensions {
            fn new(size: Size<u32>) -> Self {
                let unpadded_bytes_per_row = size.width as usize * 4; //slice of buffer per row; always RGBA
                let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize; //256
                let padded_bytes_per_row_padding =
                    (alignment - unpadded_bytes_per_row % alignment) % alignment;
                let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;

                Self {
                    width: size.width,
                    height: size.height,
                    unpadded_bytes_per_row,
                    padded_bytes_per_row,
                }
            }
        }

        let dimensions = BufferDimensions::new(viewport.physical_size());

        let texture_extent = wgpu::Extent3d {
            width: dimensions.width,
            height: dimensions.height,
            depth_or_array_layers: 1,
        };

        let texture = self.engine.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("iced_wgpu.offscreen.source_texture"),
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.engine.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.draw(Some(background_color), &view, Some(&texture), viewport);

        let texture = crate::color::convert(
            &self.engine.device,
            &mut encoder,
            texture,
            if graphics::color::GAMMA_CORRECTION {
                wgpu::TextureFormat::Rgba8UnormSrgb
            } else {
                wgpu::TextureFormat::Rgba8Unorm
            },
        );

        let output_buffer = self.engine.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("iced_wgpu.offscreen.output_texture_buffer"),
            size: (dimensions.padded_bytes_per_row * dimensions.height as usize) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(dimensions.padded_bytes_per_row as u32),
                    rows_per_image: None,
                },
            },
            texture_extent,
        );

        self.staging_belt.finish();
        let index = self.engine.queue.submit([encoder.finish()]);
        self.staging_belt.recall();

        let slice = output_buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});

        let _ = self.engine.device.poll(wgpu::PollType::Wait {
            submission_index: Some(index),
            timeout: None,
        });

        let mapped_buffer = slice.get_mapped_range();

        mapped_buffer
            .chunks(dimensions.padded_bytes_per_row)
            .fold(vec![], |mut acc, row| {
                acc.extend(&row[..dimensions.unpadded_bytes_per_row]);
                acc
            })
    }

    fn prepare(&mut self, encoder: &mut wgpu::CommandEncoder, viewport: &Viewport) {
        let scale_factor = viewport.scale_factor();

        self.text_viewport
            .update(&self.engine.queue, viewport.physical_size());

        let physical_bounds =
            Rectangle::<f32>::from(Rectangle::with_size(viewport.physical_size()));

        // Only merge layers if there are no pending blur effects
        // Layer merging changes indices which would break post-blur content tracking
        let has_blur = self.blur_state.has_post_blur_content() || self.blur_state.has_regions();
        log::trace!(
            "PREPARE: has_blur={}, post_blur_content={}, regions={}",
            has_blur,
            self.blur_state.has_post_blur_content(),
            self.blur_state.has_regions()
        );

        if !has_blur {
            log::trace!("PREPARE: merging layers");
            self.layers.merge();
        } else {
            log::trace!("PREPARE: skipping merge, just flushing");
            // Just flush without merging
            self.layers.flush();
        }

        log::trace!(
            "PREPARE: active layers after prepare: {}",
            self.layers.active_count()
        );

        for layer in self.layers.iter() {
            let clip_bounds = layer.bounds * scale_factor;

            if physical_bounds
                .intersection(&clip_bounds)
                .and_then(Rectangle::snap)
                .is_none()
            {
                continue;
            }

            if !layer.quads.is_empty() {
                let prepare_span = debug::prepare(debug::Primitive::Quad);

                self.quad.prepare(
                    &self.engine.quad_pipeline,
                    &self.engine.device,
                    &mut self.staging_belt,
                    encoder,
                    &layer.quads,
                    viewport.projection(),
                    scale_factor,
                );

                prepare_span.finish();
            }

            if !layer.triangles.is_empty() {
                let prepare_span = debug::prepare(debug::Primitive::Triangle);

                self.triangle.prepare(
                    &self.engine.triangle_pipeline,
                    &self.engine.device,
                    &mut self.staging_belt,
                    encoder,
                    &layer.triangles,
                    Transformation::scale(scale_factor),
                    viewport.physical_size(),
                );

                prepare_span.finish();
            }

            if !layer.primitives.is_empty() {
                let prepare_span = debug::prepare(debug::Primitive::Shader);

                let mut primitive_storage = self
                    .engine
                    .primitive_storage
                    .write()
                    .expect("Write primitive storage");

                for instance in &layer.primitives {
                    instance.primitive.prepare(
                        &mut primitive_storage,
                        &self.engine.device,
                        &self.engine.queue,
                        self.engine.format,
                        &instance.bounds,
                        viewport,
                    );
                }

                prepare_span.finish();
            }

            #[cfg(any(feature = "svg", feature = "image"))]
            if !layer.images.is_empty() {
                let prepare_span = debug::prepare(debug::Primitive::Image);

                self.image.prepare(
                    &self.engine.image_pipeline,
                    &self.engine.device,
                    &mut self.staging_belt,
                    encoder,
                    &mut self.image_cache.borrow_mut(),
                    &layer.images,
                    viewport.projection(),
                    scale_factor,
                );

                prepare_span.finish();
            }

            if !layer.text.is_empty() {
                let prepare_span = debug::prepare(debug::Primitive::Text);

                self.text.prepare(
                    &self.engine.text_pipeline,
                    &self.engine.device,
                    &self.engine.queue,
                    &self.text_viewport,
                    encoder,
                    &layer.text,
                    layer.bounds,
                    Transformation::scale(scale_factor),
                );

                prepare_span.finish();
            }
        }
    }

    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame: &wgpu::TextureView,
        clear_color: Option<Color>,
        viewport: &Viewport,
    ) {
        use std::mem::ManuallyDrop;

        let mut render_pass =
            ManuallyDrop::new(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("iced_wgpu render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: frame,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: match clear_color {
                            Some(background_color) => wgpu::LoadOp::Clear({
                                let [r, g, b, a] =
                                    graphics::color::pack(background_color).components();

                                wgpu::Color {
                                    r: f64::from(r * a),
                                    g: f64::from(g * a),
                                    b: f64::from(b * a),
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
                multiview_mask: None,
            }));

        let mut quad_layer = 0;
        let mut mesh_layer = 0;
        let mut text_layer = 0;

        #[cfg(any(feature = "svg", feature = "image"))]
        let mut image_layer = 0;

        let scale_factor = viewport.scale_factor();
        let physical_bounds =
            Rectangle::<f32>::from(Rectangle::with_size(viewport.physical_size()));

        let scale = Transformation::scale(scale_factor);

        // Take gradient fade regions so we can apply them inline at the correct
        // z-position, rather than after all layers (which breaks overlay z-order).
        let fade_regions = self.gradient_fade.take_regions();
        let mut fade_applied = vec![false; fade_regions.len()];

        // Take cached scale regions for GPU-accelerated scale animations
        let scale_regions = self.cached_scale_state.take_regions();
        let mut scale_applied = vec![false; scale_regions.len()];

        let layer_count = self.layers.as_slice().len();

        // Use index-based loop so we can call &mut self methods for gradient
        // fade compositing between layers without borrow conflicts.
        for layer_index in 0..layer_count {
            // Apply any gradient fade regions whose end_layer we've reached.
            // This ensures fade content is composited before layers that come
            // after it in the z-order (e.g. modal overlays).
            {
                let mut needs_offscreen = false;
                for (i, region) in fade_regions.iter().enumerate() {
                    if !fade_applied[i] && layer_index >= region.end_layer {
                        needs_offscreen = true;
                        break;
                    }
                }
                for (i, region) in scale_regions.iter().enumerate() {
                    if !scale_applied[i] && layer_index >= region.end_layer {
                        needs_offscreen = true;
                        break;
                    }
                }
                if needs_offscreen {
                    let _ = ManuallyDrop::into_inner(render_pass);
                    for (i, region) in fade_regions.iter().enumerate() {
                        if !fade_applied[i] && layer_index >= region.end_layer {
                            self.apply_gradient_fade_region(encoder, frame, region, viewport);
                            fade_applied[i] = true;
                        }
                    }
                    for (i, region) in scale_regions.iter().enumerate() {
                        if !scale_applied[i] && layer_index >= region.end_layer {
                            self.apply_cached_scale_region(encoder, frame, region, viewport);
                            scale_applied[i] = true;
                        }
                    }
                    render_pass =
                        ManuallyDrop::new(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("iced_wgpu render pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: frame,
                                depth_slice: None,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        }));
                }
            }

            let layer = &self.layers.as_slice()[layer_index];

            // Check if this layer is in a gradient fade region - if so, skip
            // it here. It will be rendered to an offscreen texture and
            // composited with the gradient shader by apply_gradient_fade_region.
            let in_fade = fade_regions
                .iter()
                .any(|r| layer_index >= r.start_layer && layer_index < r.end_layer);

            // Check if this layer is in a cached scale region
            let in_cached_scale = scale_regions
                .iter()
                .any(|r| layer_index >= r.start_layer && layer_index < r.end_layer);

            if in_fade || in_cached_scale {
                // Still need to count primitives so offsets are correct,
                // but only if prepare() actually processed this layer
                // (prepare skips layers with no physical bounds intersection)
                if physical_bounds
                    .intersection(&(layer.bounds * scale_factor))
                    .and_then(Rectangle::snap)
                    .is_some()
                {
                    if !layer.quads.is_empty() {
                        quad_layer += 1;
                    }
                    text_layer += layer
                        .text
                        .iter()
                        .filter(|item| matches!(item, text::Item::Group { .. }))
                        .count();
                    #[cfg(any(feature = "svg", feature = "image"))]
                    if !layer.images.is_empty() {
                        image_layer += 1;
                    }
                }
                continue;
            }

            // Check if this layer is post-blur content - if so, skip it here
            // It will be rendered after blur is applied
            if self.blur_state.is_layer_in_post_blur(layer_index) {
                log::trace!(
                    "render: SKIPPING layer {} (post-blur content), bounds=({:.1},{:.1},{:.1},{:.1})",
                    layer_index,
                    layer.bounds.x,
                    layer.bounds.y,
                    layer.bounds.width,
                    layer.bounds.height
                );
                // Still need to count primitives so offsets are correct,
                // but only if prepare() actually processed this layer
                if physical_bounds
                    .intersection(&(layer.bounds * scale_factor))
                    .and_then(Rectangle::snap)
                    .is_some()
                {
                    if !layer.quads.is_empty() {
                        quad_layer += 1;
                    }
                    text_layer += layer
                        .text
                        .iter()
                        .filter(|item| matches!(item, text::Item::Group { .. }))
                        .count();
                    #[cfg(any(feature = "svg", feature = "image"))]
                    if !layer.images.is_empty() {
                        image_layer += 1;
                    }
                }
                continue;
            }

            let Some(physical_bounds) =
                physical_bounds.intersection(&(layer.bounds * scale_factor))
            else {
                continue;
            };

            let Some(scissor_rect) = physical_bounds.snap() else {
                continue;
            };

            if !layer.quads.is_empty() {
                let render_span = debug::render(debug::Primitive::Quad);
                self.quad.render(
                    &self.engine.quad_pipeline,
                    quad_layer,
                    scissor_rect,
                    &layer.quads,
                    &mut render_pass,
                );
                render_span.finish();

                quad_layer += 1;
            }

            if !layer.triangles.is_empty() {
                let _ = ManuallyDrop::into_inner(render_pass);

                let render_span = debug::render(debug::Primitive::Triangle);
                mesh_layer += self.triangle.render(
                    &self.engine.triangle_pipeline,
                    encoder,
                    frame,
                    mesh_layer,
                    &layer.triangles,
                    physical_bounds,
                    scale,
                );
                render_span.finish();

                render_pass =
                    ManuallyDrop::new(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("iced_wgpu render pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: frame,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    }));
            }

            if !layer.primitives.is_empty() {
                let render_span = debug::render(debug::Primitive::Shader);

                let primitive_storage = self
                    .engine
                    .primitive_storage
                    .read()
                    .expect("Read primitive storage");

                let mut need_render = Vec::new();

                for instance in &layer.primitives {
                    let bounds = instance.bounds * scale;

                    if let Some(clip_bounds) = (instance.bounds * scale)
                        .intersection(&physical_bounds)
                        .and_then(Rectangle::snap)
                    {
                        render_pass.set_viewport(
                            bounds.x,
                            bounds.y,
                            bounds.width,
                            bounds.height,
                            0.0,
                            1.0,
                        );

                        render_pass.set_scissor_rect(
                            clip_bounds.x,
                            clip_bounds.y,
                            clip_bounds.width,
                            clip_bounds.height,
                        );

                        let drawn = instance
                            .primitive
                            .draw(&primitive_storage, &mut render_pass);

                        if !drawn {
                            need_render.push((instance, clip_bounds));
                        }
                    }
                }

                render_pass.set_viewport(
                    0.0,
                    0.0,
                    viewport.physical_width() as f32,
                    viewport.physical_height() as f32,
                    0.0,
                    1.0,
                );

                render_pass.set_scissor_rect(
                    0,
                    0,
                    viewport.physical_width(),
                    viewport.physical_height(),
                );

                if !need_render.is_empty() {
                    let _ = ManuallyDrop::into_inner(render_pass);

                    for (instance, clip_bounds) in need_render {
                        instance
                            .primitive
                            .render(&primitive_storage, encoder, frame, &clip_bounds);
                    }

                    render_pass =
                        ManuallyDrop::new(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("iced_wgpu render pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: frame,
                                depth_slice: None,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        }));
                }

                render_span.finish();
            }

            #[cfg(any(feature = "svg", feature = "image"))]
            if !layer.images.is_empty() {
                let render_span = debug::render(debug::Primitive::Image);
                self.image.render(
                    &self.engine.image_pipeline,
                    image_layer,
                    scissor_rect,
                    &mut render_pass,
                );
                render_span.finish();

                image_layer += 1;
            }

            if !layer.text.is_empty() {
                let render_span = debug::render(debug::Primitive::Text);
                text_layer += self.text.render(
                    &self.engine.text_pipeline,
                    &self.text_viewport,
                    text_layer,
                    &layer.text,
                    scissor_rect,
                    &mut render_pass,
                );
                render_span.finish();
            }
        }

        let _ = ManuallyDrop::into_inner(render_pass);

        // Apply any remaining gradient fade regions whose layers were at
        // the end of the stack (no later layers triggered inline application).
        for (i, region) in fade_regions.iter().enumerate() {
            if !fade_applied[i] {
                self.apply_gradient_fade_region(encoder, frame, region, viewport);
            }
        }

        // Apply any remaining cached scale regions.
        for (i, region) in scale_regions.iter().enumerate() {
            if !scale_applied[i] {
                self.apply_cached_scale_region(encoder, frame, region, viewport);
            }
        }

        debug::layers_rendered(|| {
            self.layers
                .iter()
                .filter(|layer| {
                    !layer.is_empty()
                        && physical_bounds
                            .intersection(&(layer.bounds * scale_factor))
                            .is_some_and(|viewport| viewport.snap().is_some())
                })
                .count()
        });
    }

    /// WASM-specific: Draw with offscreen blur support.
    ///
    /// On WASM/WebGL, we can't copy from the swapchain (no COPY_SRC support).
    /// Instead, we render background content to an offscreen texture first,
    /// apply blur from that, then render to the swapchain.
    #[cfg(target_arch = "wasm32")]
    fn draw_with_offscreen_blur(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clear_color: Option<Color>,
        viewport: &Viewport,
    ) {
        let physical_size = viewport.physical_size();

        // Ensure textures exist
        let _ = self.blur_cache.get_blur_textures(
            &self.engine.device,
            physical_size,
            self.engine.format,
        );

        // Get a view from the scene_copy texture for rendering to
        let scene_copy_view = self
            .blur_cache
            .get_scene_copy_texture()
            .unwrap()
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Render background (non-post-blur layers) to offscreen texture
        self.render(encoder, &scene_copy_view, clear_color, viewport);

        // Take blur regions and post-blur content
        let regions = self.blur_state.take_regions();
        let post_blur_content = self.blur_state.take_post_blur_content();

        // First, blit background to swapchain (need fresh view reference)
        let scene_view = self
            .blur_cache
            .get_scene_copy_texture()
            .unwrap()
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.engine.blur_pipeline.blit_full(
            &self.engine.device,
            encoder,
            &scene_view,
            target,
            physical_size,
        );

        // Apply gradient fades to swapchain before blur so faded content
        // is included in the blurred regions
        self.apply_gradient_fades(encoder, target, viewport);

        // Apply blur for each region
        for region in &regions {
            // Ensure textures exist (get_blur_textures creates them if needed)
            let _ = self.blur_cache.get_blur_textures(
                &self.engine.device,
                physical_size,
                self.engine.format,
            );

            // Create owned views to avoid lifetime issues
            let scene_view_owned = self
                .blur_cache
                .get_scene_copy_texture()
                .unwrap()
                .create_view(&wgpu::TextureViewDescriptor::default());
            let intermediate_texture = self.blur_cache.get_intermediate(
                &self.engine.device,
                physical_size,
                self.engine.format,
            );

            self.engine.blur_pipeline.render(
                &self.engine.device,
                encoder,
                &scene_view_owned,
                intermediate_texture,
                target,
                &region.blur,
                viewport,
            );
        }

        // Render post-blur content on top
        if !post_blur_content.is_empty() {
            self.render_post_blur_layers(encoder, target, viewport, &post_blur_content);
        }
    }

    /// Apply backdrop blur effects to specified regions (native path).
    ///
    /// This copies the rendered content within blur bounds, applies a two-pass
    /// Gaussian blur, and draws the blurred result back.
    ///
    /// If `target_texture` is provided, uses GPU copy to capture the scene
    /// before applying blur. Otherwise, blur won't work (needs scene content).
    ///
    /// Note: On WASM, this is not called - see `draw_with_offscreen_blur` instead.
    #[cfg(not(target_arch = "wasm32"))]
    fn apply_backdrop_blurs(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        target_texture: Option<&wgpu::Texture>,
        viewport: &Viewport,
    ) {
        let regions = self.blur_state.take_regions();

        if regions.is_empty() {
            return;
        }

        log::debug!(
            "apply_backdrop_blurs: processing {} blur regions",
            regions.len()
        );

        let physical_size = viewport.physical_size();

        // Get post-blur content regions - these layers were skipped in main render
        let post_blur_content = self.blur_state.take_post_blur_content();

        log::trace!(
            "apply_backdrop_blurs: {} blur regions, {} post-blur content regions",
            regions.len(),
            post_blur_content.len()
        );

        // First, ensure textures exist and do the copy before getting views
        // This avoids borrow checker issues
        {
            // Ensure textures exist by getting (and discarding) the views
            let _ = self.blur_cache.get_blur_textures(
                &self.engine.device,
                physical_size,
                self.engine.format,
            );
        }

        // If we have the target texture, copy the scene to scene_copy
        // Since we skipped post-blur content in the main render, this contains only the background
        if let Some(texture) = target_texture {
            log::trace!(
                "apply_backdrop_blurs: copying scene texture {}x{} (background + gradient_fades)",
                physical_size.width,
                physical_size.height
            );
            if let Some(scene_copy_texture) = self.blur_cache.get_scene_copy_texture() {
                // GPU copy from swapchain to our scene_copy texture
                encoder.copy_texture_to_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyTextureInfo {
                        texture: scene_copy_texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::Extent3d {
                        width: physical_size.width,
                        height: physical_size.height,
                        depth_or_array_layers: 1,
                    },
                );
            } else {
                log::warn!("apply_backdrop_blurs: scene_copy_texture is None!");
            }
        } else {
            log::warn!("apply_backdrop_blurs: target_texture is None - blur won't work!");
        }

        // Now get the views again for rendering (textures already exist)
        let (scene_copy_view, intermediate_view) = self.blur_cache.get_blur_textures(
            &self.engine.device,
            physical_size,
            self.engine.format,
        );

        // Process each blur region - this blurs the background and writes to target
        for (i, region) in regions.iter().enumerate() {
            log::trace!(
                "apply_backdrop_blurs: rendering blur region {} - bounds=({:.1},{:.1},{:.1},{:.1}), radius={:.1}",
                i,
                region.blur.bounds.x,
                region.blur.bounds.y,
                region.blur.bounds.width,
                region.blur.bounds.height,
                region.blur.radius
            );
            self.engine.blur_pipeline.render(
                &self.engine.device,
                encoder,
                scene_copy_view,   // Source (background only, no children)
                intermediate_view, // Intermediate for two-pass
                target,            // Final target
                &region.blur,
                viewport,
            );
        }

        // Now render the post-blur content (children) on top of the blurred background
        // This is done by re-rendering the layers that were skipped in the main pass
        if !post_blur_content.is_empty() {
            self.render_post_blur_layers(encoder, target, viewport, &post_blur_content);
        }
    }

    /// Render layers that were skipped because they're post-blur content.
    fn render_post_blur_layers(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame: &wgpu::TextureView,
        viewport: &Viewport,
        post_blur_content: &[blur::PostBlurContent],
    ) {
        log::trace!(
            "render_post_blur_layers: rendering {} regions, total layers: {}",
            post_blur_content.len(),
            self.layers.as_slice().len()
        );

        for content in post_blur_content {
            log::trace!(
                "  post-blur range: start={}, end={:?}",
                content.start_layer,
                content.end_layer
            );
        }

        let scale_factor = viewport.scale_factor();
        let physical_bounds =
            Rectangle::<f32>::from(Rectangle::with_size(viewport.physical_size()));
        let scale = Transformation::scale(scale_factor);

        // Track layer offsets - we need to match them with the main render
        let mut quad_layer = 0;
        let mut mesh_layer = 0;
        let mut text_layer = 0;

        #[cfg(any(feature = "svg", feature = "image"))]
        let mut image_layer = 0;

        // Create a render pass for post-blur content
        let mut render_pass =
            std::mem::ManuallyDrop::new(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("iced_wgpu post-blur render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: frame,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            }));

        for (layer_index, layer) in self.layers.as_slice().iter().enumerate() {
            // Check if this layer is within any post-blur region
            let is_post_blur = post_blur_content.iter().any(|content| {
                let end = content.end_layer.unwrap_or(usize::MAX);
                layer_index >= content.start_layer && layer_index < end
            });

            if !is_post_blur {
                // Still count the layers for offset tracking,
                // but only if prepare() actually processed this layer
                if physical_bounds
                    .intersection(&(layer.bounds * scale_factor))
                    .and_then(Rectangle::snap)
                    .is_some()
                {
                    if !layer.quads.is_empty() {
                        quad_layer += 1;
                    }
                    text_layer += layer
                        .text
                        .iter()
                        .filter(|item| matches!(item, text::Item::Group { .. }))
                        .count();
                    #[cfg(any(feature = "svg", feature = "image"))]
                    if !layer.images.is_empty() {
                        image_layer += 1;
                    }
                }
                continue;
            }

            log::trace!(
                "Rendering post-blur layer {}: has_quads={}, has_triangles={}, has_text={}",
                layer_index,
                !layer.quads.is_empty(),
                !layer.triangles.is_empty(),
                !layer.text.is_empty()
            );

            let Some(physical_bounds) =
                physical_bounds.intersection(&(layer.bounds * scale_factor))
            else {
                log::warn!(
                    "Post-blur layer {} has no physical bounds intersection",
                    layer_index
                );
                continue;
            };

            let Some(scissor_rect) = physical_bounds.snap() else {
                log::warn!("Post-blur layer {} scissor rect snap failed", layer_index);
                continue;
            };

            log::trace!(
                "Post-blur layer {} scissor_rect: {:?}",
                layer_index,
                scissor_rect
            );

            if !layer.quads.is_empty() {
                log::trace!("Rendering quads at quad_layer {}", quad_layer);
                self.quad.render(
                    &self.engine.quad_pipeline,
                    quad_layer,
                    scissor_rect,
                    &layer.quads,
                    &mut render_pass,
                );
                quad_layer += 1;
            }

            if !layer.triangles.is_empty() {
                let _ = std::mem::ManuallyDrop::into_inner(render_pass);

                mesh_layer += self.triangle.render(
                    &self.engine.triangle_pipeline,
                    encoder,
                    frame,
                    mesh_layer,
                    &layer.triangles,
                    physical_bounds,
                    scale,
                );

                render_pass = std::mem::ManuallyDrop::new(encoder.begin_render_pass(
                    &wgpu::RenderPassDescriptor {
                        label: Some("iced_wgpu post-blur render pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: frame,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    },
                ));
            }

            if !layer.text.is_empty() {
                log::trace!("Rendering text at text_layer {}", text_layer);
                text_layer += self.text.render(
                    &self.engine.text_pipeline,
                    &self.text_viewport,
                    text_layer,
                    &layer.text,
                    scissor_rect,
                    &mut render_pass,
                );
            }

            #[cfg(any(feature = "svg", feature = "image"))]
            if !layer.images.is_empty() {
                self.image.render(
                    &self.engine.image_pipeline,
                    image_layer,
                    scissor_rect,
                    &mut render_pass,
                );
                image_layer += 1;
            }
        }

        // Drop the render pass
        let _ = std::mem::ManuallyDrop::into_inner(render_pass);
    }

    /// Apply gradient fade effects by re-rendering regions with the gradient shader.
    ///
    /// This function processes pending gradient fade regions and composites them
    /// with the gradient alpha mask.
    ///
    /// NOTE: In the native path, gradient fades are now applied inline during
    /// `render()` at the correct z-position. This method is kept for the WASM
    /// path.
    #[cfg(target_arch = "wasm32")]
    fn apply_gradient_fades(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        viewport: &Viewport,
    ) {
        let regions = self.gradient_fade.take_regions();

        for region in &regions {
            self.apply_gradient_fade_region(encoder, target, region, viewport);
        }
    }

    /// Apply a single gradient fade region by rendering its layers to an
    /// offscreen texture with the gradient alpha shader, then compositing
    /// the result back to the main target.
    fn apply_gradient_fade_region(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        region: &gradient_fade::GradientFadeRegion,
        viewport: &Viewport,
    ) {
        let physical_size = viewport.physical_size();
        let scale_factor = viewport.scale_factor();
        let physical_bounds =
            Rectangle::<f32>::from(Rectangle::with_size(viewport.physical_size()));

        // Ensure the offscreen texture exists
        {
            let _ = self.gradient_fade.get_or_create_texture(
                &self.engine.device,
                self.engine.format,
                physical_size,
            );
        }

        // Clear the offscreen texture
        {
            let offscreen_view = self.gradient_fade.texture_view().unwrap();
            let _clear_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("iced_wgpu.gradient_fade.clear_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: offscreen_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }

        // Render layers to offscreen texture
        {
            use std::mem::ManuallyDrop;

            let offscreen_view = self.gradient_fade.texture_view().unwrap();
            let mut render_pass =
                ManuallyDrop::new(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("iced_wgpu.gradient_fade.layer_render_pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: offscreen_view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                }));

            // Set viewport and scissor for the offscreen render pass
            render_pass.set_viewport(
                0.0,
                0.0,
                physical_size.width as f32,
                physical_size.height as f32,
                0.0,
                1.0,
            );
            render_pass.set_scissor_rect(0, 0, physical_size.width, physical_size.height);

            let layers_slice = self.layers.as_slice();
            let end_idx = region.end_layer.min(layers_slice.len());
            let start_idx = region.start_layer.min(end_idx);

            // Count primitives in layers before our range to calculate offsets
            // For text, we need to count Item::Group entries, not just non-empty batches
            let mut quad_offset = 0usize;
            let mut text_offset = 0usize;
            #[cfg(any(feature = "svg", feature = "image"))]
            let mut image_offset = 0usize;

            for layer in &layers_slice[..start_idx] {
                // Only count if prepare() actually processed this layer
                if physical_bounds
                    .intersection(&(layer.bounds * scale_factor))
                    .and_then(Rectangle::snap)
                    .is_none()
                {
                    continue;
                }
                if !layer.quads.is_empty() {
                    quad_offset += 1;
                }
                text_offset += layer
                    .text
                    .iter()
                    .filter(|item| matches!(item, text::Item::Group { .. }))
                    .count();
                #[cfg(any(feature = "svg", feature = "image"))]
                if !layer.images.is_empty() {
                    image_offset += 1;
                }
            }

            let mut quad_layer = quad_offset;
            let mut text_layer = text_offset;
            #[cfg(any(feature = "svg", feature = "image"))]
            let mut image_layer = image_offset;

            for layer in &layers_slice[start_idx..end_idx] {
                let Some(layer_physical_bounds) =
                    physical_bounds.intersection(&(layer.bounds * scale_factor))
                else {
                    continue;
                };

                let Some(scissor_rect) = layer_physical_bounds.snap() else {
                    continue;
                };

                if !layer.quads.is_empty() {
                    self.quad.render(
                        &self.engine.quad_pipeline,
                        quad_layer,
                        scissor_rect,
                        &layer.quads,
                        &mut render_pass,
                    );
                    quad_layer += 1;
                }

                #[cfg(any(feature = "svg", feature = "image"))]
                if !layer.images.is_empty() {
                    self.image.render(
                        &self.engine.image_pipeline,
                        image_layer,
                        scissor_rect,
                        &mut render_pass,
                    );
                    image_layer += 1;
                }

                if !layer.text.is_empty() {
                    text_layer += self.text.render(
                        &self.engine.text_pipeline,
                        &self.text_viewport,
                        text_layer,
                        &layer.text,
                        scissor_rect,
                        &mut render_pass,
                    );
                }
            }

            let _ = ManuallyDrop::into_inner(render_pass);
        }

        // Composite the offscreen texture back to the main target with gradient fade
        {
            let offscreen_view = self.gradient_fade.texture_view().unwrap();
            self.engine.gradient_fade_pipeline.render(
                &self.engine.device,
                encoder,
                offscreen_view,
                target,
                &region.fade,
                viewport,
            );
        }
    }

    /// Apply a single cached scale region by rendering its layers to an
    /// offscreen texture at 1x, then compositing as a scaled quad.
    fn apply_cached_scale_region(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        region: &cached_scale::CachedScaleRegion,
        viewport: &Viewport,
    ) {
        let physical_size = viewport.physical_size();
        let scale_factor = viewport.scale_factor();
        let physical_bounds =
            Rectangle::<f32>::from(Rectangle::with_size(viewport.physical_size()));

        let total_layers = self.layers.as_slice().len();
        let start = region.start_layer.min(total_layers);
        let end = region.end_layer.min(total_layers);
        if start >= end {
            return;
        }

        // Ensure the offscreen texture exists
        {
            let _ = self.cached_scale_state.get_or_create_texture(
                &self.engine.device,
                self.engine.format,
                physical_size,
            );
        }

        // Clear the offscreen texture
        {
            let offscreen_view = self.cached_scale_state.texture_view().unwrap();
            let _clear_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("iced_wgpu.cached_scale.clear_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: offscreen_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }

        // Render layers to offscreen texture.
        // Content is already at render_scale (transformation applied in start_cached_scale).
        {
            use std::mem::ManuallyDrop;

            let offscreen_view = self.cached_scale_state.texture_view().unwrap();
            let mut render_pass =
                ManuallyDrop::new(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("iced_wgpu.cached_scale.layer_render_pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: offscreen_view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                }));

            render_pass.set_viewport(
                0.0,
                0.0,
                physical_size.width as f32,
                physical_size.height as f32,
                0.0,
                1.0,
            );
            render_pass.set_scissor_rect(0, 0, physical_size.width, physical_size.height);

            let layers_slice = self.layers.as_slice();
            let end_idx = region.end_layer.min(layers_slice.len());
            let start_idx = region.start_layer.min(end_idx);

            // Count primitives in layers before our range to calculate offsets
            let mut quad_offset = 0usize;
            let mut text_offset = 0usize;
            #[cfg(any(feature = "svg", feature = "image"))]
            let mut image_offset = 0usize;

            for layer in &layers_slice[..start_idx] {
                if physical_bounds
                    .intersection(&(layer.bounds * scale_factor))
                    .and_then(Rectangle::snap)
                    .is_none()
                {
                    continue;
                }
                if !layer.quads.is_empty() {
                    quad_offset += 1;
                }
                text_offset += layer
                    .text
                    .iter()
                    .filter(|item| matches!(item, text::Item::Group { .. }))
                    .count();
                #[cfg(any(feature = "svg", feature = "image"))]
                if !layer.images.is_empty() {
                    image_offset += 1;
                }
            }

            let mut quad_layer = quad_offset;
            let mut text_layer = text_offset;
            #[cfg(any(feature = "svg", feature = "image"))]
            let mut image_layer = image_offset;

            for layer in &layers_slice[start_idx..end_idx] {
                let Some(layer_physical_bounds) =
                    physical_bounds.intersection(&(layer.bounds * scale_factor))
                else {
                    continue;
                };

                let Some(scissor_rect) = layer_physical_bounds.snap() else {
                    continue;
                };

                if !layer.quads.is_empty() {
                    self.quad.render(
                        &self.engine.quad_pipeline,
                        quad_layer,
                        scissor_rect,
                        &layer.quads,
                        &mut render_pass,
                    );
                    quad_layer += 1;
                }

                #[cfg(any(feature = "svg", feature = "image"))]
                if !layer.images.is_empty() {
                    self.image.render(
                        &self.engine.image_pipeline,
                        image_layer,
                        scissor_rect,
                        &mut render_pass,
                    );
                    image_layer += 1;
                }

                if !layer.text.is_empty() {
                    text_layer += self.text.render(
                        &self.engine.text_pipeline,
                        &self.text_viewport,
                        text_layer,
                        &layer.text,
                        scissor_rect,
                        &mut render_pass,
                    );
                }
            }

            let _ = ManuallyDrop::into_inner(render_pass);
        }

        // Composite the offscreen texture back to the main target as a scaled quad
        {
            let offscreen_view = self.cached_scale_state.texture_view().unwrap();
            self.engine.cached_scale_pipeline.render(
                &self.engine.device,
                encoder,
                offscreen_view,
                target,
                region,
                viewport,
            );
        }
    }
}

/// Applies opacity to a background, quad border, and shadow, returning the modified values.
#[inline]
fn apply_opacity(
    opacity: f32,
    background: impl Into<Background>,
    quad: core::renderer::Quad,
) -> (Background, core::renderer::Quad) {
    if opacity >= 1.0 {
        return (background.into(), quad);
    }

    let background = match background.into() {
        Background::Color(mut color) => {
            color.a *= opacity;
            Background::Color(color)
        }
        Background::Gradient(mut gradient) => {
            match &mut gradient {
                core::Gradient::Linear(linear) => {
                    for color_stop in linear.stops.iter_mut().flatten() {
                        color_stop.color.a *= opacity;
                    }
                }
                core::Gradient::Radial(radial) => {
                    for color_stop in radial.stops.iter_mut().flatten() {
                        color_stop.color.a *= opacity;
                    }
                }
                core::Gradient::Conic(conic) => {
                    for color_stop in conic.stops.iter_mut().flatten() {
                        color_stop.color.a *= opacity;
                    }
                }
            }
            Background::Gradient(gradient)
        }
    };

    let mut border = quad.border;
    border.color.a *= opacity;

    let mut shadow = quad.shadow;
    shadow.color.a *= opacity;

    let quad = core::renderer::Quad {
        border,
        shadow,
        ..quad
    };

    (background, quad)
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

    fn fill_quad(&mut self, quad: core::renderer::Quad, background: impl Into<Background>) {
        let (background, quad) = apply_opacity(self.current_opacity(), background, quad);
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_quad(quad, background, transformation);
    }

    fn allocate_image(
        &mut self,
        _handle: &core::image::Handle,
        _callback: impl FnOnce(Result<core::image::Allocation, core::image::Error>) + Send + 'static,
    ) {
        #[cfg(feature = "image")]
        self.image_cache
            .get_mut()
            .allocate_image(_handle, _callback);
    }

    fn hint(&mut self, scale_factor: f32) {
        self.scale_factor = Some(scale_factor);
    }

    fn scale_factor(&self) -> Option<f32> {
        Some(self.scale_factor? * self.layers.transformation().scale_factor())
    }

    fn tick(&mut self) {
        #[cfg(feature = "image")]
        self.image_cache.get_mut().receive();
    }

    fn reset(&mut self, new_bounds: Rectangle) {
        self.layers.reset(new_bounds);
        self.opacity_stack.clear();
        self.opacity_stack.push(1.0);
        self.gradient_fade.clear();
        self.cached_scale_state.clear();
        self.blur_state.clear();
    }

    fn start_opacity(&mut self, _bounds: Rectangle, opacity: f32) {
        let current = *self.opacity_stack.last().unwrap_or(&1.0);
        let new_opacity = current * opacity.clamp(0.0, 1.0);
        self.opacity_stack.push(new_opacity);
    }

    fn end_opacity(&mut self) {
        if self.opacity_stack.len() > 1 {
            let _ = self.opacity_stack.pop();
        }
    }

    fn start_gradient_fade(
        &mut self,
        bounds: Rectangle,
        direction: u8,
        fade_start: f32,
        fade_end: f32,
        overflow_margin: f32,
    ) {
        // Flush current layer to ensure all prior content is committed
        self.layers.flush();
        let layer_count = self.layers.active_count();

        let fade_direction = match direction {
            0 => gradient_fade::FadeDirection::TopToBottom,
            1 => gradient_fade::FadeDirection::BottomToTop,
            2 => gradient_fade::FadeDirection::LeftToRight,
            3 => gradient_fade::FadeDirection::RightToLeft,
            4 => gradient_fade::FadeDirection::VerticalBoth,
            5 => gradient_fade::FadeDirection::HorizontalBoth,
            _ => gradient_fade::FadeDirection::TopToBottom,
        };

        let fade = gradient_fade::GradientFade::new(bounds)
            .direction(fade_direction)
            .fade_start(fade_start)
            .fade_end(fade_end)
            .overflow_margin(overflow_margin);

        self.gradient_fade.start(fade, layer_count);
    }

    fn end_gradient_fade(&mut self) {
        // Flush current layer to ensure all gradient fade content is committed
        self.layers.flush();
        let layer_count = self.layers.active_count();

        let _ = self.gradient_fade.end(layer_count);
    }

    fn start_cached_scale(&mut self, bounds: Rectangle, render_scale: f32, display_scale: f32) {
        // Get the current transformation (e.g. scroll offset) BEFORE flushing
        // so we can transform bounds from content-space to screen-space.
        // The region must store screen-space bounds for correct offscreen
        // rendering and compositing (content-space positions may be outside
        // the physical viewport in scrollable containers).
        let transformation = self.layers.transformation();

        self.layers.flush();
        let layer_count = self.layers.active_count();
        // Expand clip bounds to accommodate shadows and the scale growth.
        // Use render_scale for padding since content is rasterized at that size.
        let padding =
            32.0 + bounds.width.max(bounds.height) * (render_scale.max(display_scale) - 1.0);
        let expanded = Rectangle {
            x: bounds.x - padding,
            y: bounds.y - padding,
            width: bounds.width + padding * 2.0,
            height: bounds.height + padding * 2.0,
        };

        // Transform to screen space for the cached scale region
        let screen_bounds = bounds * transformation;
        let screen_expanded = expanded * transformation;

        self.cached_scale_state.start(
            screen_bounds,
            screen_expanded,
            render_scale,
            display_scale,
            layer_count,
        );
        // Push a dedicated clip layer so content gets its own layer slot
        // (otherwise it draws into the current layer and active_count won't change)
        // Note: start_layer calls push_clip which applies the transformation,
        // so we pass the original content-space expanded bounds here.
        self.start_layer(expanded);

        // Apply render_scale transformation inside the layer so that text/SVGs
        // are rasterized at the higher resolution. Scale around bounds center.
        if (render_scale - 1.0).abs() > 0.0001 {
            let cx = bounds.x + bounds.width / 2.0;
            let cy = bounds.y + bounds.height / 2.0;
            let transform = Transformation::translate(cx, cy)
                * Transformation::scale(render_scale)
                * Transformation::translate(-cx, -cy);
            self.start_transformation(transform);
        }
    }

    fn end_cached_scale(&mut self) {
        // Pop the render_scale transformation if one was pushed.
        // Check if the active region has a render_scale != 1.0.
        if self
            .cached_scale_state
            .active_render_scale()
            .is_some_and(|rs| (rs - 1.0).abs() > 0.0001)
        {
            self.end_transformation();
        }
        // Pop the clip layer we pushed in start_cached_scale
        self.end_layer();
        self.layers.flush();
        let layer_count = self.layers.active_count();
        let _ = self.cached_scale_state.end(layer_count);
    }

    fn draw_backdrop_blur(
        &mut self,
        bounds: Rectangle,
        radius: f32,
        border_radius: [f32; 4],
        fade_start: f32,
    ) {
        log::trace!(
            "draw_backdrop_blur: bounds=({:.1},{:.1},{:.1},{:.1}), radius={:.1}, layer_count={}",
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height,
            radius,
            self.layers.active_count()
        );

        // Flush current layer to ensure all prior content is committed
        self.layers.flush();
        let layer_count = self.layers.active_count();

        // Record this blur region with the current layer index
        let blur =
            blur::BackdropBlur::with_border_radius(bounds, radius, border_radius, fade_start);
        self.blur_state.add_region(blur, layer_count);

        log::trace!(
            "draw_backdrop_blur: region added at layer_index={}, has_regions={}",
            layer_count,
            self.blur_state.has_regions()
        );
    }

    fn start_post_blur_layer(&mut self, bounds: Rectangle) {
        log::trace!(
            "start_post_blur_layer: bounds=({:.1},{:.1},{:.1},{:.1})",
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height
        );

        // Expand bounds to accommodate shadows that extend beyond the blur region.
        // We use a generous margin since we don't know the actual shadow size here.
        // Content outside the expanded bounds will still be clipped, but typical
        // shadows (offset + blur_radius < 100px) will render correctly.
        const SHADOW_MARGIN: f32 = 100.0;
        let expanded_bounds = Rectangle {
            x: bounds.x - SHADOW_MARGIN,
            y: bounds.y - SHADOW_MARGIN,
            width: bounds.width + SHADOW_MARGIN * 2.0,
            height: bounds.height + SHADOW_MARGIN * 2.0,
        };

        // Push a new layer for post-blur content with expanded bounds
        // This ensures children (including shadows) are drawn to a dedicated layer
        // that can be skipped in the main render pass and rendered after blur
        self.layers.push_clip(expanded_bounds);
        let layer_count = self.layers.active_count();
        log::trace!(
            "start_post_blur_layer: pushed layer, now have {} layers, recording from layer {}",
            layer_count,
            layer_count - 1
        );

        // Start recording post-blur content - use the new layer's index
        // The new layer is at index (layer_count - 1)
        self.blur_state.start_post_blur(bounds, layer_count - 1);
    }

    fn end_post_blur_layer(&mut self) {
        let current_layer = self.layers.active_count() - 1;
        log::trace!(
            "end_post_blur_layer: current_layer={}, will record end_layer={}",
            current_layer,
            current_layer + 1
        );

        // Pop the clipping layer we pushed in start_post_blur_layer
        self.layers.pop_clip();

        // End recording post-blur content
        // The content is in the layer we just popped, so end_layer = current_layer + 1
        self.blur_state.end_post_blur(current_layer + 1);
    }
}

impl core::text::Renderer for Renderer {
    type Font = Font;
    type Paragraph = Paragraph;
    type Editor = Editor;

    const ICON_FONT: Font = Font::with_name("Iced-Icons");
    const CHECKMARK_ICON: char = '\u{f00c}';
    const ARROW_DOWN_ICON: char = '\u{e800}';
    const ICED_LOGO: char = '\u{e801}';
    const SCROLL_UP_ICON: char = '\u{e802}';
    const SCROLL_DOWN_ICON: char = '\u{e803}';
    const SCROLL_LEFT_ICON: char = '\u{e804}';
    const SCROLL_RIGHT_ICON: char = '\u{e805}';

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
        let opacity = self.current_opacity();
        let color = Color {
            a: color.a * opacity,
            ..color
        };
        let (layer, transformation) = self.layers.current_mut();

        layer.draw_paragraph(text, position, color, clip_bounds, transformation);
    }

    fn fill_editor(
        &mut self,
        editor: &Self::Editor,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        let opacity = self.current_opacity();
        let color = Color {
            a: color.a * opacity,
            ..color
        };
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
        let opacity = self.current_opacity();
        let color = Color {
            a: color.a * opacity,
            ..color
        };
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_text(text, position, color, clip_bounds, transformation);
    }
}

impl graphics::text::Renderer for Renderer {
    fn fill_raw(&mut self, raw: graphics::text::Raw) {
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_text_raw(raw, transformation);
    }
}

#[cfg(feature = "image")]
impl core::image::Renderer for Renderer {
    type Handle = core::image::Handle;

    fn load_image(
        &self,
        handle: &Self::Handle,
    ) -> Result<core::image::Allocation, core::image::Error> {
        self.image_cache
            .borrow_mut()
            .load_image(&self.engine.device, &self.engine.queue, handle)
    }

    fn measure_image(&self, handle: &Self::Handle) -> Option<core::Size<u32>> {
        self.image_cache.borrow_mut().measure_image(handle)
    }

    fn draw_image(&mut self, image: core::Image, bounds: Rectangle, clip_bounds: Rectangle) {
        let opacity = self.current_opacity();
        let image = core::Image {
            opacity: image.opacity * opacity,
            ..image
        };
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_raster(image, bounds, clip_bounds, transformation);
    }
}

#[cfg(feature = "svg")]
impl core::svg::Renderer for Renderer {
    fn measure_svg(&self, handle: &core::svg::Handle) -> core::Size<u32> {
        self.image_cache.borrow_mut().measure_svg(handle)
    }

    fn draw_svg(&mut self, svg: core::Svg, bounds: Rectangle, clip_bounds: Rectangle) {
        let opacity = self.current_opacity();
        let svg = core::Svg {
            opacity: svg.opacity * opacity,
            ..svg
        };
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_svg(svg, bounds, clip_bounds, transformation);
    }
}

impl graphics::mesh::Renderer for Renderer {
    fn draw_mesh(&mut self, mesh: graphics::Mesh) {
        debug_assert!(
            !mesh.indices().is_empty(),
            "Mesh must not have empty indices"
        );

        debug_assert!(
            mesh.indices().len().is_multiple_of(3),
            "Mesh indices length must be a multiple of 3"
        );

        let (layer, transformation) = self.layers.current_mut();
        layer.draw_mesh(mesh, transformation);
    }

    fn draw_mesh_cache(&mut self, cache: mesh::Cache) {
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_mesh_cache(cache, transformation);
    }
}

#[cfg(feature = "geometry")]
impl graphics::geometry::Renderer for Renderer {
    type Geometry = Geometry;
    type Frame = geometry::Frame;

    fn new_frame(&self, bounds: Rectangle) -> Self::Frame {
        geometry::Frame::new(bounds)
    }

    fn draw_geometry(&mut self, geometry: Self::Geometry) {
        let (layer, transformation) = self.layers.current_mut();

        match geometry {
            Geometry::Live {
                meshes,
                images,
                text,
            } => {
                layer.draw_mesh_group(meshes, transformation);

                for image in images {
                    layer.draw_image(image, transformation);
                }

                layer.draw_text_group(text, transformation);
            }
            Geometry::Cached(cache) => {
                if let Some(meshes) = cache.meshes {
                    layer.draw_mesh_cache(meshes, transformation);
                }

                if let Some(images) = cache.images {
                    for image in images.iter().cloned() {
                        layer.draw_image(image, transformation);
                    }
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
        layer.draw_primitive(bounds, primitive, transformation);
    }
}

impl graphics::compositor::Default for crate::Renderer {
    type Compositor = window::Compositor;
}

impl renderer::Headless for Renderer {
    async fn new(
        default_font: Font,
        default_text_size: Pixels,
        backend: Option<&str>,
    ) -> Option<Self> {
        if backend.is_some_and(|backend| backend != "wgpu") {
            return None;
        }

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::from_env().unwrap_or(wgpu::Backends::PRIMARY),
            flags: wgpu::InstanceFlags::empty(),
            ..wgpu::InstanceDescriptor::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .ok()?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("iced_wgpu [headless]"),
                required_features: {
                    let mut features = wgpu::Features::empty();
                    if adapter
                        .features()
                        .contains(wgpu::Features::VULKAN_EXTERNAL_MEMORY_DMA_BUF)
                    {
                        features |= wgpu::Features::VULKAN_EXTERNAL_MEMORY_DMA_BUF;
                    }
                    if adapter
                        .features()
                        .contains(wgpu::Features::TEXTURE_FORMAT_16BIT_NORM)
                    {
                        features |= wgpu::Features::TEXTURE_FORMAT_16BIT_NORM;
                    }
                    features
                },
                required_limits: wgpu::Limits {
                    max_bind_groups: 2,
                    ..wgpu::Limits::default()
                },
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
            })
            .await
            .ok()?;

        let engine = Engine::new(
            &adapter,
            device,
            queue,
            if graphics::color::GAMMA_CORRECTION {
                wgpu::TextureFormat::Rgba8UnormSrgb
            } else {
                wgpu::TextureFormat::Rgba8Unorm
            },
            Some(graphics::Antialiasing::MSAAx4),
            Shell::headless(),
        );

        Some(Self::new(engine, default_font, default_text_size))
    }

    fn name(&self) -> String {
        "wgpu".to_owned()
    }

    fn screenshot(
        &mut self,
        size: Size<u32>,
        scale_factor: f32,
        background_color: Color,
    ) -> Vec<u8> {
        self.screenshot(
            &Viewport::with_physical_size(size, scale_factor),
            background_color,
        )
    }
}
