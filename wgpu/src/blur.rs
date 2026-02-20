//! Backdrop blur rendering support.
//!
//! This module provides the ability to blur content behind a widget,
//! similar to CSS backdrop-filter: blur().
//!
//! The implementation uses a two-pass Gaussian blur (horizontal + vertical)
//! for efficient O(n) blur instead of O(n²).

use crate::core::{Rectangle, Size};
use crate::graphics::Viewport;
use std::borrow::Cow;
use wgpu::util::DeviceExt;

/// Configuration for a backdrop blur effect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackdropBlur {
    /// The bounds where the blur applies
    pub bounds: Rectangle,
    /// Blur radius in logical pixels
    pub radius: f32,
    /// Border radius [top_left, top_right, bottom_right, bottom_left] in logical pixels
    pub border_radius: [f32; 4],
}

/// A backdrop blur region with layer indices for tracking which layers to render.
#[derive(Debug, Clone)]
pub struct BlurRegion {
    /// The blur configuration
    pub blur: BackdropBlur,
    /// The starting layer index (content BEFORE this is what gets blurred)
    pub layer_index: usize,
}

impl BackdropBlur {
    /// Creates a new backdrop blur with the given bounds and radius.
    pub fn new(bounds: Rectangle, radius: f32) -> Self {
        Self {
            bounds,
            radius: radius.max(0.0),
            border_radius: [0.0; 4],
        }
    }

    /// Creates a new backdrop blur with the given bounds, radius, and border radius.
    pub fn with_border_radius(bounds: Rectangle, radius: f32, border_radius: [f32; 4]) -> Self {
        Self {
            bounds,
            radius: radius.max(0.0),
            border_radius,
        }
    }
}

/// Uniform data for the blur shader.
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
struct BlurUniforms {
    /// Quad bounds in normalized device coordinates (x, y, width, height) - expanded for blur sampling
    quad_bounds: [f32; 4],
    /// Clip bounds in normalized device coordinates (x, y, width, height) - original widget bounds for SDF
    clip_bounds: [f32; 4],
    /// params.x = blur_radius, params.y = direction (0=horizontal, 1=vertical)
    /// params.z = texture_width, params.w = texture_height
    params: [f32; 4],
    /// Border radius [top_left, top_right, bottom_right, bottom_left] in pixels
    border_radius: [f32; 4],
}

/// Pipeline for rendering blur effects.
#[derive(Debug, Clone)]
pub struct Pipeline {
    /// Pipeline without blending (for intermediate passes)
    pipeline: wgpu::RenderPipeline,
    /// Pipeline with alpha blending (for final pass with border radius)
    pipeline_blend: wgpu::RenderPipeline,
    constant_layout: wgpu::BindGroupLayout,
    texture_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl Pipeline {
    /// Creates a new blur pipeline.
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("iced_wgpu.blur.sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..wgpu::SamplerDescriptor::default()
        });

        let constant_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("iced_wgpu.blur.constant_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("iced_wgpu.blur.texture_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("iced_wgpu.blur.pipeline_layout"),
            bind_group_layouts: &[&constant_layout, &texture_layout],
            immediate_size: 0,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("iced_wgpu.blur.shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader/blur.wgsl"))),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("iced_wgpu.blur.pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None, // No blending for intermediate passes
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Cw,
                ..wgpu::PrimitiveState::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // Pipeline with alpha blending for final pass (border radius clipping)
        let pipeline_blend = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("iced_wgpu.blur.pipeline_blend"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Cw,
                ..wgpu::PrimitiveState::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            pipeline_blend,
            constant_layout,
            texture_layout,
            sampler,
        }
    }

    /// Performs a single blur pass (horizontal or vertical).
    fn blur_pass(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source_texture: &wgpu::TextureView,
        target: &wgpu::TextureView,
        quad_bounds: &[f32; 4], // Expanded bounds for the rendered quad
        clip_bounds: &[f32; 4], // Original bounds for SDF clipping
        radius: f32,
        direction: u32, // 0 = horizontal, 1 = vertical
        target_width: f32,
        target_height: f32,
        border_radius: [f32; 4],
        use_blend: bool, // Use blending pipeline for final pass with border radius
    ) {
        let uniforms = BlurUniforms {
            quad_bounds: *quad_bounds,
            clip_bounds: *clip_bounds,
            params: [radius, direction as f32, target_width, target_height],
            border_radius,
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("iced_wgpu.blur.uniform_buffer"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let constant_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("iced_wgpu.blur.constant_bind_group"),
            layout: &self.constant_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("iced_wgpu.blur.texture_bind_group"),
            layout: &self.texture_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(source_texture),
            }],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("iced_wgpu.blur.render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
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
        });

        // Set viewport to match actual content size (textures may be larger due to grow-only policy)
        pass.set_viewport(0.0, 0.0, target_width, target_height, 0.0, 1.0);

        // Use blending pipeline for final pass (border radius needs alpha blending)
        let pipeline = if use_blend {
            &self.pipeline_blend
        } else {
            &self.pipeline
        };
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &constant_bind_group, &[]);
        pass.set_bind_group(1, &texture_bind_group, &[]);
        pass.draw(0..6, 0..1); // 6 vertices for bounds quad
    }

    /// Renders the blur effect using two passes (horizontal + vertical).
    ///
    /// Requires an intermediate texture for the two-pass blur.
    /// Uses iterative passes for large blur radii to achieve smooth results.
    pub fn render(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source_texture: &wgpu::TextureView,
        intermediate_texture: &wgpu::TextureView,
        target: &wgpu::TextureView,
        blur: &BackdropBlur,
        viewport: &Viewport,
    ) {
        let physical_size = viewport.physical_size();
        let scale_factor = viewport.scale_factor();

        // Calculate normalized clip bounds (original widget bounds for SDF)
        let bounds = blur.bounds * scale_factor;
        let clip_bounds = [
            bounds.x / physical_size.width as f32,
            bounds.y / physical_size.height as f32,
            bounds.width / physical_size.width as f32,
            bounds.height / physical_size.height as f32,
        ];

        let total_radius = blur.radius * scale_factor;
        let tex_width = physical_size.width as f32;
        let tex_height = physical_size.height as f32;

        // Calculate expanded quad bounds (add padding to sample beyond edges)
        // W3C box blur formula: d = floor(sigma * 1.88 + 0.5)
        // Each box blur pass samples ±(d-1)/2 ≈ ±sigma pixels
        // After 3 passes, the effective sampling range compounds slightly
        // Using 3*sigma as padding provides good coverage
        let padding = total_radius * 3.0;
        let expanded_x = (bounds.x - padding).max(0.0);
        let expanded_y = (bounds.y - padding).max(0.0);
        let expanded_right = (bounds.x + bounds.width + padding).min(tex_width);
        let expanded_bottom = (bounds.y + bounds.height + padding).min(tex_height);
        let quad_bounds = [
            expanded_x / tex_width,
            expanded_y / tex_height,
            (expanded_right - expanded_x) / tex_width,
            (expanded_bottom - expanded_y) / tex_height,
        ];

        log::trace!(
            "blur render: logical_bounds={:?}, physical_bounds=({:.1},{:.1},{:.1},{:.1}), \
             expanded_quad=({:.1},{:.1},{:.1},{:.1}), radius={:.1}",
            blur.bounds,
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height,
            expanded_x,
            expanded_y,
            expanded_right - expanded_x,
            expanded_bottom - expanded_y,
            total_radius
        );

        // Scale border radius by scale factor
        let scaled_border_radius = [
            blur.border_radius[0] * scale_factor,
            blur.border_radius[1] * scale_factor,
            blur.border_radius[2] * scale_factor,
            blur.border_radius[3] * scale_factor,
        ];

        // W3C spec recommends three successive box-blurs to approximate Gaussian:
        // "Three successive box-blurs build a piece-wise quadratic convolution kernel,
        //  which approximates the Gaussian kernel to within roughly 3%."
        //
        // Each box blur is separable (H+V), so we do 6 total passes:
        // Pass 1: H blur (source -> intermediate)
        // Pass 2: V blur (intermediate -> source) - reuse source as temp
        // Pass 3: H blur (source -> intermediate)
        // Pass 4: V blur (intermediate -> source)
        // Pass 5: H blur (source -> intermediate)
        // Pass 6: V blur (intermediate -> target) with border radius

        // Check if we need blending (has border radius)
        let has_border_radius = scaled_border_radius.iter().any(|&r| r > 0.0);

        // For 3 box blur passes, we ping-pong between source and intermediate
        // Note: source_texture must be both RENDER_ATTACHMENT and TEXTURE_BINDING capable

        // Pass 1: H blur (source -> intermediate)
        self.blur_pass(
            device,
            encoder,
            source_texture,
            intermediate_texture,
            &quad_bounds,
            &clip_bounds,
            total_radius,
            0, // horizontal
            tex_width,
            tex_height,
            [0.0; 4],
            false,
        );

        // Pass 2: V blur (intermediate -> source)
        self.blur_pass(
            device,
            encoder,
            intermediate_texture,
            source_texture,
            &quad_bounds,
            &clip_bounds,
            total_radius,
            1, // vertical
            tex_width,
            tex_height,
            [0.0; 4],
            false,
        );

        // Pass 3: H blur (source -> intermediate)
        self.blur_pass(
            device,
            encoder,
            source_texture,
            intermediate_texture,
            &quad_bounds,
            &clip_bounds,
            total_radius,
            0, // horizontal
            tex_width,
            tex_height,
            [0.0; 4],
            false,
        );

        // Pass 4: V blur (intermediate -> source)
        self.blur_pass(
            device,
            encoder,
            intermediate_texture,
            source_texture,
            &quad_bounds,
            &clip_bounds,
            total_radius,
            1, // vertical
            tex_width,
            tex_height,
            [0.0; 4],
            false,
        );

        // Pass 5: H blur (source -> intermediate)
        self.blur_pass(
            device,
            encoder,
            source_texture,
            intermediate_texture,
            &quad_bounds,
            &clip_bounds,
            total_radius,
            0, // horizontal
            tex_width,
            tex_height,
            [0.0; 4],
            false,
        );

        // Pass 6: V blur (intermediate -> target) with border radius
        self.blur_pass(
            device,
            encoder,
            intermediate_texture,
            target,
            &quad_bounds,
            &clip_bounds,
            total_radius,
            1, // vertical
            tex_width,
            tex_height,
            scaled_border_radius,
            has_border_radius,
        );
    }

    /// Copies (blits) content from source to destination using viewport.
    /// Uses the blur shader with radius=0 which acts as a simple copy.
    pub fn blit(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source_texture: &wgpu::TextureView,
        target: &wgpu::TextureView,
        viewport: &Viewport,
    ) {
        let physical_size = viewport.physical_size();
        self.blit_full(device, encoder, source_texture, target, physical_size);
    }

    /// Copies (blits) full content from source to destination.
    /// Uses the blur shader with radius=0 which acts as a simple copy.
    pub fn blit_full(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source_texture: &wgpu::TextureView,
        target: &wgpu::TextureView,
        physical_size: Size<u32>,
    ) {
        let tex_width = physical_size.width as f32;
        let tex_height = physical_size.height as f32;

        // Full screen bounds (normalized)
        let full_bounds = [0.0, 0.0, 1.0, 1.0];

        // Use blur pass with radius=0 to just copy
        self.blur_pass(
            device,
            encoder,
            source_texture,
            target,
            &full_bounds,
            &full_bounds, // clip_bounds same as quad_bounds for full blit
            0.0,          // radius = 0 means just copy
            0,            // direction doesn't matter for radius=0
            tex_width,
            tex_height,
            [0.0; 4], // No border radius for full blit
            false,    // No blending for blit
        );
    }

    /// Copies (blits) a specific region from source to destination.
    /// Uses the blur shader with radius=0 which acts as a simple copy.
    pub fn blit_region(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source_texture: &wgpu::TextureView,
        target: &wgpu::TextureView,
        bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        let physical_size = viewport.physical_size();
        let scale_factor = viewport.scale_factor();

        // Calculate normalized bounds
        let scaled_bounds = *bounds * scale_factor;
        let normalized_bounds = [
            scaled_bounds.x / physical_size.width as f32,
            scaled_bounds.y / physical_size.height as f32,
            scaled_bounds.width / physical_size.width as f32,
            scaled_bounds.height / physical_size.height as f32,
        ];

        let tex_width = physical_size.width as f32;
        let tex_height = physical_size.height as f32;

        log::debug!(
            "blit_region: bounds=({:.2}, {:.2}, {:.2}, {:.2})",
            normalized_bounds[0],
            normalized_bounds[1],
            normalized_bounds[2],
            normalized_bounds[3]
        );

        // Use blur pass with radius=0 to just copy the region
        self.blur_pass(
            device,
            encoder,
            source_texture,
            target,
            &normalized_bounds,
            &normalized_bounds, // clip_bounds same as quad_bounds for region blit
            0.0,                // radius = 0 means just copy
            0,                  // direction doesn't matter for radius=0
            tex_width,
            tex_height,
            [0.0; 4], // No border radius for region blit
            false,    // No blending for blit
        );
    }
}

/// Represents content that should be rendered after blur effects.
#[derive(Debug, Clone)]
pub struct PostBlurContent {
    /// The bounds of this post-blur layer
    pub bounds: Rectangle,
    /// The layer index where this content starts
    pub start_layer: usize,
    /// The layer index where this content ends (exclusive)
    pub end_layer: Option<usize>,
}

/// State for managing blur rendering.
#[derive(Debug, Default)]
pub struct State {
    /// Pending blur regions to process
    regions: Vec<BlurRegion>,
    /// Content that should be rendered after blur
    post_blur_content: Vec<PostBlurContent>,
    /// Currently recording post-blur content (bounds, start_layer)
    current_post_blur: Option<(Rectangle, usize)>,
}

impl State {
    /// Creates a new blur state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a blur region.
    pub fn add_region(&mut self, blur: BackdropBlur, layer_index: usize) {
        self.regions.push(BlurRegion { blur, layer_index });
    }

    /// Takes all pending regions, clearing the state.
    pub fn take_regions(&mut self) -> Vec<BlurRegion> {
        std::mem::take(&mut self.regions)
    }

    /// Returns true if there are pending blur regions.
    pub fn has_regions(&self) -> bool {
        !self.regions.is_empty()
    }

    /// Clears all pending regions.
    pub fn clear(&mut self) {
        self.regions.clear();
    }

    /// Begins recording post-blur content.
    pub fn start_post_blur(&mut self, bounds: Rectangle, layer_index: usize) {
        log::trace!(
            "start_post_blur: bounds={:?}, layer_index={}",
            bounds,
            layer_index
        );
        self.current_post_blur = Some((bounds, layer_index));
    }

    /// Ends recording post-blur content.
    pub fn end_post_blur(&mut self, end_layer: usize) {
        log::trace!("end_post_blur: end_layer={}", end_layer);
        if let Some((bounds, start_layer)) = self.current_post_blur.take() {
            log::trace!(
                "Recording post-blur content: start_layer={}, end_layer={}",
                start_layer,
                end_layer
            );
            self.post_blur_content.push(PostBlurContent {
                bounds,
                start_layer,
                end_layer: Some(end_layer),
            });
        }
    }

    /// Checks if a layer index is within any post-blur content region.
    /// Used to skip these layers in the first render pass.
    pub fn is_layer_in_post_blur(&self, layer_index: usize) -> bool {
        self.post_blur_content.iter().any(|content| {
            let end = content.end_layer.unwrap_or(usize::MAX);
            layer_index >= content.start_layer && layer_index < end
        }) || self
            .current_post_blur
            .as_ref()
            .is_some_and(|(_, start)| layer_index >= *start)
    }

    /// Takes all post-blur content, clearing the state.
    pub fn take_post_blur_content(&mut self) -> Vec<PostBlurContent> {
        std::mem::take(&mut self.post_blur_content)
    }

    /// Returns true if there is post-blur content to render.
    pub fn has_post_blur_content(&self) -> bool {
        !self.post_blur_content.is_empty()
    }

    /// Returns the post-blur content without taking ownership.
    pub fn post_blur_content(&self) -> &[PostBlurContent] {
        &self.post_blur_content
    }
}

/// Texture cache for blur operations.
///
/// Maintains intermediate textures needed for two-pass blur.
#[derive(Debug)]
pub struct TextureCache {
    /// Intermediate texture for blur passes
    intermediate: Option<(wgpu::Texture, wgpu::TextureView, Size<u32>)>,
    /// Copy of the scene before blur regions
    scene_copy: Option<(wgpu::Texture, wgpu::TextureView, Size<u32>)>,
}

impl TextureCache {
    /// Creates a new texture cache.
    pub fn new() -> Self {
        Self {
            intermediate: None,
            scene_copy: None,
        }
    }

    /// Gets or creates the intermediate texture for blur passes.
    /// Texture is recreated if size doesn't match exactly (blur requires 1:1 pixel mapping).
    pub fn get_intermediate(
        &mut self,
        device: &wgpu::Device,
        size: Size<u32>,
        format: wgpu::TextureFormat,
    ) -> &wgpu::TextureView {
        // Blur requires exact 1:1 pixel mapping - recreate if size doesn't match
        let needs_resize = self
            .intermediate
            .as_ref()
            .is_none_or(|(_, _, s)| s.width != size.width || s.height != size.height);

        if needs_resize {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("iced_wgpu.blur.intermediate_texture"),
                size: wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            self.intermediate = Some((texture, view, size));
        }

        &self.intermediate.as_ref().unwrap().1
    }

    /// Gets or creates a texture to copy the scene for blurring.
    /// Texture is recreated if size doesn't match exactly (blur requires 1:1 pixel mapping).
    pub fn get_scene_copy(
        &mut self,
        device: &wgpu::Device,
        size: Size<u32>,
        format: wgpu::TextureFormat,
    ) -> &wgpu::TextureView {
        // Blur requires exact 1:1 pixel mapping - recreate if size doesn't match
        let needs_resize = self
            .scene_copy
            .as_ref()
            .is_none_or(|(_, _, s)| s.width != size.width || s.height != size.height);

        if needs_resize {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("iced_wgpu.blur.scene_copy_texture"),
                size: wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            self.scene_copy = Some((texture, view, size));
        }

        &self.scene_copy.as_ref().unwrap().1
    }

    /// Gets the raw scene copy texture for copy operations.
    pub fn get_scene_copy_texture(&self) -> Option<&wgpu::Texture> {
        self.scene_copy.as_ref().map(|(t, _, _)| t)
    }

    /// Gets or creates both intermediate and scene copy textures, returning their views.
    ///
    /// This method ensures both textures exist and returns their views together,
    /// avoiding borrow checker issues when both are needed simultaneously.
    pub fn get_blur_textures(
        &mut self,
        device: &wgpu::Device,
        size: Size<u32>,
        format: wgpu::TextureFormat,
    ) -> (&wgpu::TextureView, &wgpu::TextureView) {
        // Ensure intermediate texture exists (exact match required for blur)
        let needs_intermediate_resize = self
            .intermediate
            .as_ref()
            .is_none_or(|(_, _, s)| s.width != size.width || s.height != size.height);

        if needs_intermediate_resize {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("iced_wgpu.blur.intermediate_texture"),
                size: wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            self.intermediate = Some((texture, view, size));
        }

        // Ensure scene_copy texture exists (exact match required for blur)
        let needs_scene_copy_resize = self
            .scene_copy
            .as_ref()
            .is_none_or(|(_, _, s)| s.width != size.width || s.height != size.height);

        if needs_scene_copy_resize {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("iced_wgpu.blur.scene_copy_texture"),
                size: wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            self.scene_copy = Some((texture, view, size));
        }

        // Now both are guaranteed to exist, return references
        (
            &self.scene_copy.as_ref().unwrap().1,
            &self.intermediate.as_ref().unwrap().1,
        )
    }
}

impl Default for TextureCache {
    fn default() -> Self {
        Self::new()
    }
}
