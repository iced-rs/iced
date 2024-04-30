use crate::core::alignment;
use crate::core::{Rectangle, Size, Transformation};
use crate::graphics::cache;
use crate::graphics::color;
use crate::graphics::text::cache::{self as text_cache, Cache as BufferCache};
use crate::graphics::text::{font_system, to_color, Editor, Paragraph};

use rustc_hash::FxHashMap;
use std::collections::hash_map;
use std::rc::{self, Rc};
use std::sync::atomic::{self, AtomicU64};
use std::sync::Arc;

pub use crate::graphics::Text;

const COLOR_MODE: glyphon::ColorMode = if color::GAMMA_CORRECTION {
    glyphon::ColorMode::Accurate
} else {
    glyphon::ColorMode::Web
};

pub type Batch = Vec<Item>;

#[derive(Debug)]
pub enum Item {
    Group {
        transformation: Transformation,
        text: Vec<Text>,
    },
    Cached {
        transformation: Transformation,
        cache: Cache,
    },
}

#[derive(Debug, Clone)]
pub struct Cache {
    id: Id,
    group: cache::Group,
    text: Rc<[Text]>,
    version: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(u64);

impl Cache {
    pub fn new(group: cache::Group, text: Vec<Text>) -> Option<Self> {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        if text.is_empty() {
            return None;
        }

        Some(Self {
            id: Id(NEXT_ID.fetch_add(1, atomic::Ordering::Relaxed)),
            group,
            text: Rc::from(text),
            version: 0,
        })
    }

    pub fn update(&mut self, text: Vec<Text>) {
        self.text = Rc::from(text);
        self.version += 1;
    }
}

struct Upload {
    renderer: glyphon::TextRenderer,
    buffer_cache: BufferCache,
    transformation: Transformation,
    version: usize,
    group_version: usize,
    text: rc::Weak<[Text]>,
    _atlas: rc::Weak<()>,
}

#[derive(Default)]
pub struct Storage {
    groups: FxHashMap<cache::Group, Group>,
    uploads: FxHashMap<Id, Upload>,
}

struct Group {
    atlas: glyphon::TextAtlas,
    version: usize,
    should_trim: bool,
    handle: Rc<()>, // Keeps track of active uploads
}

impl Storage {
    pub fn new() -> Self {
        Self::default()
    }

    fn get(&self, cache: &Cache) -> Option<(&glyphon::TextAtlas, &Upload)> {
        if cache.text.is_empty() {
            return None;
        }

        self.groups
            .get(&cache.group)
            .map(|group| &group.atlas)
            .zip(self.uploads.get(&cache.id))
    }

    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        format: wgpu::TextureFormat,
        cache: &Cache,
        new_transformation: Transformation,
        bounds: Rectangle,
        target_size: Size<u32>,
    ) {
        let group_count = self.groups.len();

        let group = self.groups.entry(cache.group).or_insert_with(|| {
            log::debug!(
                "New text atlas: {:?} (total: {})",
                cache.group,
                group_count + 1
            );

            Group {
                atlas: glyphon::TextAtlas::with_color_mode(
                    device, queue, format, COLOR_MODE,
                ),
                version: 0,
                should_trim: false,
                handle: Rc::new(()),
            }
        });

        match self.uploads.entry(cache.id) {
            hash_map::Entry::Occupied(entry) => {
                let upload = entry.into_mut();

                if !cache.text.is_empty()
                    && (upload.version != cache.version
                        || upload.group_version != group.version
                        || upload.transformation != new_transformation)
                {
                    let _ = prepare(
                        device,
                        queue,
                        encoder,
                        &mut upload.renderer,
                        &mut group.atlas,
                        &mut upload.buffer_cache,
                        &cache.text,
                        bounds,
                        new_transformation,
                        target_size,
                    );

                    upload.text = Rc::downgrade(&cache.text);
                    upload.version = cache.version;
                    upload.group_version = group.version;
                    upload.transformation = new_transformation;

                    upload.buffer_cache.trim();
                    group.should_trim = true;
                }
            }
            hash_map::Entry::Vacant(entry) => {
                let mut renderer = glyphon::TextRenderer::new(
                    &mut group.atlas,
                    device,
                    wgpu::MultisampleState::default(),
                    None,
                );

                let mut buffer_cache = BufferCache::new();

                let _ = prepare(
                    device,
                    queue,
                    encoder,
                    &mut renderer,
                    &mut group.atlas,
                    &mut buffer_cache,
                    &cache.text,
                    bounds,
                    new_transformation,
                    target_size,
                );

                let _ = entry.insert(Upload {
                    renderer,
                    buffer_cache,
                    transformation: new_transformation,
                    version: 0,
                    group_version: group.version,
                    text: Rc::downgrade(&cache.text),
                    _atlas: Rc::downgrade(&group.handle),
                });

                log::debug!(
                    "New text upload: {} (total: {})",
                    cache.id.0,
                    self.uploads.len()
                );
            }
        }
    }

    pub fn trim(&mut self) {
        self.uploads
            .retain(|_id, upload| upload.text.strong_count() > 0);

        self.groups.retain(|id, group| {
            if Rc::weak_count(&group.handle) == 0 {
                log::debug!("Dropping text atlas: {id:?}");

                return false;
            }

            if group.should_trim {
                log::debug!("Trimming text atlas: {id:?}");

                group.atlas.trim();

                group.version += 1;
                group.should_trim = false;
            }

            true
        });
    }
}

#[allow(missing_debug_implementations)]
pub struct Pipeline {
    format: wgpu::TextureFormat,
    atlas: glyphon::TextAtlas,
    renderers: Vec<glyphon::TextRenderer>,
    prepare_layer: usize,
    cache: BufferCache,
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
                device, queue, format, COLOR_MODE,
            ),
            prepare_layer: 0,
            cache: BufferCache::new(),
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        storage: &mut Storage,
        batch: &Batch,
        layer_bounds: Rectangle,
        layer_transformation: Transformation,
        target_size: Size<u32>,
    ) {
        for item in batch {
            match item {
                Item::Group {
                    transformation,
                    text,
                } => {
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
                        text,
                        layer_bounds * layer_transformation,
                        layer_transformation * *transformation,
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
                Item::Cached {
                    transformation,
                    cache,
                } => {
                    storage.prepare(
                        device,
                        queue,
                        encoder,
                        self.format,
                        cache,
                        layer_transformation * *transformation,
                        layer_bounds * layer_transformation,
                        target_size,
                    );
                }
            }
        }
    }

    pub fn render<'a>(
        &'a self,
        storage: &'a Storage,
        start: usize,
        batch: &'a Batch,
        bounds: Rectangle<u32>,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) -> usize {
        let mut layer_count = 0;

        render_pass.set_scissor_rect(
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height,
        );

        for item in batch {
            match item {
                Item::Group { .. } => {
                    let renderer = &self.renderers[start + layer_count];

                    renderer
                        .render(&self.atlas, render_pass)
                        .expect("Render text");

                    layer_count += 1;
                }
                Item::Cached { cache, .. } => {
                    if let Some((atlas, upload)) = storage.get(cache) {
                        upload
                            .renderer
                            .render(atlas, render_pass)
                            .expect("Render cached text");
                    }
                }
            }
        }

        layer_count
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
    sections: &[Text],
    layer_bounds: Rectangle,
    layer_transformation: Transformation,
    target_size: Size<u32>,
) -> Result<(), glyphon::PrepareError> {
    let mut font_system = font_system().write().expect("Write font system");
    let font_system = font_system.raw();

    enum Allocation {
        Paragraph(Paragraph),
        Editor(Editor),
        Cache(text_cache::KeyHash),
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
                    text_cache::Key {
                        content,
                        size: f32::from(*size),
                        line_height: f32::from(*line_height),
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
