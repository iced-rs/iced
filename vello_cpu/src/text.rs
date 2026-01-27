use crate::core::alignment;
use crate::core::text::{Alignment, Shaping};
use crate::core::{Color, Font, Pixels, Point, Rectangle, Transformation};
use crate::graphics::text::cache::{self, Cache};
use crate::graphics::text::cosmic_text;
use crate::graphics::text::editor;
use crate::graphics::text::font_system;
use crate::graphics::text::paragraph;

use rustc_hash::{FxHashMap, FxHashSet};
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::hash_map;
use std::sync::Arc;

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
        renderer: &mut vello_cpu::RenderContext,
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
            renderer,
            transformation,
        );
    }

    pub fn draw_editor(
        &mut self,
        editor: &editor::Weak,
        position: Point,
        color: Color,
        renderer: &mut vello_cpu::RenderContext,
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
            renderer,
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
        renderer: &mut vello_cpu::RenderContext,
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
            align_x,
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
            renderer,
            transformation,
        );
    }

    pub fn draw_raw(
        &mut self,
        buffer: &cosmic_text::Buffer,
        position: Point,
        color: Color,
        renderer: &mut vello_cpu::RenderContext,
        transformation: Transformation,
    ) {
        let mut font_system = font_system().write().expect("Write font system");

        draw(
            font_system.raw(),
            &mut self.glyph_cache,
            buffer,
            position,
            color,
            renderer,
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
    renderer: &mut vello_cpu::RenderContext,
    transformation: Transformation,
) {
    let position = position * transformation;

    let mut swash = cosmic_text::SwashCache::new();

    for run in buffer.layout_runs() {
        for glyph in run.glyphs {
            let physical_glyph =
                glyph.physical((position.x, position.y), transformation.scale_factor());

            if let Some((pixmap, placement)) = glyph_cache.allocate(
                physical_glyph.cache_key,
                glyph.color_opt.map(from_color).unwrap_or(color),
                font_system,
                &mut swash,
            ) {
                // TODO
                let _opacity =
                    color.a * glyph.color_opt.map(|c| c.a() as f32 / 255.0).unwrap_or(1.0);

                renderer.set_paint(vello_cpu::peniko::Brush::Image(
                    vello_cpu::peniko::ImageBrush {
                        image: vello_cpu::ImageSource::Pixmap(pixmap.clone()),
                        sampler: vello_cpu::peniko::ImageSampler::new()
                            // .with_alpha(opacity) // TODO: Uncomment once vello_cpu supports it
                            .with_quality(vello_cpu::peniko::ImageQuality::Low),
                    },
                ));

                let position = Point {
                    x: physical_glyph.x as f32 + placement.left as f32,
                    y: physical_glyph.y as f32 - placement.top as f32
                        + run.line_y * transformation.scale_factor(),
                };

                renderer.set_transform(vello_cpu::kurbo::Affine::translate((
                    f64::from(position.x),
                    f64::from(position.y),
                )));

                renderer.fill_rect(&crate::into_rect(Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: f32::from(pixmap.width()),
                    height: f32::from(pixmap.height()),
                }));

                renderer.reset_transform();
            }
        }
    }
}

fn from_color(color: cosmic_text::Color) -> Color {
    let [r, g, b, a] = color.as_rgba();

    Color::from_rgba8(r, g, b, a as f32 / 255.0)
}

#[derive(Debug, Clone, Default)]
struct GlyphCache {
    entries: FxHashMap<
        (cosmic_text::CacheKey, [u8; 3]),
        (Arc<vello_cpu::Pixmap>, cosmic_text::Placement),
    >,
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
    ) -> Option<(&Arc<vello_cpu::Pixmap>, cosmic_text::Placement)> {
        let color = crate::into_color(color).premultiply();
        let vello_cpu::color::PremulRgba8 { r, g, b, .. } = color.to_rgba8();
        let key = (cache_key, [r, g, b]);

        if let hash_map::Entry::Vacant(entry) = self.entries.entry(key) {
            // TODO: Outline support
            let image = swash.get_image_uncached(font_system, cache_key)?;

            let width = image.placement.width as u16;
            let height = image.placement.height as u16;

            if width == 0 || height == 0 {
                return None;
            }

            let mut buffer = vello_cpu::Pixmap::new(width, height);

            match image.content {
                cosmic_text::SwashContent::Mask => {
                    // TODO: Blend alpha
                    let mut i = 0;

                    for y in 0..height {
                        for x in 0..width {
                            buffer.set_pixel(
                                x,
                                y,
                                color
                                    .multiply_alpha(f32::from(image.data[i]) / 255.0)
                                    .to_rgba8(),
                            );

                            i += 1;
                        }
                    }
                }
                cosmic_text::SwashContent::Color => {
                    let mut i = 0;

                    for y in 0..height {
                        for x in 0..width {
                            // TODO: Blend alpha
                            buffer.set_pixel(
                                x,
                                y,
                                vello_cpu::color::AlphaColor::from_rgba8(
                                    image.data[i + 2],
                                    image.data[i + 1],
                                    image.data[i],
                                    image.data[i + 3],
                                )
                                .premultiply()
                                .to_rgba8(),
                            );

                            i += 4;
                        }
                    }
                }
                cosmic_text::SwashContent::SubpixelMask => {
                    // TODO
                }
            }

            let _ = entry.insert((Arc::new(buffer), image.placement));
        }

        let _ = self.recently_used.insert(key);

        self.entries
            .get(&key)
            .map(|(buffer, placement)| (buffer, *placement))
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
