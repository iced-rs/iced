pub use iced_native::text::Hit;

use iced_graphics::layer::Text;
use iced_native::alignment;
use iced_native::{Color, Font, Rectangle, Size};

use rustc_hash::{FxHashMap, FxHashSet};
use std::cell::RefCell;
use std::hash::{BuildHasher, Hash, Hasher};
use twox_hash::RandomXxHashBuilder64;

#[allow(missing_debug_implementations)]
pub struct Pipeline {
    renderers: Vec<glyphon::TextRenderer>,
    atlas: glyphon::TextAtlas,
    cache: glyphon::SwashCache<'static>,
    measurement_cache: RefCell<Cache>,
    render_cache: Cache,
    layer: usize,
}

struct Cache {
    entries: FxHashMap<KeyHash, glyphon::Buffer<'static>>,
    recently_used: FxHashSet<KeyHash>,
    hasher: RandomXxHashBuilder64,
}

impl Cache {
    fn new() -> Self {
        Self {
            entries: FxHashMap::default(),
            recently_used: FxHashSet::default(),
            hasher: RandomXxHashBuilder64::default(),
        }
    }

    fn get(&self, key: &KeyHash) -> Option<&glyphon::Buffer<'static>> {
        self.entries.get(key)
    }

    fn allocate(
        &mut self,
        key: Key<'_>,
    ) -> (KeyHash, &mut glyphon::Buffer<'static>) {
        let hash = {
            let mut hasher = self.hasher.build_hasher();

            key.content.hash(&mut hasher);
            (key.size as i32).hash(&mut hasher);
            key.font.hash(&mut hasher);
            (key.bounds.width as i32).hash(&mut hasher);
            (key.bounds.height as i32).hash(&mut hasher);
            key.color.into_rgba8().hash(&mut hasher);

            hasher.finish()
        };

        if !self.entries.contains_key(&hash) {
            let metrics =
                glyphon::Metrics::new(key.size as i32, (key.size * 1.2) as i32);

            let mut buffer = glyphon::Buffer::new(&FONT_SYSTEM, metrics);

            buffer.set_size(key.bounds.width as i32, key.bounds.height as i32);
            buffer.set_text(
                key.content,
                glyphon::Attrs::new().family(to_family(key.font)).color({
                    let [r, g, b, a] = key.color.into_linear();

                    glyphon::Color::rgba(
                        (r * 255.0) as u8,
                        (g * 255.0) as u8,
                        (b * 255.0) as u8,
                        (a * 255.0) as u8,
                    )
                }),
            );

            let _ = self.entries.insert(hash, buffer);
        }

        let _ = self.recently_used.insert(hash);

        (hash, self.entries.get_mut(&hash).unwrap())
    }

    fn trim(&mut self) {
        self.entries
            .retain(|key, _| self.recently_used.contains(key));

        self.recently_used.clear();
    }
}

#[derive(Debug, Clone, Copy)]
struct Key<'a> {
    content: &'a str,
    size: f32,
    font: Font,
    bounds: Size,
    color: Color,
}

type KeyHash = u64;

// TODO: Share with `iced_graphics`
static FONT_SYSTEM: once_cell::sync::Lazy<glyphon::FontSystem> =
    once_cell::sync::Lazy::new(glyphon::FontSystem::new);

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        _default_font: Option<&[u8]>,
        _multithreading: bool,
    ) -> Self {
        Pipeline {
            renderers: Vec::new(),
            atlas: glyphon::TextAtlas::new(device, queue, format),
            cache: glyphon::SwashCache::new(&FONT_SYSTEM),
            measurement_cache: RefCell::new(Cache::new()),
            render_cache: Cache::new(),
            layer: 0,
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sections: &[Text<'_>],
        bounds: Rectangle,
        scale_factor: f32,
        target_size: Size<u32>,
    ) {
        if self.renderers.len() <= self.layer {
            self.renderers
                .push(glyphon::TextRenderer::new(device, queue));
        }

        let renderer = &mut self.renderers[self.layer];

        let keys: Vec<_> = sections
            .iter()
            .map(|section| {
                let (key, _) = self.render_cache.allocate(Key {
                    content: section.content,
                    size: section.size * scale_factor,
                    font: section.font,
                    bounds: Size {
                        width: section.bounds.width * scale_factor,
                        height: section.bounds.height * scale_factor,
                    },
                    color: section.color,
                });

                key
            })
            .collect();

        let buffers: Vec<_> = keys
            .iter()
            .map(|key| self.render_cache.get(key).expect("Get cached buffer"))
            .collect();

        let bounds = glyphon::TextBounds {
            left: (bounds.x * scale_factor) as i32,
            top: (bounds.y * scale_factor) as i32,
            right: ((bounds.x + bounds.width) * scale_factor) as i32,
            bottom: ((bounds.y + bounds.height) * scale_factor) as i32,
        };

        let text_areas: Vec<_> = sections
            .iter()
            .zip(buffers.iter())
            .map(|(section, buffer)| {
                let x = section.bounds.x * scale_factor;
                let y = section.bounds.y * scale_factor;

                let max_width = buffer
                    .layout_runs()
                    .fold(0.0f32, |max, run| max.max(run.line_w));

                let total_height = buffer.visible_lines() as f32
                    * section.size
                    * 1.2
                    * scale_factor;

                let left = match section.horizontal_alignment {
                    alignment::Horizontal::Left => x,
                    alignment::Horizontal::Center => x - max_width / 2.0,
                    alignment::Horizontal::Right => x - max_width,
                };

                let top = match section.vertical_alignment {
                    alignment::Vertical::Top => y,
                    alignment::Vertical::Center => y - total_height / 2.0,
                    alignment::Vertical::Bottom => y - total_height,
                };

                glyphon::TextArea {
                    buffer,
                    left: left as i32,
                    top: top as i32,
                    bounds,
                }
            })
            .collect();

        renderer
            .prepare(
                device,
                queue,
                &mut self.atlas,
                glyphon::Resolution {
                    width: target_size.width,
                    height: target_size.height,
                },
                &text_areas,
                glyphon::Color::rgb(0, 0, 0),
                &mut self.cache,
            )
            .expect("Prepare text sections");
    }

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        let mut render_pass =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

        let renderer = &mut self.renderers[self.layer];

        renderer
            .render(&self.atlas, &mut render_pass)
            .expect("Render text");

        self.layer += 1;
    }

    pub fn end_frame(&mut self) {
        self.renderers.truncate(self.layer);
        self.layer = 0;
    }

    pub fn measure(
        &self,
        content: &str,
        size: f32,
        font: Font,
        bounds: Size,
    ) -> (f32, f32) {
        let mut measurement_cache = self.measurement_cache.borrow_mut();

        let (_, paragraph) = measurement_cache.allocate(Key {
            content,
            size: size,
            font,
            bounds: Size {
                width: bounds.width,
                height: f32::INFINITY,
            },
            color: Color::BLACK,
        });

        let (total_lines, max_width) = paragraph
            .layout_runs()
            .enumerate()
            .fold((0, 0.0), |(_, max), (i, buffer)| {
                (i + 1, buffer.line_w.max(max))
            });

        (max_width, size * 1.2 * total_lines as f32)
    }

    pub fn hit_test(
        &self,
        _content: &str,
        _size: f32,
        _font: iced_native::Font,
        _bounds: iced_native::Size,
        _point: iced_native::Point,
        _nearest_only: bool,
    ) -> Option<Hit> {
        None
    }

    pub fn trim_measurement_cache(&mut self) {
        self.measurement_cache.borrow_mut().trim();
    }
}

fn to_family(font: Font) -> glyphon::Family<'static> {
    match font {
        Font::Default => glyphon::Family::SansSerif,
        Font::External { name, .. } => glyphon::Family::Name(name),
    }
}
