use crate::core::alignment;
use crate::core::font::{self, Font};
use crate::core::text::{Hit, LineHeight, Shaping};
use crate::core::{Color, Pixels, Point, Rectangle, Size};

use rustc_hash::{FxHashMap, FxHashSet};
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::hash_map;
use std::hash::{BuildHasher, Hash, Hasher};
use std::sync::Arc;

#[allow(missing_debug_implementations)]
pub struct Pipeline {
    font_system: RefCell<cosmic_text::FontSystem>,
    glyph_cache: GlyphCache,
    measurement_cache: RefCell<Cache>,
    render_cache: Cache,
}

impl Pipeline {
    pub fn new() -> Self {
        Pipeline {
            font_system: RefCell::new(cosmic_text::FontSystem::new_with_fonts(
                [cosmic_text::fontdb::Source::Binary(Arc::new(
                    include_bytes!("../../wgpu/fonts/Iced-Icons.ttf")
                        .as_slice(),
                ))]
                .into_iter(),
            )),
            glyph_cache: GlyphCache::new(),
            measurement_cache: RefCell::new(Cache::new()),
            render_cache: Cache::new(),
        }
    }

    pub fn load_font(&mut self, bytes: Cow<'static, [u8]>) {
        self.font_system.get_mut().db_mut().load_font_source(
            cosmic_text::fontdb::Source::Binary(Arc::new(bytes.into_owned())),
        );
    }

    pub fn draw(
        &mut self,
        content: &str,
        bounds: Rectangle,
        color: Color,
        size: f32,
        line_height: LineHeight,
        font: Font,
        horizontal_alignment: alignment::Horizontal,
        vertical_alignment: alignment::Vertical,
        shaping: Shaping,
        scale_factor: f32,
        pixels: &mut tiny_skia::PixmapMut<'_>,
        clip_mask: Option<&tiny_skia::Mask>,
    ) {
        let line_height =
            f32::from(line_height.to_absolute(Pixels(size))) * scale_factor;

        let bounds = bounds * scale_factor;
        let size = size * scale_factor;

        let font_system = self.font_system.get_mut();
        let key = Key {
            bounds: {
                let size = bounds.size();

                // TODO: Reuse buffers from layouting
                Size::new(size.width.ceil(), size.height.ceil())
            },
            content,
            font,
            size,
            line_height,
            shaping,
        };

        let (_, buffer) = self.render_cache.allocate(font_system, key);

        let (total_lines, max_width) = buffer
            .layout_runs()
            .enumerate()
            .fold((0, 0.0), |(_, max), (i, buffer)| {
                (i + 1, buffer.line_w.max(max))
            });

        let total_height = total_lines as f32 * line_height;

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

        // TODO: Subpixel glyph positioning
        let x = x.round() as i32;
        let y = y.round() as i32;

        let mut swash = cosmic_text::SwashCache::new();

        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                if let Some((buffer, placement)) = self.glyph_cache.allocate(
                    glyph.cache_key,
                    color,
                    font_system,
                    &mut swash,
                ) {
                    let pixmap = tiny_skia::PixmapRef::from_bytes(
                        buffer,
                        placement.width,
                        placement.height,
                    )
                    .expect("Create glyph pixel map");

                    pixels.draw_pixmap(
                        x + glyph.x_int + placement.left,
                        y - glyph.y_int - placement.top + run.line_y as i32,
                        pixmap,
                        &tiny_skia::PixmapPaint::default(),
                        tiny_skia::Transform::identity(),
                        clip_mask,
                    );
                }
            }
        }
    }

    pub fn trim_cache(&mut self) {
        self.render_cache.trim();
        self.glyph_cache.trim();
    }

    pub fn measure(
        &self,
        content: &str,
        size: f32,
        line_height: LineHeight,
        font: Font,
        bounds: Size,
        shaping: Shaping,
    ) -> (f32, f32) {
        let mut measurement_cache = self.measurement_cache.borrow_mut();

        let line_height = f32::from(line_height.to_absolute(Pixels(size)));

        let (_, paragraph) = measurement_cache.allocate(
            &mut self.font_system.borrow_mut(),
            Key {
                content,
                size,
                line_height,
                font,
                bounds,
                shaping,
            },
        );

        let (total_lines, max_width) = paragraph
            .layout_runs()
            .enumerate()
            .fold((0, 0.0), |(_, max), (i, buffer)| {
                (i + 1, buffer.line_w.max(max))
            });

        (max_width, line_height * total_lines as f32)
    }

    pub fn hit_test(
        &self,
        content: &str,
        size: f32,
        line_height: LineHeight,
        font: Font,
        bounds: Size,
        shaping: Shaping,
        point: Point,
        _nearest_only: bool,
    ) -> Option<Hit> {
        let mut measurement_cache = self.measurement_cache.borrow_mut();

        let line_height = f32::from(line_height.to_absolute(Pixels(size)));

        let (_, paragraph) = measurement_cache.allocate(
            &mut self.font_system.borrow_mut(),
            Key {
                content,
                size,
                line_height,
                font,
                bounds,
                shaping,
            },
        );

        let cursor = paragraph.hit(point.x, point.y)?;

        Some(Hit::CharOffset(cursor.index))
    }

    pub fn trim_measurement_cache(&mut self) {
        self.measurement_cache.borrow_mut().trim();
    }
}

fn to_family(family: font::Family) -> cosmic_text::Family<'static> {
    match family {
        font::Family::Name(name) => cosmic_text::Family::Name(name),
        font::Family::SansSerif => cosmic_text::Family::SansSerif,
        font::Family::Serif => cosmic_text::Family::Serif,
        font::Family::Cursive => cosmic_text::Family::Cursive,
        font::Family::Fantasy => cosmic_text::Family::Fantasy,
        font::Family::Monospace => cosmic_text::Family::Monospace,
    }
}

fn to_weight(weight: font::Weight) -> cosmic_text::Weight {
    match weight {
        font::Weight::Thin => cosmic_text::Weight::THIN,
        font::Weight::ExtraLight => cosmic_text::Weight::EXTRA_LIGHT,
        font::Weight::Light => cosmic_text::Weight::LIGHT,
        font::Weight::Normal => cosmic_text::Weight::NORMAL,
        font::Weight::Medium => cosmic_text::Weight::MEDIUM,
        font::Weight::Semibold => cosmic_text::Weight::SEMIBOLD,
        font::Weight::Bold => cosmic_text::Weight::BOLD,
        font::Weight::ExtraBold => cosmic_text::Weight::EXTRA_BOLD,
        font::Weight::Black => cosmic_text::Weight::BLACK,
    }
}

fn to_stretch(stretch: font::Stretch) -> cosmic_text::Stretch {
    match stretch {
        font::Stretch::UltraCondensed => cosmic_text::Stretch::UltraCondensed,
        font::Stretch::ExtraCondensed => cosmic_text::Stretch::ExtraCondensed,
        font::Stretch::Condensed => cosmic_text::Stretch::Condensed,
        font::Stretch::SemiCondensed => cosmic_text::Stretch::SemiCondensed,
        font::Stretch::Normal => cosmic_text::Stretch::Normal,
        font::Stretch::SemiExpanded => cosmic_text::Stretch::SemiExpanded,
        font::Stretch::Expanded => cosmic_text::Stretch::Expanded,
        font::Stretch::ExtraExpanded => cosmic_text::Stretch::ExtraExpanded,
        font::Stretch::UltraExpanded => cosmic_text::Stretch::UltraExpanded,
    }
}

fn to_shaping(shaping: Shaping) -> cosmic_text::Shaping {
    match shaping {
        Shaping::Basic => cosmic_text::Shaping::Basic,
        Shaping::Advanced => cosmic_text::Shaping::Advanced,
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
    const TRIM_INTERVAL: usize = 300;

    fn new() -> Self {
        GlyphCache::default()
    }

    fn allocate(
        &mut self,
        cache_key: cosmic_text::CacheKey,
        color: Color,
        font_system: &mut cosmic_text::FontSystem,
        swash: &mut cosmic_text::SwashCache,
    ) -> Option<(&[u8], cosmic_text::Placement)> {
        let [r, g, b, _a] = color.into_rgba8();
        let key = (cache_key, [r, g, b]);

        if let hash_map::Entry::Vacant(entry) = self.entries.entry(key) {
            // TODO: Outline support
            let image = swash.get_image_uncached(font_system, cache_key)?;

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
        if self.trim_count > Self::TRIM_INTERVAL {
            self.entries
                .retain(|key, _| self.recently_used.contains(key));

            self.recently_used.clear();

            self.trim_count = 0;
        } else {
            self.trim_count += 1;
        }
    }
}

struct Cache {
    entries: FxHashMap<KeyHash, cosmic_text::Buffer>,
    recently_used: FxHashSet<KeyHash>,
    hasher: HashBuilder,
    trim_count: usize,
}

#[cfg(not(target_arch = "wasm32"))]
type HashBuilder = twox_hash::RandomXxHashBuilder64;

#[cfg(target_arch = "wasm32")]
type HashBuilder = std::hash::BuildHasherDefault<twox_hash::XxHash64>;

impl Cache {
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
        font_system: &mut cosmic_text::FontSystem,
        key: Key<'_>,
    ) -> (KeyHash, &mut cosmic_text::Buffer) {
        let hash = {
            let mut hasher = self.hasher.build_hasher();

            key.content.hash(&mut hasher);
            key.size.to_bits().hash(&mut hasher);
            key.line_height.to_bits().hash(&mut hasher);
            key.font.hash(&mut hasher);
            key.bounds.width.to_bits().hash(&mut hasher);
            key.bounds.height.to_bits().hash(&mut hasher);
            key.shaping.hash(&mut hasher);

            hasher.finish()
        };

        if let hash_map::Entry::Vacant(entry) = self.entries.entry(hash) {
            let metrics = cosmic_text::Metrics::new(key.size, key.size * 1.2);
            let mut buffer = cosmic_text::Buffer::new(font_system, metrics);

            buffer.set_size(
                font_system,
                key.bounds.width,
                key.bounds.height.max(key.size * 1.2),
            );
            buffer.set_text(
                font_system,
                key.content,
                cosmic_text::Attrs::new()
                    .family(to_family(key.font.family))
                    .weight(to_weight(key.font.weight))
                    .stretch(to_stretch(key.font.stretch)),
                to_shaping(key.shaping),
            );

            let _ = entry.insert(buffer);
        }

        let _ = self.recently_used.insert(hash);

        (hash, self.entries.get_mut(&hash).unwrap())
    }

    fn trim(&mut self) {
        if self.trim_count > Self::TRIM_INTERVAL {
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
    line_height: f32,
    font: Font,
    bounds: Size,
    shaping: Shaping,
}

type KeyHash = u64;
