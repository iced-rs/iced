pub use iced_native::text::Hit;

use iced_graphics::layer::Text;
use iced_native::alignment;
use iced_native::{Color, Font, Rectangle, Size};

use rustc_hash::{FxHashMap, FxHashSet};
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::hash_map;
use std::hash::{BuildHasher, Hash, Hasher};
use std::sync::Arc;

#[allow(missing_debug_implementations)]
pub struct Pipeline {
    system: Option<System>,
    renderers: Vec<glyphon::TextRenderer>,
    atlas: glyphon::TextAtlas,
    prepare_layer: usize,
}

#[ouroboros::self_referencing]
struct System {
    fonts: glyphon::FontSystem,

    #[borrows(fonts)]
    #[not_covariant]
    measurement_cache: RefCell<Cache<'this>>,

    #[borrows(fonts)]
    #[not_covariant]
    render_cache: Cache<'this>,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self {
        Pipeline {
            system: Some(
                SystemBuilder {
                    fonts: glyphon::FontSystem::new_with_fonts(
                        [glyphon::fontdb::Source::Binary(Arc::new(
                            include_bytes!("../fonts/Iced-Icons.ttf")
                                .as_slice(),
                        ))]
                        .into_iter(),
                    ),
                    measurement_cache_builder: |_| RefCell::new(Cache::new()),
                    render_cache_builder: |_| Cache::new(),
                }
                .build(),
            ),
            renderers: Vec::new(),
            atlas: glyphon::TextAtlas::new(device, queue, format),
            prepare_layer: 0,
        }
    }

    pub fn load_font(&mut self, bytes: Cow<'static, [u8]>) {
        let heads = self.system.take().unwrap().into_heads();

        let (locale, mut db) = heads.fonts.into_locale_and_db();

        db.load_font_source(glyphon::fontdb::Source::Binary(Arc::new(
            bytes.into_owned(),
        )));

        self.system = Some(
            SystemBuilder {
                fonts: glyphon::FontSystem::new_with_locale_and_db(locale, db),
                measurement_cache_builder: |_| RefCell::new(Cache::new()),
                render_cache_builder: |_| Cache::new(),
            }
            .build(),
        );
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sections: &[Text<'_>],
        bounds: Rectangle,
        scale_factor: f32,
        target_size: Size<u32>,
    ) -> bool {
        self.system.as_mut().unwrap().with_mut(|fields| {
            if self.renderers.len() <= self.prepare_layer {
                self.renderers
                    .push(glyphon::TextRenderer::new(device, queue));
            }

            let renderer = &mut self.renderers[self.prepare_layer];

            let keys: Vec<_> = sections
                .iter()
                .map(|section| {
                    let (key, _) = fields.render_cache.allocate(
                        fields.fonts,
                        Key {
                            content: section.content,
                            size: section.size * scale_factor,
                            font: section.font,
                            bounds: Size {
                                width: (section.bounds.width * scale_factor)
                                    .ceil(),
                                height: (section.bounds.height * scale_factor)
                                    .ceil(),
                            },
                            color: section.color,
                        },
                    );

                    key
                })
                .collect();

            let bounds = glyphon::TextBounds {
                left: (bounds.x * scale_factor) as i32,
                top: (bounds.y * scale_factor) as i32,
                right: ((bounds.x + bounds.width) * scale_factor) as i32,
                bottom: ((bounds.y + bounds.height) * scale_factor) as i32,
            };

            let text_areas =
                sections.iter().zip(keys.iter()).map(|(section, key)| {
                    let buffer = fields
                        .render_cache
                        .get(key)
                        .expect("Get cached buffer");

                    let x = section.bounds.x * scale_factor;
                    let y = section.bounds.y * scale_factor;

                    let (total_lines, max_width) = buffer
                        .layout_runs()
                        .enumerate()
                        .fold((0, 0.0), |(_, max), (i, buffer)| {
                            (i + 1, buffer.line_w.max(max))
                        });

                    let total_height =
                        total_lines as f32 * section.size * 1.2 * scale_factor;

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
                });

            let result = renderer.prepare(
                device,
                queue,
                &mut self.atlas,
                glyphon::Resolution {
                    width: target_size.width,
                    height: target_size.height,
                },
                text_areas,
                glyphon::Color::rgb(0, 0, 0),
                &mut glyphon::SwashCache::new(fields.fonts),
            );

            match result {
                Ok(()) => {
                    self.prepare_layer += 1;

                    true
                }
                Err(glyphon::PrepareError::AtlasFull(content_type)) => {
                    self.prepare_layer = 0;

                    if self.atlas.grow(device, content_type) {
                        false
                    } else {
                        // If the atlas cannot grow, then all bets are off.
                        // Instead of panicking, we will just pray that the result
                        // will be somewhat readable...
                        true
                    }
                }
            }
        })
    }

    pub fn render<'a>(
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

    pub fn end_frame(&mut self) {
        self.atlas.trim();

        self.system
            .as_mut()
            .unwrap()
            .with_render_cache_mut(|cache| cache.trim());

        self.prepare_layer = 0;
    }

    pub fn measure(
        &self,
        content: &str,
        size: f32,
        font: Font,
        bounds: Size,
    ) -> (f32, f32) {
        self.system.as_ref().unwrap().with(|fields| {
            let mut measurement_cache = fields.measurement_cache.borrow_mut();

            let (_, paragraph) = measurement_cache.allocate(
                fields.fonts,
                Key {
                    content,
                    size,
                    font,
                    bounds,
                    color: Color::BLACK,
                },
            );

            let (total_lines, max_width) = paragraph
                .layout_runs()
                .enumerate()
                .fold((0, 0.0), |(_, max), (i, buffer)| {
                    (i + 1, buffer.line_w.max(max))
                });

            (max_width, size * 1.2 * total_lines as f32)
        })
    }

    pub fn hit_test(
        &self,
        content: &str,
        size: f32,
        font: iced_native::Font,
        bounds: iced_native::Size,
        point: iced_native::Point,
        _nearest_only: bool,
    ) -> Option<Hit> {
        self.system.as_ref().unwrap().with(|fields| {
            let mut measurement_cache = fields.measurement_cache.borrow_mut();

            let (_, paragraph) = measurement_cache.allocate(
                fields.fonts,
                Key {
                    content,
                    size,
                    font,
                    bounds,
                    color: Color::BLACK,
                },
            );

            let cursor = paragraph.hit(point.x, point.y)?;

            Some(Hit::CharOffset(cursor.index))
        })
    }

    pub fn trim_measurement_cache(&mut self) {
        self.system
            .as_mut()
            .unwrap()
            .with_measurement_cache_mut(|cache| cache.borrow_mut().trim());
    }
}

fn to_family(font: Font) -> glyphon::Family<'static> {
    match font {
        Font::Name(name) => glyphon::Family::Name(name),
        Font::SansSerif => glyphon::Family::SansSerif,
        Font::Serif => glyphon::Family::Serif,
        Font::Cursive => glyphon::Family::Cursive,
        Font::Fantasy => glyphon::Family::Fantasy,
        Font::Monospace => glyphon::Family::Monospace,
    }
}

struct Cache<'a> {
    entries: FxHashMap<KeyHash, glyphon::Buffer<'a>>,
    recently_used: FxHashSet<KeyHash>,
    hasher: HashBuilder,
}

#[cfg(not(target_arch = "wasm32"))]
type HashBuilder = twox_hash::RandomXxHashBuilder64;

#[cfg(target_arch = "wasm32")]
type HashBuilder = std::hash::BuildHasherDefault<twox_hash::XxHash64>;

impl<'a> Cache<'a> {
    fn new() -> Self {
        Self {
            entries: FxHashMap::default(),
            recently_used: FxHashSet::default(),
            hasher: HashBuilder::default(),
        }
    }

    fn get(&self, key: &KeyHash) -> Option<&glyphon::Buffer<'a>> {
        self.entries.get(key)
    }

    fn allocate(
        &mut self,
        fonts: &'a glyphon::FontSystem,
        key: Key<'_>,
    ) -> (KeyHash, &mut glyphon::Buffer<'a>) {
        let hash = {
            let mut hasher = self.hasher.build_hasher();

            key.content.hash(&mut hasher);
            key.size.to_bits().hash(&mut hasher);
            key.font.hash(&mut hasher);
            key.bounds.width.to_bits().hash(&mut hasher);
            key.bounds.height.to_bits().hash(&mut hasher);
            key.color.into_rgba8().hash(&mut hasher);

            hasher.finish()
        };

        if let hash_map::Entry::Vacant(entry) = self.entries.entry(hash) {
            let metrics = glyphon::Metrics::new(key.size, key.size * 1.2);
            let mut buffer = glyphon::Buffer::new(fonts, metrics);

            buffer.set_size(
                key.bounds.width,
                key.bounds.height.max(key.size * 1.2),
            );
            buffer.set_text(
                key.content,
                glyphon::Attrs::new()
                    .family(to_family(key.font))
                    .color({
                        let [r, g, b, a] = key.color.into_linear();

                        glyphon::Color::rgba(
                            (r * 255.0) as u8,
                            (g * 255.0) as u8,
                            (b * 255.0) as u8,
                            (a * 255.0) as u8,
                        )
                    })
                    .monospaced(matches!(key.font, Font::Monospace)),
            );

            let _ = entry.insert(buffer);
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
