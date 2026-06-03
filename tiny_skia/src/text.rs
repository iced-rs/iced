use crate::core::alignment;
use crate::core::text::{Alignment, Ellipsis, Shaping, Wrapping};
use crate::core::{Color, Font, Pixels, Point, Rectangle, Transformation};
use crate::graphics::text::cache::{self, Cache};
use crate::graphics::text::editor;
use crate::graphics::text::font_system;
use crate::graphics::text::paragraph;

use rustc_hash::{FxHashMap, FxHashSet};
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::hash_map;

#[derive(Debug)]
pub struct Pipeline {
    glyph_cache: GlyphCache,
    cache: RefCell<Cache>,
}

impl Pipeline {
    pub fn new() -> Self {
        Pipeline {
            glyph_cache: GlyphCache::new(),
            cache: RefCell::new(Cache::new()),
        }
    }

    // TODO: Shared engine
    #[allow(dead_code)]
    pub fn load_font(&mut self, bytes: Cow<'static, [u8]>) {
        font_system()
            .write()
            .expect("Write font system")
            .load_font(bytes);

        self.cache = RefCell::new(Cache::new());
    }

    pub fn draw_paragraph(
        &mut self,
        paragraph: &paragraph::Weak,
        position: Point,
        color: Color,
        pixels: &mut tiny_skia::PixmapMut<'_>,
        clip_mask: Option<&tiny_skia::Mask>,
        transformation: Transformation,
    ) {
        let Some(paragraph) = paragraph.upgrade() else {
            return;
        };

        let mut font_system = font_system().write().expect("Write font system");

        draw(
            font_system.raw(),
            &mut self.glyph_cache,
            paragraph.buffer(),
            position,
            color,
            pixels,
            clip_mask,
            transformation,
        );
    }

    pub fn draw_editor(
        &mut self,
        editor: &editor::Weak,
        position: Point,
        color: Color,
        pixels: &mut tiny_skia::PixmapMut<'_>,
        clip_mask: Option<&tiny_skia::Mask>,
        transformation: Transformation,
    ) {
        let Some(editor) = editor.upgrade() else {
            return;
        };

        let mut font_system = font_system().write().expect("Write font system");

        draw(
            font_system.raw(),
            &mut self.glyph_cache,
            editor.buffer(),
            position,
            color,
            pixels,
            clip_mask,
            transformation,
        );
    }

    pub fn draw_cached(
        &mut self,
        content: &str,
        bounds: Rectangle,
        color: Color,
        size: Pixels,
        line_height: Pixels,
        font: Font,
        align_x: Alignment,
        align_y: alignment::Vertical,
        shaping: Shaping,
        wrapping: Wrapping,
        ellipsis: Ellipsis,
        letter_spacing: Option<f32>,
        pixels: &mut tiny_skia::PixmapMut<'_>,
        clip_mask: Option<&tiny_skia::Mask>,
        transformation: Transformation,
    ) {
        let line_height = f32::from(line_height);

        let mut font_system = font_system().write().expect("Write font system");
        let font_system = font_system.raw();

        let key = cache::Key {
            bounds: bounds.size(),
            content,
            font,
            size: size.into(),
            line_height,
            shaping,
            wrapping,
            ellipsis,
            align_x,
            letter_spacing,
        };

        let (_, entry) = self.cache.get_mut().allocate(font_system, key);

        let width = entry.min_bounds.width;
        let height = entry.min_bounds.height;

        let x = match align_x {
            Alignment::Default | Alignment::Left | Alignment::Justified => bounds.x,
            Alignment::Center => bounds.x - width / 2.0,
            Alignment::Right => bounds.x - width,
        };

        let y = match align_y {
            alignment::Vertical::Top => bounds.y,
            alignment::Vertical::Center => bounds.y - height / 2.0,
            alignment::Vertical::Bottom => bounds.y - height,
        };

        draw(
            font_system,
            &mut self.glyph_cache,
            &entry.buffer,
            Point::new(x, y),
            color,
            pixels,
            clip_mask,
            transformation,
        );
    }

    pub fn draw_raw(
        &mut self,
        buffer: &cosmic_text::Buffer,
        position: Point,
        color: Color,
        pixels: &mut tiny_skia::PixmapMut<'_>,
        clip_mask: Option<&tiny_skia::Mask>,
        transformation: Transformation,
    ) {
        let mut font_system = font_system().write().expect("Write font system");

        draw(
            font_system.raw(),
            &mut self.glyph_cache,
            buffer,
            position,
            color,
            pixels,
            clip_mask,
            transformation,
        );
    }

    pub fn trim_cache(&mut self) {
        self.cache.get_mut().trim();
        self.glyph_cache.trim();
    }
}

fn draw(
    font_system: &mut cosmic_text::FontSystem,
    glyph_cache: &mut GlyphCache,
    buffer: &cosmic_text::Buffer,
    position: Point,
    color: Color,
    pixels: &mut tiny_skia::PixmapMut<'_>,
    clip_mask: Option<&tiny_skia::Mask>,
    transformation: Transformation,
) {
    let position = position * transformation;
    let scale = transformation.scale_factor();

    let mut swash = cosmic_text::SwashCache::new();

    for run in buffer.layout_runs() {
        let baseline = position.y + run.line_y * scale;
        let line_top = position.y + run.line_top * scale;
        let line_height = run.line_height * scale;

        // 1. Highlight backgrounds, drawn behind the glyphs of this run.
        for glyph in run.glyphs {
            if let Some(background) = glyph.background_opt {
                let mut fill = from_color(background);
                fill.a *= color.a;
                fill_decoration_rect(
                    pixels,
                    position.x + glyph.x * scale,
                    line_top,
                    glyph.w * scale,
                    line_height,
                    fill,
                    clip_mask,
                );
            }
        }

        // 2. Glyphs.
        for glyph in run.glyphs {
            let physical_glyph = glyph.physical((position.x, position.y), scale);

            if let Some((buffer, placement)) = glyph_cache.allocate(
                physical_glyph.cache_key,
                glyph.color_opt.map(from_color).unwrap_or(color),
                font_system,
                &mut swash,
            ) {
                let pixmap =
                    tiny_skia::PixmapRef::from_bytes(buffer, placement.width, placement.height)
                        .expect("Create glyph pixel map");

                let opacity =
                    color.a * glyph.color_opt.map(|c| c.a() as f32 / 255.0).unwrap_or(1.0);

                pixels.draw_pixmap(
                    physical_glyph.x + placement.left,
                    physical_glyph.y - placement.top + (run.line_y * scale).round() as i32,
                    pixmap,
                    &tiny_skia::PixmapPaint {
                        opacity,
                        ..tiny_skia::PixmapPaint::default()
                    },
                    tiny_skia::Transform::identity(),
                    clip_mask,
                );
            }
        }

        // 3. Underline / strikethrough, drawn on top of the glyphs.
        for glyph in run.glyphs {
            if glyph.underline_opt.is_none() && glyph.strikethrough_opt.is_none() {
                continue;
            }

            let x = position.x + glyph.x * scale;
            let w = glyph.w * scale;
            let font_size = glyph.font_size * scale;
            let thickness = (font_size * 0.06).max(1.0);

            // Decoration color: explicit decoration color, else the glyph color,
            // else the run's text color (all scaled by element opacity).
            let resolve = |decoration: cosmic_text::Decoration| -> Color {
                match decoration.color_opt.or(glyph.color_opt) {
                    Some(c) => {
                        let mut fill = from_color(c);
                        fill.a *= color.a;
                        fill
                    }
                    None => color,
                }
            };

            if let Some(underline) = glyph.underline_opt {
                fill_decoration_rect(
                    pixels,
                    x,
                    baseline + font_size * 0.1,
                    w,
                    thickness,
                    resolve(underline),
                    clip_mask,
                );
            }
            if let Some(strikethrough) = glyph.strikethrough_opt {
                fill_decoration_rect(
                    pixels,
                    x,
                    baseline - font_size * 0.3,
                    w,
                    thickness,
                    resolve(strikethrough),
                    clip_mask,
                );
            }
        }
    }
}

/// Fill a solid decoration rectangle (underline / strikethrough / highlight
/// background) in the surface's color convention.
fn fill_decoration_rect(
    pixels: &mut tiny_skia::PixmapMut<'_>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: Color,
    clip_mask: Option<&tiny_skia::Mask>,
) {
    if width <= 0.0 || height <= 0.0 || color.a <= 0.0 {
        return;
    }
    let Some(rect) = tiny_skia::Rect::from_xywh(x, y, width, height) else {
        return;
    };
    let mut paint = tiny_skia::Paint::default();
    paint.set_color(crate::engine::into_color(color));
    paint.anti_alias = false;
    pixels.fill_rect(rect, &paint, tiny_skia::Transform::identity(), clip_mask);
}

fn from_color(color: cosmic_text::Color) -> Color {
    let [r, g, b, a] = color.as_rgba();

    Color::from_rgba8(r, g, b, a as f32 / 255.0)
}

#[derive(Debug, Clone, Default)]
struct GlyphCache {
    entries: FxHashMap<(cosmic_text::CacheKey, [u8; 3]), (Vec<u32>, cosmic_text::Placement)>,
    recently_used: FxHashSet<(cosmic_text::CacheKey, [u8; 3])>,
    trim_count: usize,
}

impl GlyphCache {
    const TRIM_INTERVAL: usize = 300;
    const CAPACITY_LIMIT: usize = 16 * 1024;

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

            let glyph_size = image.placement.width as usize * image.placement.height as usize;

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
                            buffer[i] = bytemuck::cast(
                                tiny_skia::ColorU8::from_rgba(b, g, r, image.data[i]).premultiply(),
                            );

                            i += 1;
                        }
                    }
                }
                cosmic_text::SwashContent::Color => {
                    let mut i = 0;

                    for _y in 0..image.placement.height {
                        for _x in 0..image.placement.width {
                            // TODO: Blend alpha
                            buffer[i >> 2] = bytemuck::cast(
                                tiny_skia::ColorU8::from_rgba(
                                    image.data[i + 2],
                                    image.data[i + 1],
                                    image.data[i],
                                    image.data[i + 3],
                                )
                                .premultiply(),
                            );

                            i += 4;
                        }
                    }
                }
                cosmic_text::SwashContent::SubpixelMask => {
                    // TODO
                }
            }

            let _ = entry.insert((buffer, image.placement));
        }

        let _ = self.recently_used.insert(key);

        self.entries
            .get(&key)
            .map(|(buffer, placement)| (bytemuck::cast_slice(buffer.as_slice()), *placement))
    }

    pub fn trim(&mut self) {
        if self.trim_count > Self::TRIM_INTERVAL || self.recently_used.len() >= Self::CAPACITY_LIMIT
        {
            self.entries
                .retain(|key, _| self.recently_used.contains(key));

            self.recently_used.clear();

            self.entries.shrink_to(Self::CAPACITY_LIMIT);
            self.recently_used.shrink_to(Self::CAPACITY_LIMIT);

            self.trim_count = 0;
        } else {
            self.trim_count += 1;
        }
    }
}
