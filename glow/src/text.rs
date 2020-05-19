mod font;

use crate::Transformation;

use std::{cell::RefCell, collections::HashMap};

pub const BUILTIN_ICONS: iced_native::Font = iced_native::Font::External {
    name: "iced_glow icons",
    bytes: include_bytes!("text/icons.ttf"),
};

pub const CHECKMARK_ICON: char = '\u{F00C}';

const FALLBACK_FONT: &[u8] =
    include_bytes!("../../wgpu/fonts/Lato-Regular.ttf");

#[derive(Debug)]
pub struct Pipeline {
    draw_brush: RefCell<glow_glyph::GlyphBrush<'static>>,
    draw_font_map: RefCell<HashMap<String, glow_glyph::FontId>>,

    measure_brush: RefCell<glyph_brush::GlyphBrush<'static, ()>>,
}

impl Pipeline {
    pub fn new(gl: &glow::Context, default_font: Option<&[u8]>) -> Self {
        // TODO: Font customization
        let font_source = font::Source::new();

        let default_font =
            default_font.map(|slice| slice.to_vec()).unwrap_or_else(|| {
                font_source
                    .load(&[font::Family::SansSerif, font::Family::Serif])
                    .unwrap_or_else(|_| FALLBACK_FONT.to_vec())
            });

        let load_glyph_brush = |font: Vec<u8>| {
            let builder =
                glow_glyph::GlyphBrushBuilder::using_fonts_bytes(vec![
                    font.clone()
                ])?;

            Ok((
                builder,
                glyph_brush::GlyphBrushBuilder::using_font_bytes(font).build(),
            ))
        };

        let (brush_builder, measure_brush) = load_glyph_brush(default_font)
            .unwrap_or_else(|_: glow_glyph::rusttype::Error| {
                log::warn!("System font failed to load. Falling back to embedded font...");

                load_glyph_brush(FALLBACK_FONT.to_vec()).expect("Load fallback font")
            });

        let draw_brush =
            brush_builder.initial_cache_size((2048, 2048)).build(gl);

        Pipeline {
            draw_brush: RefCell::new(draw_brush),
            draw_font_map: RefCell::new(HashMap::new()),

            measure_brush: RefCell::new(measure_brush),
        }
    }

    pub fn overlay_font(&self) -> glow_glyph::FontId {
        glow_glyph::FontId(0)
    }

    pub fn queue(&mut self, section: glow_glyph::Section<'_>) {
        self.draw_brush.borrow_mut().queue(section);
    }

    pub fn draw_queued(
        &mut self,
        gl: &glow::Context,
        transformation: Transformation,
        region: glow_glyph::Region,
    ) {
        self.draw_brush
            .borrow_mut()
            .draw_queued_with_transform_and_scissoring(
                gl,
                transformation.into(),
                region,
            )
            .expect("Draw text");
    }

    pub fn measure(
        &self,
        content: &str,
        size: f32,
        font: iced_native::Font,
        bounds: iced_native::Size,
    ) -> (f32, f32) {
        use glow_glyph::GlyphCruncher;

        let glow_glyph::FontId(font_id) = self.find_font(font);

        let section = glow_glyph::Section {
            text: content,
            scale: glow_glyph::Scale { x: size, y: size },
            bounds: (bounds.width, bounds.height),
            font_id: glow_glyph::FontId(font_id),
            ..Default::default()
        };

        if let Some(bounds) =
            self.measure_brush.borrow_mut().glyph_bounds(section)
        {
            (bounds.width().ceil(), bounds.height().ceil())
        } else {
            (0.0, 0.0)
        }
    }

    pub fn space_width(&self, size: f32) -> f32 {
        use glow_glyph::GlyphCruncher;

        let glyph_brush = self.measure_brush.borrow();

        // TODO: Select appropriate font
        let font = &glyph_brush.fonts()[0];

        font.glyph(' ')
            .scaled(glow_glyph::Scale { x: size, y: size })
            .h_metrics()
            .advance_width
    }

    pub fn trim_measurement_cache(&mut self) {
        // TODO: We should probably use a `GlyphCalculator` for this. However,
        // it uses a lifetimed `GlyphCalculatorGuard` with side-effects on drop.
        // This makes stuff quite inconvenient. A manual method for trimming the
        // cache would make our lives easier.
        loop {
            let action = self
                .measure_brush
                .borrow_mut()
                .process_queued(|_, _| {}, |_| {});

            match action {
                Ok(_) => break,
                Err(glyph_brush::BrushError::TextureTooSmall { suggested }) => {
                    let (width, height) = suggested;

                    self.measure_brush
                        .borrow_mut()
                        .resize_texture(width, height);
                }
            }
        }
    }

    pub fn find_font(&self, font: iced_native::Font) -> glow_glyph::FontId {
        match font {
            iced_native::Font::Default => glow_glyph::FontId(0),
            iced_native::Font::External { name, bytes } => {
                if let Some(font_id) = self.draw_font_map.borrow().get(name) {
                    return *font_id;
                }

                // TODO: Find a way to share font data
                let _ = self.measure_brush.borrow_mut().add_font_bytes(bytes);

                let font_id =
                    self.draw_brush.borrow_mut().add_font_bytes(bytes);

                let _ = self
                    .draw_font_map
                    .borrow_mut()
                    .insert(String::from(name), font_id);

                font_id
            }
        }
    }
}
