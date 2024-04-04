use crate::core::alignment;
use crate::core::{Rectangle, Size, Transformation};
use crate::graphics::color;
use crate::graphics::text::cache::{self, Cache as BufferCache};
use crate::graphics::text::{font_system, to_color, Editor, Paragraph};

use std::sync::Arc;

pub use crate::graphics::Text;

pub type Batch = Vec<Text>;

#[allow(missing_debug_implementations)]
pub struct Pipeline {
    format: wgpu::TextureFormat,
    atlas: glyphon::TextAtlas,
    renderers: Vec<glyphon::TextRenderer>,
    prepare_layer: usize,
    cache: BufferCache,
}

pub enum Cache {
    Staged(Batch),
    Uploaded {
        batch: Batch,
        renderer: glyphon::TextRenderer,
        atlas: Option<glyphon::TextAtlas>,
        buffer_cache: Option<BufferCache>,
        transformation: Transformation,
        target_size: Size<u32>,
        needs_reupload: bool,
    },
}

impl Cache {
    pub fn is_empty(&self) -> bool {
        match self {
            Cache::Staged(batch) | Cache::Uploaded { batch, .. } => {
                batch.is_empty()
            }
        }
    }

    pub fn update(&mut self, new_batch: Batch) {
        match self {
            Self::Staged(batch) => {
                *batch = new_batch;
            }
            Self::Uploaded {
                batch,
                needs_reupload,
                ..
            } => {
                *batch = new_batch;
                *needs_reupload = true;
            }
        }
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::Staged(Batch::default())
    }
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self {
        Pipeline {
            format,
            renderers: Vec::new(),
            atlas: glyphon::TextAtlas::with_color_mode(
                device,
                queue,
                format,
                if color::GAMMA_CORRECTION {
                    glyphon::ColorMode::Accurate
                } else {
                    glyphon::ColorMode::Web
                },
            ),
            prepare_layer: 0,
            cache: BufferCache::new(),
        }
    }

    pub fn prepare_batch(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        sections: &Batch,
        layer_bounds: Rectangle,
        layer_transformation: Transformation,
        target_size: Size<u32>,
    ) {
        if self.renderers.len() <= self.prepare_layer {
            self.renderers.push(glyphon::TextRenderer::new(
                &mut self.atlas,
                device,
                wgpu::MultisampleState::default(),
                None,
            ));
        }

        let renderer = &mut self.renderers[self.prepare_layer];
        let result = prepare(
            device,
            queue,
            encoder,
            renderer,
            &mut self.atlas,
            &mut self.cache,
            sections,
            layer_bounds,
            layer_transformation,
            target_size,
        );

        match result {
            Ok(()) => {
                self.prepare_layer += 1;
            }
            Err(glyphon::PrepareError::AtlasFull) => {
                // If the atlas cannot grow, then all bets are off.
                // Instead of panicking, we will just pray that the result
                // will be somewhat readable...
            }
        }
    }

    pub fn prepare_cache(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        cache: &mut Cache,
        layer_bounds: Rectangle,
        new_transformation: Transformation,
        new_target_size: Size<u32>,
    ) {
        match cache {
            Cache::Staged(_) => {
                let Cache::Staged(batch) =
                    std::mem::replace(cache, Cache::Staged(Batch::default()))
                else {
                    unreachable!()
                };

                // TODO: Find a better heuristic (?)
                let (mut atlas, mut buffer_cache) = if batch.len() > 10 {
                    (
                        Some(glyphon::TextAtlas::with_color_mode(
                            device,
                            queue,
                            self.format,
                            if color::GAMMA_CORRECTION {
                                glyphon::ColorMode::Accurate
                            } else {
                                glyphon::ColorMode::Web
                            },
                        )),
                        Some(BufferCache::new()),
                    )
                } else {
                    (None, None)
                };

                let mut renderer = glyphon::TextRenderer::new(
                    atlas.as_mut().unwrap_or(&mut self.atlas),
                    device,
                    wgpu::MultisampleState::default(),
                    None,
                );

                let _ = prepare(
                    device,
                    queue,
                    encoder,
                    &mut renderer,
                    atlas.as_mut().unwrap_or(&mut self.atlas),
                    buffer_cache.as_mut().unwrap_or(&mut self.cache),
                    &batch,
                    layer_bounds,
                    new_transformation,
                    new_target_size,
                );

                *cache = Cache::Uploaded {
                    batch,
                    needs_reupload: false,
                    renderer,
                    atlas,
                    buffer_cache,
                    transformation: new_transformation,
                    target_size: new_target_size,
                }
            }
            Cache::Uploaded {
                batch,
                needs_reupload,
                renderer,
                atlas,
                buffer_cache,
                transformation,
                target_size,
            } => {
                if *needs_reupload
                    || atlas.is_none()
                    || buffer_cache.is_none()
                    || new_transformation != *transformation
                    || new_target_size != *target_size
                {
                    let _ = prepare(
                        device,
                        queue,
                        encoder,
                        renderer,
                        atlas.as_mut().unwrap_or(&mut self.atlas),
                        buffer_cache.as_mut().unwrap_or(&mut self.cache),
                        batch,
                        layer_bounds,
                        new_transformation,
                        new_target_size,
                    );

                    *transformation = new_transformation;
                    *target_size = new_target_size;
                    *needs_reupload = false;
                }
            }
        }
    }

    pub fn render_batch<'a>(
        &'a self,
        layer: usize,
        bounds: Rectangle<u32>,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        let renderer = &self.renderers[layer];

        render_pass.set_scissor_rect(
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height,
        );

        renderer
            .render(&self.atlas, render_pass)
            .expect("Render text");
    }

    pub fn render_cache<'a>(
        &'a self,
        cache: &'a Cache,
        bounds: Rectangle<u32>,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        let Cache::Uploaded {
            renderer, atlas, ..
        } = cache
        else {
            return;
        };

        render_pass.set_scissor_rect(
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height,
        );

        renderer
            .render(atlas.as_ref().unwrap_or(&self.atlas), render_pass)
            .expect("Render text");
    }

    pub fn end_frame(&mut self) {
        self.atlas.trim();
        self.cache.trim();

        self.prepare_layer = 0;
    }
}

fn prepare(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    renderer: &mut glyphon::TextRenderer,
    atlas: &mut glyphon::TextAtlas,
    buffer_cache: &mut BufferCache,
    sections: &Batch,
    layer_bounds: Rectangle,
    layer_transformation: Transformation,
    target_size: Size<u32>,
) -> Result<(), glyphon::PrepareError> {
    let mut font_system = font_system().write().expect("Write font system");
    let font_system = font_system.raw();

    enum Allocation {
        Paragraph(Paragraph),
        Editor(Editor),
        Cache(cache::KeyHash),
        Raw(Arc<glyphon::Buffer>),
    }

    let allocations: Vec<_> = sections
        .iter()
        .map(|section| match section {
            Text::Paragraph { paragraph, .. } => {
                paragraph.upgrade().map(Allocation::Paragraph)
            }
            Text::Editor { editor, .. } => {
                editor.upgrade().map(Allocation::Editor)
            }
            Text::Cached {
                content,
                bounds,
                size,
                line_height,
                font,
                shaping,
                ..
            } => {
                let (key, _) = buffer_cache.allocate(
                    font_system,
                    cache::Key {
                        content,
                        size: (*size).into(),
                        line_height: f32::from(line_height.to_absolute(*size)),
                        font: *font,
                        bounds: Size {
                            width: bounds.width,
                            height: bounds.height,
                        },
                        shaping: *shaping,
                    },
                );

                Some(Allocation::Cache(key))
            }
            Text::Raw { raw, .. } => raw.buffer.upgrade().map(Allocation::Raw),
        })
        .collect();

    let layer_bounds = layer_bounds * layer_transformation;

    let text_areas = sections.iter().zip(allocations.iter()).filter_map(
        |(section, allocation)| {
            let (
                buffer,
                bounds,
                horizontal_alignment,
                vertical_alignment,
                color,
                clip_bounds,
                transformation,
            ) = match section {
                Text::Paragraph {
                    position,
                    color,
                    clip_bounds,
                    transformation,
                    ..
                } => {
                    use crate::core::text::Paragraph as _;

                    let Some(Allocation::Paragraph(paragraph)) = allocation
                    else {
                        return None;
                    };

                    (
                        paragraph.buffer(),
                        Rectangle::new(*position, paragraph.min_bounds()),
                        paragraph.horizontal_alignment(),
                        paragraph.vertical_alignment(),
                        *color,
                        *clip_bounds,
                        *transformation,
                    )
                }
                Text::Editor {
                    position,
                    color,
                    clip_bounds,
                    transformation,
                    ..
                } => {
                    use crate::core::text::Editor as _;

                    let Some(Allocation::Editor(editor)) = allocation else {
                        return None;
                    };

                    (
                        editor.buffer(),
                        Rectangle::new(*position, editor.bounds()),
                        alignment::Horizontal::Left,
                        alignment::Vertical::Top,
                        *color,
                        *clip_bounds,
                        *transformation,
                    )
                }
                Text::Cached {
                    bounds,
                    horizontal_alignment,
                    vertical_alignment,
                    color,
                    clip_bounds,
                    ..
                } => {
                    let Some(Allocation::Cache(key)) = allocation else {
                        return None;
                    };

                    let entry =
                        buffer_cache.get(key).expect("Get cached buffer");

                    (
                        &entry.buffer,
                        Rectangle::new(bounds.position(), entry.min_bounds),
                        *horizontal_alignment,
                        *vertical_alignment,
                        *color,
                        *clip_bounds,
                        Transformation::IDENTITY,
                    )
                }
                Text::Raw {
                    raw,
                    transformation,
                } => {
                    let Some(Allocation::Raw(buffer)) = allocation else {
                        return None;
                    };

                    let (width, height) = buffer.size();

                    (
                        buffer.as_ref(),
                        Rectangle::new(raw.position, Size::new(width, height)),
                        alignment::Horizontal::Left,
                        alignment::Vertical::Top,
                        raw.color,
                        raw.clip_bounds,
                        *transformation,
                    )
                }
            };

            let bounds = bounds * transformation * layer_transformation;

            let left = match horizontal_alignment {
                alignment::Horizontal::Left => bounds.x,
                alignment::Horizontal::Center => bounds.x - bounds.width / 2.0,
                alignment::Horizontal::Right => bounds.x - bounds.width,
            };

            let top = match vertical_alignment {
                alignment::Vertical::Top => bounds.y,
                alignment::Vertical::Center => bounds.y - bounds.height / 2.0,
                alignment::Vertical::Bottom => bounds.y - bounds.height,
            };

            let clip_bounds = layer_bounds.intersection(
                &(clip_bounds * transformation * layer_transformation),
            )?;

            Some(glyphon::TextArea {
                buffer,
                left,
                top,
                scale: transformation.scale_factor()
                    * layer_transformation.scale_factor(),
                bounds: glyphon::TextBounds {
                    left: clip_bounds.x as i32,
                    top: clip_bounds.y as i32,
                    right: (clip_bounds.x + clip_bounds.width) as i32,
                    bottom: (clip_bounds.y + clip_bounds.height) as i32,
                },
                default_color: to_color(color),
            })
        },
    );

    renderer.prepare(
        device,
        queue,
        encoder,
        font_system,
        atlas,
        glyphon::Resolution {
            width: target_size.width,
            height: target_size.height,
        },
        text_areas,
        &mut glyphon::SwashCache::new(),
    )
}
