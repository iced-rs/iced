//! Cached draw rendering support.
//!
//! This module provides the ability to render content to a persistent offscreen
//! texture and replay it in subsequent frames without re-executing draw calls.
//!
//! Used for crossfade transitions: on each non-transitioning frame the content
//! is rendered normally AND captured to a keyed texture. When a transition starts,
//! the cached texture represents the "old" page and is composited at decreasing
//! opacity while the new page fades in.
//!
//! Implementation uses layer-index tracking (same pattern as gradient_fade):
//! 1. `start_cached_draw(key)` → flush layers, record start_layer index
//! 2. Content drawn normally (primitives added to layers)
//! 3. `end_cached_draw()` → flush layers, record end_layer index
//! 4. During render: layers in the region are rendered to both the main target
//!    AND the persistent texture. The texture persists for use by `draw_cached()`.

use crate::core::{Rectangle, Size};
use crate::graphics::Viewport;
use std::borrow::Cow;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

/// A cached draw region with layer indices for tracking which layers to capture.
#[derive(Debug, Clone)]
pub struct CachedDrawRegion {
    /// Stable key identifying this cached surface.
    pub key: u64,
    /// The logical bounds of the content.
    pub bounds: Rectangle,
    /// The starting layer index (inclusive).
    pub start_layer: usize,
    /// The ending layer index (exclusive).
    pub end_layer: usize,
}

/// A request to composite a previously cached texture.
#[derive(Debug, Clone)]
pub struct CachedDrawComposite {
    /// The key identifying which cached texture to draw.
    pub key: u64,
    /// The logical bounds at which to composite.
    pub bounds: Rectangle,
    /// Opacity for compositing (0.0 to 1.0).
    pub opacity: f32,
    /// The layer index at which this composite should be inserted in draw order.
    pub at_layer: usize,
}

/// Uniform data for the cached draw shader.
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
struct CachedDrawUniforms {
    /// Bounds in normalized device coordinates (x, y, width, height).
    bounds: [f32; 4],
    /// params.x = opacity, params.yzw = unused.
    params: [f32; 4],
}

/// A persistent cached texture entry.
#[derive(Debug)]
struct CachedEntry {
    #[allow(dead_code)]
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    size: Size<u32>,
}

/// GPU pipeline for compositing cached textures.
#[derive(Debug, Clone)]
pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    constant_layout: wgpu::BindGroupLayout,
    texture_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl Pipeline {
    /// Creates a new cached draw pipeline.
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("iced_wgpu.cached_draw.sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..wgpu::SamplerDescriptor::default()
        });

        let constant_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("iced_wgpu.cached_draw.constant_layout"),
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
            label: Some("iced_wgpu.cached_draw.texture_layout"),
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
            label: Some("iced_wgpu.cached_draw.pipeline_layout"),
            bind_group_layouts: &[&constant_layout, &texture_layout],
            immediate_size: 0,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("iced_wgpu.cached_draw.shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "shader/cached_draw.wgsl"
            ))),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("iced_wgpu.cached_draw.pipeline"),
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

    /// Composites a cached texture onto the target with the given opacity.
    pub fn render(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source_texture: &wgpu::TextureView,
        target: &wgpu::TextureView,
        bounds: Rectangle,
        opacity: f32,
        viewport: &Viewport,
    ) {
        let physical_size = viewport.physical_size();
        let scale_factor = viewport.scale_factor();

        let bounds = bounds * scale_factor;
        let normalized_bounds = [
            bounds.x / physical_size.width as f32,
            bounds.y / physical_size.height as f32,
            bounds.width / physical_size.width as f32,
            bounds.height / physical_size.height as f32,
        ];

        let uniforms = CachedDrawUniforms {
            bounds: normalized_bounds,
            params: [opacity, 0.0, 0.0, 0.0],
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("iced_wgpu.cached_draw.uniform_buffer"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let constant_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("iced_wgpu.cached_draw.constant_bind_group"),
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
            label: Some("iced_wgpu.cached_draw.texture_bind_group"),
            layout: &self.texture_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(source_texture),
            }],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("iced_wgpu.cached_draw.render_pass"),
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

/// State for managing cached draw rendering.
///
/// Tracks layer indices for active recordings and maintains persistent
/// textures keyed by u64 identifiers.
#[derive(Debug)]
pub struct State {
    /// Completed cached draw regions for the current frame.
    completed_regions: Vec<CachedDrawRegion>,
    /// Pending composite operations for the current frame.
    composite_requests: Vec<CachedDrawComposite>,
    /// Active (not yet closed) recording.
    active: Option<ActiveCachedDraw>,
    /// Persistent cached textures keyed by ID.
    cache: HashMap<u64, CachedEntry>,
}

/// An active (not yet closed) cached draw recording.
#[derive(Debug, Clone)]
struct ActiveCachedDraw {
    key: u64,
    bounds: Rectangle,
    start_layer: usize,
}

impl State {
    /// Creates a new cached draw state.
    pub fn new() -> Self {
        Self {
            completed_regions: Vec::new(),
            composite_requests: Vec::new(),
            active: None,
            cache: HashMap::new(),
        }
    }

    /// Returns whether a cached draw is currently being recorded.
    pub fn is_active(&self) -> bool {
        self.active.is_some()
    }

    /// Starts a new cached draw recording.
    pub fn start(&mut self, key: u64, bounds: Rectangle, current_layer_count: usize) {
        self.active = Some(ActiveCachedDraw {
            key,
            bounds,
            start_layer: current_layer_count,
        });
    }

    /// Ends the current cached draw recording.
    pub fn end(&mut self, current_layer_count: usize) {
        if let Some(active) = self.active.take() {
            self.completed_regions.push(CachedDrawRegion {
                key: active.key,
                bounds: active.bounds,
                start_layer: active.start_layer,
                end_layer: current_layer_count,
            });
        }
    }

    /// Requests compositing a previously cached texture.
    pub fn request_composite(
        &mut self,
        key: u64,
        bounds: Rectangle,
        opacity: f32,
        at_layer: usize,
    ) {
        self.composite_requests.push(CachedDrawComposite {
            key,
            bounds,
            opacity,
            at_layer,
        });
    }

    /// Returns completed regions and clears the per-frame list.
    pub fn take_regions(&mut self) -> Vec<CachedDrawRegion> {
        std::mem::take(&mut self.completed_regions)
    }

    /// Returns composite requests and clears the per-frame list.
    pub fn take_composites(&mut self) -> Vec<CachedDrawComposite> {
        std::mem::take(&mut self.composite_requests)
    }

    /// Returns true if a layer index is within any cached draw region.
    pub fn is_layer_in_region(&self, layer_index: usize) -> bool {
        self.completed_regions
            .iter()
            .any(|r| layer_index >= r.start_layer && layer_index < r.end_layer)
    }

    /// Gets or creates a persistent texture for the given key and size.
    pub fn get_or_create_texture(
        &mut self,
        key: u64,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        size: Size<u32>,
    ) -> &wgpu::TextureView {
        let entry = self.cache.entry(key).or_insert_with(|| {
            let texture = create_texture(device, format, size, key);
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            CachedEntry {
                texture,
                view,
                size,
            }
        });

        // Recreate if size changed
        if entry.size != size {
            let texture = create_texture(device, format, size, key);
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            *entry = CachedEntry {
                texture,
                view,
                size,
            };
        }

        &entry.view
    }

    /// Returns the texture view for a previously cached key, if it exists.
    pub fn texture_view(&self, key: u64) -> Option<&wgpu::TextureView> {
        self.cache.get(&key).map(|e| &e.view)
    }

    /// Returns whether a cached texture exists for the given key.
    pub fn has_cache(&self, key: u64) -> bool {
        self.cache.contains_key(&key)
    }

    /// Clears per-frame state (called at the start of a new frame).
    /// Does NOT clear persistent textures.
    pub fn clear_frame(&mut self) {
        self.completed_regions.clear();
        self.composite_requests.clear();
        self.active = None;
    }

    /// Removes a cached texture by key.
    #[allow(dead_code)]
    pub fn remove(&mut self, key: u64) {
        let _ = self.cache.remove(&key);
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

fn create_texture(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    size: Size<u32>,
    key: u64,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(&format!("iced_wgpu.cached_draw.texture.{key}")),
        size: wgpu::Extent3d {
            width: size.width.max(1),
            height: size.height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}
