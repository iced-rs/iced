mod font;

use crate::Transformation;

use std::cell::RefCell;
use std::collections::HashMap;

pub const BUILTIN_ICONS: iced_native::Font = iced_native::Font::External {
    name: "iced_wgpu icons",
    bytes: include_bytes!("text/icons.ttf"),
};

pub const CHECKMARK_ICON: char = '\u{F00C}';

pub struct Pipeline {
    draw_brush: RefCell<wgpu_glyph::GlyphBrush<'static, ()>>,
    draw_font_map: RefCell<HashMap<String, wgpu_glyph::FontId>>,

    measure_brush: RefCell<glyph_brush::GlyphBrush<'static, ()>>,
}

impl Pipeline {
    pub fn new(device: &mut wgpu::Device) -> Self {
        // TODO: Font customization
        let font_source = font::Source::new();

        let default_font = font_source
            .load(&[font::Family::SansSerif, font::Family::Serif])
            .expect("Find sans-serif or serif font");

        let mono_font = font_source
            .load(&[font::Family::Monospace])
            .expect("Find monospace font");

        let draw_brush =
            wgpu_glyph::GlyphBrushBuilder::using_fonts_bytes(vec![
                mono_font,
                default_font.clone(),
            ])
            .initial_cache_size((2048, 2048))
            .build(device, wgpu::TextureFormat::Bgra8UnormSrgb);

        let measure_brush =
            glyph_brush::GlyphBrushBuilder::using_font_bytes(default_font)
                .build();

        Pipeline {
            draw_brush: RefCell::new(draw_brush),
            draw_font_map: RefCell::new(HashMap::new()),

            measure_brush: RefCell::new(measure_brush),
        }
    }

    pub fn overlay_font(&self) -> wgpu_glyph::FontId {
        wgpu_glyph::FontId(0)
    }

    pub fn queue(&mut self, section: wgpu_glyph::Section) {
        self.draw_brush.borrow_mut().queue(section);
    }

    pub fn draw_queued(
        &mut self,
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        transformation: Transformation,
        region: wgpu_glyph::Region,
    ) {
        self.draw_brush
            .borrow_mut()
            .draw_queued_with_transform_and_scissoring(
                device,
                encoder,
                target,
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
        use wgpu_glyph::GlyphCruncher;

        let wgpu_glyph::FontId(font_id) = self.find_font(font);

        let section = wgpu_glyph::Section {
            text: content,
            scale: wgpu_glyph::Scale { x: size, y: size },
            bounds: (bounds.width, bounds.height),

            // TODO: This is a bit hacky. We are loading the debug font as the
            // first font in the `draw_brush`. The `measure_brush` does not
            // contain this, hence we subtract 1.
            //
            // This should go away once we unify `draw_brush` and
            // `measure_brush`.
            font_id: wgpu_glyph::FontId(font_id - 1),
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
        use wgpu_glyph::GlyphCruncher;

        let glyph_brush = self.measure_brush.borrow();

        // TODO: Select appropriate font
        let font = &glyph_brush.fonts()[0];

        font.glyph(' ')
            .scaled(wgpu_glyph::Scale { x: size, y: size })
            .h_metrics()
            .advance_width
    }

    pub fn clear_measurement_cache(&mut self) {
        // TODO: We should probably use a `GlyphCalculator` for this. However,
        // it uses a lifetimed `GlyphCalculatorGuard` with side-effects on drop.
        // This makes stuff quite inconvenient. A manual method for trimming the
        // cache would make our lives easier.
        self.measure_brush
            .borrow_mut()
            .process_queued(|_, _| {}, |_| {})
            .expect("Trim text measurements");
    }

    pub fn find_font(&self, font: iced_native::Font) -> wgpu_glyph::FontId {
        match font {
            iced_native::Font::Default => wgpu_glyph::FontId(1),
            iced_native::Font::External { name, bytes } => {
                if let Some(font_id) = self.draw_font_map.borrow().get(name) {
                    return *font_id;
                }

                // TODO: Find a way to share font data
                let _ = self.measure_brush.borrow_mut().add_font_bytes(bytes);

                let font_id =
                    self.draw_brush.borrow_mut().add_font_bytes(bytes);

                self.draw_font_map
                    .borrow_mut()
                    .insert(String::from(name), font_id);

                font_id
            }
        }
    }
}
