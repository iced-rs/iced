use crate::core::alignment;
use crate::core::text::Alignment;
use crate::core::{Rectangle, Size, Transformation};
use crate::graphics::cache;
use crate::graphics::color;
use crate::graphics::text::cache::{self as text_cache, Cache as BufferCache};
use crate::graphics::text::{Editor, Paragraph, font_system, to_color};

use rustc_hash::FxHashMap;
use std::collections::hash_map;
use std::sync::atomic::{self, AtomicU64};
use std::sync::{self, Arc, RwLock};

pub use crate::graphics::Text;

const COLOR_MODE: cryoglyph::ColorMode = if color::GAMMA_CORRECTION {
    cryoglyph::ColorMode::Accurate
} else {
    cryoglyph::ColorMode::Web
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
    text: Arc<[Text]>,
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
            text: Arc::from(text),
            version: 0,
        })
    }

    pub fn update(&mut self, text: Vec<Text>) {
        if self.text.is_empty() && text.is_empty() {
            return;
        }

        self.text = Arc::from(text);
        self.version += 1;
    }
}

struct Upload {
    renderer: cryoglyph::TextRenderer,
    buffer_cache: BufferCache,
    transformation: Transformation,
    version: usize,
    group_version: usize,
    text: sync::Weak<[Text]>,
    _atlas: sync::Weak<()>,
}

#[derive(Default)]
pub struct Storage {
    groups: FxHashMap<cache::Group, Group>,
    uploads: FxHashMap<Id, Upload>,
}

struct Group {
    atlas: cryoglyph::TextAtlas,
    version: usize,
    should_trim: bool,
    handle: Arc<()>, // Keeps track of active uploads
}

impl Storage {
    fn get(&self, cache: &Cache) -> Option<(&cryoglyph::TextAtlas, &Upload)> {
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
        viewport: &cryoglyph::Viewport,
        encoder: &mut wgpu::CommandEncoder,
        format: wgpu::TextureFormat,
        state: &cryoglyph::Cache,
        cache: &Cache,
        new_transformation: Transformation,
        bounds: Rectangle,
    ) {
        let group_count = self.groups.len();

        let group = self.groups.entry(cache.group).or_insert_with(|| {
            log::debug!(
                "New text atlas: {:?} (total: {})",
                cache.group,
                group_count + 1
            );

            Group {
                atlas: cryoglyph::TextAtlas::with_color_mode(
                    device, queue, state, format, COLOR_MODE,
                ),
                version: 0,
                should_trim: false,
                handle: Arc::new(()),
            }
        });

        match self.uploads.entry(cache.id) {
            hash_map::Entry::Occupied(entry) => {
                let upload = entry.into_mut();

                if upload.version != cache.version
                    || upload.group_version != group.version
                    || upload.transformation != new_transformation
                {
                    if !cache.text.is_empty() {
                        let _ = prepare(
                            device,
                            queue,
                            viewport,
                            encoder,
                            &mut upload.renderer,
                            &mut group.atlas,
                            &mut upload.buffer_cache,
                            &cache.text,
                            bounds,
                            new_transformation,
                        );
                    }

                    // Only trim if glyphs have changed
                    group.should_trim =
                        group.should_trim || upload.version != cache.version;

                    upload.text = Arc::downgrade(&cache.text);
                    upload.version = cache.version;
                    upload.group_version = group.version;
                    upload.transformation = new_transformation;

                    upload.buffer_cache.trim();
                }
            }
            hash_map::Entry::Vacant(entry) => {
                let mut renderer = cryoglyph::TextRenderer::new(
                    &mut group.atlas,
                    device,
                    wgpu::MultisampleState::default(),
                    None,
                );

                let mut buffer_cache = BufferCache::new();

                if !cache.text.is_empty() {
                    let _ = prepare(
                        device,
                        queue,
                        viewport,
                        encoder,
                        &mut renderer,
                        &mut group.atlas,
                        &mut buffer_cache,
                        &cache.text,
                        bounds,
                        new_transformation,
                    );
                }

                let _ = entry.insert(Upload {
                    renderer,
                    buffer_cache,
                    transformation: new_transformation,
                    version: 0,
                    group_version: group.version,
                    text: Arc::downgrade(&cache.text),
                    _atlas: Arc::downgrade(&group.handle),
                });

                group.should_trim = cache.group.is_singleton();

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
            let active_uploads = Arc::weak_count(&group.handle);

            if active_uploads == 0 {
                log::debug!("Dropping text atlas: {id:?}");

                return false;
            }

            if group.should_trim {
                log::trace!("Trimming text atlas: {id:?}");

                group.atlas.trim();
                group.should_trim = false;

                // We only need to worry about glyph fighting
                // when the atlas may be shared by multiple
                // uploads.
                if !id.is_singleton() {
                    log::debug!(
                        "Invalidating text atlas: {id:?} \
                        (uploads: {active_uploads})"
                    );

                    group.version += 1;
                }
            }

            true
        });
    }
}

pub struct Viewport(cryoglyph::Viewport);

impl Viewport {
    pub fn update(&mut self, queue: &wgpu::Queue, resolution: Size<u32>) {
        self.0.update(
            queue,
            cryoglyph::Resolution {
                width: resolution.width,
                height: resolution.height,
            },
        );
    }
}

#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct Pipeline {
    format: wgpu::TextureFormat,
    cache: cryoglyph::Cache,
    atlas: Arc<RwLock<cryoglyph::TextAtlas>>,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self {
        let cache = cryoglyph::Cache::new(device);
        let atlas = cryoglyph::TextAtlas::with_color_mode(
            device, queue, &cache, format, COLOR_MODE,
        );

        Pipeline {
            format,
            cache,
            atlas: Arc::new(RwLock::new(atlas)),
        }
    }

    pub fn create_viewport(&self, device: &wgpu::Device) -> Viewport {
        Viewport(cryoglyph::Viewport::new(device, &self.cache))
    }

    pub fn trim(&self) {
        self.atlas.write().expect("Write text atlas").trim();
    }
}

#[derive(Default)]
pub struct State {
    renderers: Vec<cryoglyph::TextRenderer>,
    prepare_layer: usize,
    cache: BufferCache,
    storage: Storage,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn prepare(
        &mut self,
        pipeline: &Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        viewport: &Viewport,
        encoder: &mut wgpu::CommandEncoder,
        batch: &Batch,
        layer_bounds: Rectangle,
        layer_transformation: Transformation,
    ) {
        let mut atlas = pipeline.atlas.write().expect("Write to text atlas");

        for item in batch {
            match item {
                Item::Group {
                    transformation,
                    text,
                } => {
                    if self.renderers.len() <= self.prepare_layer {
                        self.renderers.push(cryoglyph::TextRenderer::new(
                            &mut atlas,
                            device,
                            wgpu::MultisampleState::default(),
                            None,
                        ));
                    }

                    let renderer = &mut self.renderers[self.prepare_layer];
                    let result = prepare(
                        device,
                        queue,
                        &viewport.0,
                        encoder,
                        renderer,
                        &mut atlas,
                        &mut self.cache,
                        text,
                        layer_bounds * layer_transformation,
                        layer_transformation * *transformation,
                    );

                    match result {
                        Ok(()) => {
                            self.prepare_layer += 1;
                        }
                        Err(cryoglyph::PrepareError::AtlasFull) => {
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
                    self.storage.prepare(
                        device,
                        queue,
                        &viewport.0,
                        encoder,
                        pipeline.format,
                        &pipeline.cache,
                        cache,
                        layer_transformation * *transformation,
                        layer_bounds * layer_transformation,
                    );
                }
            }
        }
    }

    pub fn render<'a>(
        &'a self,
        pipeline: &'a Pipeline,
        viewport: &'a Viewport,
        start: usize,
        batch: &'a Batch,
        bounds: Rectangle<u32>,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) -> usize {
        let atlas = pipeline.atlas.read().expect("Read text atlas");
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
                        .render(&atlas, &viewport.0, render_pass)
                        .expect("Render text");

                    layer_count += 1;
                }
                Item::Cached { cache, .. } => {
                    if let Some((atlas, upload)) = self.storage.get(cache) {
                        upload
                            .renderer
                            .render(atlas, &viewport.0, render_pass)
                            .expect("Render cached text");
                    }
                }
            }
        }

        layer_count
    }

    pub fn trim(&mut self) {
        self.cache.trim();
        self.storage.trim();

        self.prepare_layer = 0;
    }
}

fn prepare(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    viewport: &cryoglyph::Viewport,
    encoder: &mut wgpu::CommandEncoder,
    renderer: &mut cryoglyph::TextRenderer,
    atlas: &mut cryoglyph::TextAtlas,
    buffer_cache: &mut BufferCache,
    sections: &[Text],
    layer_bounds: Rectangle,
    layer_transformation: Transformation,
) -> Result<(), cryoglyph::PrepareError> {
    let mut font_system = font_system().write().expect("Write font system");
    let font_system = font_system.raw();

    enum Allocation {
        Paragraph(Paragraph),
        Editor(Editor),
        Cache(text_cache::KeyHash),
        Raw(Arc<cryoglyph::Buffer>),
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
                align_x,
                ..
            } => {
                let (key, _) = buffer_cache.allocate(
                    font_system,
                    text_cache::Key {
                        content,
                        size: f32::from(*size),
                        line_height: f32::from(*line_height),
                        font: *font,
                        align_x: *align_x,
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
            let (buffer, position, color, clip_bounds, transformation) =
                match section {
                    Text::Paragraph {
                        position,
                        color,
                        clip_bounds,
                        transformation,
                        ..
                    } => {
                        let Some(Allocation::Paragraph(paragraph)) = allocation
                        else {
                            return None;
                        };

                        (
                            paragraph.buffer(),
                            *position,
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
                        let Some(Allocation::Editor(editor)) = allocation
                        else {
                            return None;
                        };

                        (
                            editor.buffer(),
                            *position,
                            *color,
                            *clip_bounds,
                            *transformation,
                        )
                    }
                    Text::Cached {
                        bounds,
                        align_x,
                        align_y,
                        color,
                        clip_bounds,
                        ..
                    } => {
                        let Some(Allocation::Cache(key)) = allocation else {
                            return None;
                        };

                        let entry =
                            buffer_cache.get(key).expect("Get cached buffer");

                        let mut position = bounds.position();

                        position.x = match align_x {
                            Alignment::Default
                            | Alignment::Left
                            | Alignment::Justified => position.x,
                            Alignment::Center => {
                                position.x - entry.min_bounds.width / 2.0
                            }
                            Alignment::Right => {
                                position.x - entry.min_bounds.width
                            }
                        };

                        position.y = match align_y {
                            alignment::Vertical::Top => position.y,
                            alignment::Vertical::Center => {
                                position.y - entry.min_bounds.height / 2.0
                            }
                            alignment::Vertical::Bottom => {
                                position.y - entry.min_bounds.height
                            }
                        };

                        (
                            &entry.buffer,
                            position,
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

                        (
                            buffer.as_ref(),
                            raw.position,
                            raw.color,
                            raw.clip_bounds,
                            *transformation,
                        )
                    }
                };

            let position = position * transformation * layer_transformation;

            let clip_bounds = layer_bounds.intersection(
                &(clip_bounds * transformation * layer_transformation),
            )?;

            Some(cryoglyph::TextArea {
                buffer,
                left: position.x,
                top: position.y,
                scale: transformation.scale_factor()
                    * layer_transformation.scale_factor(),
                bounds: cryoglyph::TextBounds {
                    left: clip_bounds.x.round() as i32,
                    top: clip_bounds.y.round() as i32,
                    right: (clip_bounds.x + clip_bounds.width).round() as i32,
                    bottom: (clip_bounds.y + clip_bounds.height).round() as i32,
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
        viewport,
        text_areas,
        &mut cryoglyph::SwashCache::new(),
    )
}
