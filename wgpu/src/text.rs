use crate::Transformation;
use iced_graphics::font;
use std::{cell::RefCell, collections::HashMap};
use wgpu_glyph::ab_glyph;

#[derive(Debug)]
pub struct Pipeline {
    draw_brush: RefCell<wgpu_glyph::GlyphBrush<()>>,
    draw_font_map: RefCell<HashMap<String, wgpu_glyph::FontId>>,
    measure_brush: RefCell<glyph_brush::GlyphBrush<()>>,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        default_font: Option<&[u8]>,
        multithreading: bool,
    ) -> Self {
        let default_font = default_font.map(|slice| slice.to_vec());

        // TODO: Font customization
        #[cfg(not(target_os = "ios"))]
        #[cfg(feature = "default_system_font")]
        let default_font = {
            default_font.or_else(|| {
                font::Source::new()
                    .load(&[font::Family::SansSerif, font::Family::Serif])
                    .ok()
            })
        };

        let default_font =
            default_font.unwrap_or_else(|| font::FALLBACK.to_vec());

        let font = ab_glyph::FontArc::try_from_vec(default_font)
            .unwrap_or_else(|_| {
                log::warn!(
                    "System font failed to load. Falling back to \
                    embedded font..."
                );

                ab_glyph::FontArc::try_from_slice(font::FALLBACK)
                    .expect("Load fallback font")
            });

        let draw_brush =
            wgpu_glyph::GlyphBrushBuilder::using_font(font.clone())
                .initial_cache_size((2048, 2048))
                .draw_cache_multithread(multithreading)
                .build(device, format);

        let measure_brush =
            glyph_brush::GlyphBrushBuilder::using_font(font).build();

        Pipeline {
            draw_brush: RefCell::new(draw_brush),
            draw_font_map: RefCell::new(HashMap::new()),
            measure_brush: RefCell::new(measure_brush),
        }
    }

    pub fn queue(&mut self, section: wgpu_glyph::Section<'_>) {
        self.draw_brush.borrow_mut().queue(section);
    }

    pub fn draw_queued(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        transformation: Transformation,
        region: wgpu_glyph::Region,
    ) {
        self.draw_brush
            .borrow_mut()
            .draw_queued_with_transform_and_scissoring(
                device,
                staging_belt,
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
            bounds: (bounds.width, bounds.height),
            text: vec![wgpu_glyph::Text {
                text: content,
                scale: size.into(),
                font_id: wgpu_glyph::FontId(font_id),
                extra: wgpu_glyph::Extra::default(),
            }],
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

    pub fn find_font(&self, font: iced_native::Font) -> wgpu_glyph::FontId {
        match font {
            iced_native::Font::Default => wgpu_glyph::FontId(0),
            iced_native::Font::External { name, bytes } => {
                if let Some(font_id) = self.draw_font_map.borrow().get(name) {
                    return *font_id;
                }

                let font = ab_glyph::FontArc::try_from_slice(bytes)
                    .expect("Load font");

                let _ = self.measure_brush.borrow_mut().add_font(font.clone());

                let font_id = self.draw_brush.borrow_mut().add_font(font);

                let _ = self
                    .draw_font_map
                    .borrow_mut()
                    .insert(String::from(name), font_id);

                font_id
            }
        }
    }
}
