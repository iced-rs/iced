//! Gradient fade rendering support.
//!
//! This module provides the ability to render content to an offscreen texture
//! and then composite it with a gradient alpha mask for fade effects.
//!
//! The implementation uses layer-index tracking:
//! 1. When `start_gradient_fade` is called, we flush current layers and record the layer count
//! 2. Content is drawn normally (primitives added to new layers)
//! 3. When `end_gradient_fade` is called, we record the ending layer count
//! 4. During rendering, layers within gradient fade regions are rendered to an offscreen
//!    texture, then composited back with gradient alpha

use crate::core::{Rectangle, Size};
use crate::graphics::Viewport;
use std::borrow::Cow;
use wgpu::util::DeviceExt;

/// Direction of the gradient fade effect.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FadeDirection {
    /// Fade from top to bottom (content fades out at the bottom)
    #[default]
    TopToBottom,
    /// Fade from bottom to top (content fades out at the top)
    BottomToTop,
    /// Fade from left to right (content fades out on the right)
    LeftToRight,
    /// Fade from right to left (content fades out on the left)
    RightToLeft,
    /// Fade at both top and bottom edges
    VerticalBoth,
    /// Fade at both left and right edges
    HorizontalBoth,
}

/// Configuration for a gradient fade effect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GradientFade {
    /// The bounds where the gradient fade applies
    pub bounds: Rectangle,
    /// Direction of the fade
    pub direction: FadeDirection,
    /// Where the fade starts (0.0 = start of bounds, 1.0 = end of bounds)
    /// Content before this point is fully opaque.
    pub fade_start: f32,
    /// Where the fade ends (0.0 = start of bounds, 1.0 = end of bounds)
    /// Content after this point is fully transparent.
    pub fade_end: f32,
}

/// A gradient fade region with layer indices for tracking which layers to render offscreen.
#[derive(Debug, Clone)]
pub struct GradientFadeRegion {
    /// The fade configuration
    pub fade: GradientFade,
    /// The starting layer index (inclusive)
    pub start_layer: usize,
    /// The ending layer index (exclusive)
    pub end_layer: usize,
}

impl GradientFade {
    /// Creates a new gradient fade with the given bounds.
    pub fn new(bounds: Rectangle) -> Self {
        Self {
            bounds,
            direction: FadeDirection::default(),
            fade_start: 0.7,
            fade_end: 1.0,
        }
    }

    /// Sets the direction of the fade.
    pub fn direction(mut self, direction: FadeDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Sets the fade start position (0.0 to 1.0).
    pub fn fade_start(mut self, start: f32) -> Self {
        self.fade_start = start.clamp(0.0, 1.0);
        self
    }

    /// Sets the fade end position (0.0 to 1.0).
    pub fn fade_end(mut self, end: f32) -> Self {
        self.fade_end = end.clamp(0.0, 1.0);
        self
    }
}

/// Uniform data for the gradient fade shader.
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
struct GradientFadeUniforms {
    /// Bounds in normalized device coordinates (x, y, width, height)
    bounds: [f32; 4],
    /// Fade parameters: (direction, fade_start, fade_end, _padding)
    /// direction: 0 = TopToBottom, 1 = BottomToTop, 2 = LeftToRight, 3 = RightToLeft
    params: [f32; 4],
}

/// Pipeline for rendering gradient fade effects.
#[derive(Debug, Clone)]
pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    constant_layout: wgpu::BindGroupLayout,
    texture_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl Pipeline {
    /// Creates a new gradient fade pipeline.
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("iced_wgpu.gradient_fade.sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..wgpu::SamplerDescriptor::default()
        });

        let constant_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("iced_wgpu.gradient_fade.constant_layout"),
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
            label: Some("iced_wgpu.gradient_fade.texture_layout"),
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
            label: Some("iced_wgpu.gradient_fade.pipeline_layout"),
            bind_group_layouts: &[&constant_layout, &texture_layout],
            immediate_size: 0,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("iced_wgpu.gradient_fade.shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "shader/gradient_fade.wgsl"
            ))),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("iced_wgpu.gradient_fade.pipeline"),
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
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
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
            constant_layout,
            texture_layout,
            sampler,
        }
    }

    /// Renders the offscreen texture with a gradient fade effect to the target.
    pub fn render(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source_texture: &wgpu::TextureView,
        target: &wgpu::TextureView,
        fade: &GradientFade,
        viewport: &Viewport,
    ) {
        let physical_size = viewport.physical_size();
        let scale_factor = viewport.scale_factor();

        // Calculate normalized bounds
        let bounds = fade.bounds * scale_factor;
        let normalized_bounds = [
            bounds.x / physical_size.width as f32,
            bounds.y / physical_size.height as f32,
            bounds.width / physical_size.width as f32,
            bounds.height / physical_size.height as f32,
        ];

        let direction = match fade.direction {
            FadeDirection::TopToBottom => 0.0,
            FadeDirection::BottomToTop => 1.0,
            FadeDirection::LeftToRight => 2.0,
            FadeDirection::RightToLeft => 3.0,
            FadeDirection::VerticalBoth => 4.0,
            FadeDirection::HorizontalBoth => 5.0,
        };

        let uniforms = GradientFadeUniforms {
            bounds: normalized_bounds,
            params: [direction, fade.fade_start, fade.fade_end, 0.0],
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("iced_wgpu.gradient_fade.uniform_buffer"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let constant_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("iced_wgpu.gradient_fade.constant_bind_group"),
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
            label: Some("iced_wgpu.gradient_fade.texture_bind_group"),
            layout: &self.texture_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(source_texture),
            }],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("iced_wgpu.gradient_fade.render_pass"),
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

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &constant_bind_group, &[]);
        pass.set_bind_group(1, &texture_bind_group, &[]);
        pass.draw(0..6, 0..1);
    }
}

/// State for managing gradient fade rendering.
///
/// This tracks layer indices to know which layers belong to gradient fade regions.
#[derive(Debug)]
pub struct State {
    /// Completed gradient fade regions with their layer indices
    completed_regions: Vec<GradientFadeRegion>,
    /// Active (not yet closed) fade with starting layer index
    active: Option<ActiveFade>,
    /// Offscreen texture for rendering content
    offscreen_texture: Option<OffscreenTexture>,
}

/// An active (not yet closed) gradient fade.
#[derive(Debug, Clone)]
struct ActiveFade {
    fade: GradientFade,
    start_layer: usize,
}

#[derive(Debug)]
struct OffscreenTexture {
    #[allow(dead_code)]
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    size: Size<u32>,
}

impl State {
    /// Creates a new gradient fade state.
    pub fn new() -> Self {
        Self {
            completed_regions: Vec::new(),
            active: None,
            offscreen_texture: None,
        }
    }

    /// Returns whether a gradient fade is currently active.
    pub fn is_active(&self) -> bool {
        self.active.is_some()
    }

    /// Starts a new gradient fade effect, recording the current layer count.
    pub fn start(&mut self, fade: GradientFade, current_layer_count: usize) {
        self.active = Some(ActiveFade {
            fade,
            start_layer: current_layer_count,
        });
    }

    /// Returns the current active gradient fade configuration.
    pub fn current(&self) -> Option<&GradientFade> {
        self.active.as_ref().map(|a| &a.fade)
    }

    /// Ends the current gradient fade and records the layer range.
    pub fn end(&mut self, current_layer_count: usize) -> Option<GradientFadeRegion> {
        if let Some(active) = self.active.take() {
            let region = GradientFadeRegion {
                fade: active.fade,
                start_layer: active.start_layer,
                end_layer: current_layer_count,
            };
            self.completed_regions.push(region.clone());
            Some(region)
        } else {
            None
        }
    }

    /// Returns completed gradient fade regions and clears the list.
    pub fn take_regions(&mut self) -> Vec<GradientFadeRegion> {
        std::mem::take(&mut self.completed_regions)
    }

    /// Returns true if a layer index is within any gradient fade region.
    pub fn is_layer_in_fade_region(&self, layer_index: usize) -> bool {
        self.completed_regions
            .iter()
            .any(|r| layer_index >= r.start_layer && layer_index < r.end_layer)
    }

    /// Returns the fade region that contains the given layer index, if any.
    pub fn get_region_for_layer(&self, layer_index: usize) -> Option<&GradientFadeRegion> {
        self.completed_regions
            .iter()
            .find(|r| layer_index >= r.start_layer && layer_index < r.end_layer)
    }

    /// Clears all pending fades (called at the start of a new frame).
    pub fn clear(&mut self) {
        self.completed_regions.clear();
        self.active = None;
    }

    /// Gets or creates an offscreen texture of the given size.
    pub fn get_or_create_texture(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        size: Size<u32>,
    ) -> &wgpu::TextureView {
        // Check if we need to recreate the texture
        let needs_recreate = self
            .offscreen_texture
            .as_ref()
            .map(|t| t.size != size)
            .unwrap_or(true);

        if needs_recreate {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("iced_wgpu.gradient_fade.offscreen_texture"),
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

            self.offscreen_texture = Some(OffscreenTexture {
                texture,
                view,
                size,
            });
        }

        &self.offscreen_texture.as_ref().unwrap().view
    }

    /// Returns the current offscreen texture view if available.
    #[allow(dead_code)]
    pub fn texture_view(&self) -> Option<&wgpu::TextureView> {
        self.offscreen_texture.as_ref().map(|t| &t.view)
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
