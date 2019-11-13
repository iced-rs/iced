mod font;

use crate::Transformation;
use std::cell::RefCell;

pub struct Pipeline {
    draw_brush: wgpu_glyph::GlyphBrush<'static, ()>,
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
                default_font.clone(),
                mono_font,
            ])
            .initial_cache_size((2048, 2048))
            .build(device, wgpu::TextureFormat::Bgra8UnormSrgb);

        let measure_brush =
            glyph_brush::GlyphBrushBuilder::using_font_bytes(default_font)
                .build();

        Pipeline {
            draw_brush,
            measure_brush: RefCell::new(measure_brush),
        }
    }

    pub fn overlay_font(&self) -> wgpu_glyph::FontId {
        wgpu_glyph::FontId(1)
    }

    pub fn queue(&mut self, section: wgpu_glyph::Section) {
        self.draw_brush.queue(section);
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
            .draw_queued_with_transform_and_scissoring(
                device,
                encoder,
                target,
                transformation.into(),
                region,
            )
            .expect("Draw text");
    }

    pub fn measure(&self, section: &wgpu_glyph::Section<'_>) -> (f32, f32) {
        use wgpu_glyph::GlyphCruncher;

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
        // Trim measurements cache
        // TODO: We should probably use a `GlyphCalculator` for this. However,
        // it uses a lifetimed `GlyphCalculatorGuard` with side-effects on drop.
        // This makes stuff quite inconvenient. A manual method for trimming the
        // cache would make our lives easier.
        self.measure_brush
            .borrow_mut()
            .process_queued(|_, _| {}, |_| {})
            .expect("Trim text measurements");
    }
}
