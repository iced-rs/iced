pub use iced_native::text::Hit;

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
    glyph_cache: GlyphCache,
}

#[ouroboros::self_referencing]
struct System {
    fonts: cosmic_text::FontSystem,

    #[borrows(fonts)]
    #[not_covariant]
    measurement_cache: RefCell<Cache<'this>>,

    #[borrows(fonts)]
    #[not_covariant]
    render_cache: Cache<'this>,
}

impl Pipeline {
    pub fn new() -> Self {
        Pipeline {
            system: Some(
                SystemBuilder {
                    fonts: cosmic_text::FontSystem::new_with_fonts(
                        [cosmic_text::fontdb::Source::Binary(Arc::new(
                            include_bytes!("../../wgpu/fonts/Iced-Icons.ttf")
                                .as_slice(),
                        ))]
                        .into_iter(),
                    ),
                    measurement_cache_builder: |_| RefCell::new(Cache::new()),
                    render_cache_builder: |_| Cache::new(),
                }
                .build(),
            ),
            glyph_cache: GlyphCache::new(),
        }
    }

    pub fn load_font(&mut self, bytes: Cow<'static, [u8]>) {
        let heads = self.system.take().unwrap().into_heads();

        let (locale, mut db) = heads.fonts.into_locale_and_db();

        db.load_font_source(cosmic_text::fontdb::Source::Binary(Arc::new(
            bytes.into_owned(),
        )));

        self.system = Some(
            SystemBuilder {
                fonts: cosmic_text::FontSystem::new_with_locale_and_db(
                    locale, db,
                ),
                measurement_cache_builder: |_| RefCell::new(Cache::new()),
                render_cache_builder: |_| Cache::new(),
            }
            .build(),
        );
    }

    pub fn draw(
        &mut self,
        content: &str,
        bounds: Rectangle,
        color: Color,
        size: f32,
        font: Font,
        horizontal_alignment: alignment::Horizontal,
        vertical_alignment: alignment::Vertical,
        pixels: &mut tiny_skia::PixmapMut<'_>,
        clip_mask: Option<&tiny_skia::ClipMask>,
    ) {
        self.system.as_mut().unwrap().with_mut(|fields| {
            let key = Key {
                bounds: bounds.size(),
                content,
                font,
                size,
            };

            let (_, buffer) = fields.render_cache.allocate(fields.fonts, key);

            let (total_lines, max_width) = buffer
                .layout_runs()
                .enumerate()
                .fold((0, 0.0), |(_, max), (i, buffer)| {
                    (i + 1, buffer.line_w.max(max))
                });

            let total_height = total_lines as f32 * size * 1.2;

            let x = match horizontal_alignment {
                alignment::Horizontal::Left => bounds.x,
                alignment::Horizontal::Center => bounds.x - max_width / 2.0,
                alignment::Horizontal::Right => bounds.x - max_width,
            };

            let y = match vertical_alignment {
                alignment::Vertical::Top => bounds.y,
                alignment::Vertical::Center => bounds.y - total_height / 2.0,
                alignment::Vertical::Bottom => bounds.y - total_height,
            };

            let mut swash = cosmic_text::SwashCache::new(fields.fonts);

            for run in buffer.layout_runs() {
                for glyph in run.glyphs {
                    if let Some((buffer, placement)) = self
                        .glyph_cache
                        .allocate(glyph.cache_key, color, &mut swash)
                    {
                        let pixmap = tiny_skia::PixmapRef::from_bytes(
                            buffer,
                            placement.width,
                            placement.height,
                        )
                        .expect("Create glyph pixel map");

                        pixels.draw_pixmap(
                            x as i32 + glyph.x_int + placement.left,
                            y as i32 - glyph.y_int - placement.top
                                + run.line_y as i32,
                            pixmap,
                            &tiny_skia::PixmapPaint::default(),
                            tiny_skia::Transform::identity(),
                            clip_mask,
                        );
                    }
                }
            }
        });
    }

    pub fn end_frame(&mut self) {
        self.system
            .as_mut()
            .unwrap()
            .with_render_cache_mut(|cache| cache.trim());

        self.glyph_cache.trim();
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

fn to_family(font: Font) -> cosmic_text::Family<'static> {
    match font {
        Font::Name(name) => cosmic_text::Family::Name(name),
        Font::SansSerif => cosmic_text::Family::SansSerif,
        Font::Serif => cosmic_text::Family::Serif,
        Font::Cursive => cosmic_text::Family::Cursive,
        Font::Fantasy => cosmic_text::Family::Fantasy,
        Font::Monospace => cosmic_text::Family::Monospace,
    }
}

#[derive(Debug, Clone, Default)]
struct GlyphCache {
    entries: FxHashMap<
        (cosmic_text::CacheKey, [u8; 3]),
        (Vec<u32>, cosmic_text::Placement),
    >,
    recently_used: FxHashSet<(cosmic_text::CacheKey, [u8; 3])>,
    trim_count: usize,
}

impl GlyphCache {
    fn new() -> Self {
        GlyphCache::default()
    }

    fn allocate(
        &mut self,
        cache_key: cosmic_text::CacheKey,
        color: Color,
        swash: &mut cosmic_text::SwashCache<'_>,
    ) -> Option<(&[u8], cosmic_text::Placement)> {
        let [r, g, b, _a] = color.into_rgba8();
        let key = (cache_key, [r, g, b]);

        if let hash_map::Entry::Vacant(entry) = self.entries.entry(key) {
            // TODO: Outline support
            let image = swash.get_image(cache_key).as_ref()?;

            let glyph_size = image.placement.width as usize
                * image.placement.height as usize;

            if glyph_size == 0 {
                return None;
            }

            let mut buffer = vec![0u32; glyph_size];

            match image.content {
                cosmic_text::SwashContent::Mask => {
                    let mut i = 0;

                    // TODO: Blend alpha

                    for _y in 0..image.placement.height {
                        for _x in 0..image.placement.width {
                            buffer[i] = tiny_skia::ColorU8::from_rgba(
                                b,
                                g,
                                r,
                                image.data[i],
                            )
                            .premultiply()
                            .get();

                            i += 1;
                        }
                    }
                }
                cosmic_text::SwashContent::Color => {
                    let mut i = 0;

                    for _y in 0..image.placement.height {
                        for _x in 0..image.placement.width {
                            // TODO: Blend alpha
                            buffer[i >> 2] = tiny_skia::ColorU8::from_rgba(
                                image.data[i + 2],
                                image.data[i + 1],
                                image.data[i],
                                image.data[i + 3],
                            )
                            .premultiply()
                            .get();

                            i += 4;
                        }
                    }
                }
                cosmic_text::SwashContent::SubpixelMask => {
                    // TODO
                }
            }

            entry.insert((buffer, image.placement));
        }

        self.recently_used.insert(key);

        self.entries.get(&key).map(|(buffer, placement)| {
            (bytemuck::cast_slice(buffer.as_slice()), *placement)
        })
    }

    pub fn trim(&mut self) {
        if self.trim_count > 300 {
            self.entries
                .retain(|key, _| self.recently_used.contains(key));

            self.recently_used.clear();

            self.trim_count = 0;
        } else {
            self.trim_count += 1;
        }
    }
}

struct Cache<'a> {
    entries: FxHashMap<KeyHash, cosmic_text::Buffer<'a>>,
    recently_used: FxHashSet<KeyHash>,
    hasher: HashBuilder,
    trim_count: usize,
}

#[cfg(not(target_arch = "wasm32"))]
type HashBuilder = twox_hash::RandomXxHashBuilder64;

#[cfg(target_arch = "wasm32")]
type HashBuilder = std::hash::BuildHasherDefault<twox_hash::XxHash64>;

impl<'a> Cache<'a> {
    const TRIM_INTERVAL: usize = 300;

    fn new() -> Self {
        Self {
            entries: FxHashMap::default(),
            recently_used: FxHashSet::default(),
            hasher: HashBuilder::default(),
            trim_count: 0,
        }
    }

    fn allocate(
        &mut self,
        fonts: &'a cosmic_text::FontSystem,
        key: Key<'_>,
    ) -> (KeyHash, &mut cosmic_text::Buffer<'a>) {
        let hash = {
            let mut hasher = self.hasher.build_hasher();

            key.content.hash(&mut hasher);
            key.size.to_bits().hash(&mut hasher);
            key.font.hash(&mut hasher);
            key.bounds.width.to_bits().hash(&mut hasher);
            key.bounds.height.to_bits().hash(&mut hasher);

            hasher.finish()
        };

        if let hash_map::Entry::Vacant(entry) = self.entries.entry(hash) {
            let metrics = cosmic_text::Metrics::new(key.size, key.size * 1.2);
            let mut buffer = cosmic_text::Buffer::new(fonts, metrics);

            buffer.set_size(
                key.bounds.width,
                key.bounds.height.max(key.size * 1.2),
            );
            buffer.set_text(
                key.content,
                cosmic_text::Attrs::new()
                    .family(to_family(key.font))
                    .monospaced(matches!(key.font, Font::Monospace)),
            );

            let _ = entry.insert(buffer);
        }

        let _ = self.recently_used.insert(hash);

        (hash, self.entries.get_mut(&hash).unwrap())
    }

    fn trim(&mut self) {
        if self.trim_count >= Self::TRIM_INTERVAL {
            self.entries
                .retain(|key, _| self.recently_used.contains(key));

            self.recently_used.clear();

            self.trim_count = 0;
        } else {
            self.trim_count += 1;
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Key<'a> {
    content: &'a str,
    size: f32,
    font: Font,
    bounds: Size,
}

type KeyHash = u64;
