use crate::core::alignment;
use crate::core::{Rectangle, Size};
use crate::graphics::color;
use crate::graphics::text::cache::{self, Cache};
use crate::graphics::text::{FontSystem, Paragraph};
use crate::layer::Text;

use std::borrow::Cow;
use std::cell::RefCell;
use std::sync::Arc;

#[allow(missing_debug_implementations)]
pub struct Pipeline {
    font_system: FontSystem,
    renderers: Vec<glyphon::TextRenderer>,
    atlas: glyphon::TextAtlas,
    prepare_layer: usize,
    cache: RefCell<Cache>,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self {
        Pipeline {
            font_system: FontSystem::new(),
            renderers: Vec::new(),
            atlas: glyphon::TextAtlas::with_color_mode(
                device,
                queue,
                format,
                if color::GAMMA_CORRECTION {
                    glyphon::ColorMode::Accurate
                } else {
                    glyphon::ColorMode::Web
                },
            ),
            prepare_layer: 0,
            cache: RefCell::new(Cache::new()),
        }
    }

    pub fn font_system(&self) -> &FontSystem {
        &self.font_system
    }

    pub fn load_font(&mut self, bytes: Cow<'static, [u8]>) {
        let _ = self.font_system.get_mut().db_mut().load_font_source(
            glyphon::fontdb::Source::Binary(Arc::new(bytes.into_owned())),
        );

        self.cache = RefCell::new(Cache::new());
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
        if self.renderers.len() <= self.prepare_layer {
            self.renderers.push(glyphon::TextRenderer::new(
                &mut self.atlas,
                device,
                Default::default(),
                None,
            ));
        }

        let font_system = self.font_system.get_mut();
        let renderer = &mut self.renderers[self.prepare_layer];
        let cache = self.cache.get_mut();

        enum Allocation {
            Paragraph(Paragraph),
            Cache(cache::KeyHash),
        }

        let allocations: Vec<_> = sections
            .iter()
            .map(|section| match section {
                Text::Managed { paragraph, .. } => {
                    paragraph.upgrade().map(Allocation::Paragraph)
                }
                Text::Cached(text) => {
                    let (key, _) = cache.allocate(
                        font_system,
                        cache::Key {
                            content: text.content,
                            size: text.size.into(),
                            line_height: f32::from(
                                text.line_height.to_absolute(text.size),
                            ),
                            font: text.font,
                            bounds: Size {
                                width: bounds.width,
                                height: bounds.height,
                            },
                            shaping: text.shaping,
                        },
                    );

                    Some(Allocation::Cache(key))
                }
            })
            .collect();

        let layer_bounds = bounds * scale_factor;

        let text_areas = sections.iter().zip(allocations.iter()).filter_map(
            |(section, allocation)| {
                let (
                    buffer,
                    bounds,
                    horizontal_alignment,
                    vertical_alignment,
                    color,
                ) = match section {
                    Text::Managed {
                        position, color, ..
                    } => {
                        use crate::core::text::Paragraph as _;

                        let Some(Allocation::Paragraph(paragraph)) = allocation
                        else {
                            return None;
                        };

                        (
                            paragraph.buffer(),
                            Rectangle::new(*position, paragraph.min_bounds()),
                            paragraph.horizontal_alignment(),
                            paragraph.vertical_alignment(),
                            *color,
                        )
                    }
                    Text::Cached(text) => {
                        let Some(Allocation::Cache(key)) = allocation else {
                            return None;
                        };

                        let buffer = cache.get(key).expect("Get cached buffer");

                        (
                            buffer,
                            text.bounds,
                            text.horizontal_alignment,
                            text.vertical_alignment,
                            text.color,
                        )
                    }
                };

                let x = bounds.x * scale_factor;
                let y = bounds.y * scale_factor;

                let max_width = bounds.width * scale_factor;
                let total_height = bounds.height * scale_factor;

                let left = match horizontal_alignment {
                    alignment::Horizontal::Left => x,
                    alignment::Horizontal::Center => x - max_width / 2.0,
                    alignment::Horizontal::Right => x - max_width,
                };

                let top = match vertical_alignment {
                    alignment::Vertical::Top => y,
                    alignment::Vertical::Center => y - total_height / 2.0,
                    alignment::Vertical::Bottom => y - total_height,
                };

                let section_bounds = Rectangle {
                    x: left,
                    y: top,
                    width: max_width,
                    height: total_height,
                };

                let clip_bounds = layer_bounds.intersection(&section_bounds)?;

                Some(glyphon::TextArea {
                    buffer,
                    left,
                    top,
                    scale: scale_factor,
                    bounds: glyphon::TextBounds {
                        left: clip_bounds.x as i32,
                        top: clip_bounds.y as i32,
                        right: (clip_bounds.x + clip_bounds.width) as i32,
                        bottom: (clip_bounds.y + clip_bounds.height) as i32,
                    },
                    default_color: {
                        let [r, g, b, a] = color::pack(color).components();

                        glyphon::Color::rgba(
                            (r * 255.0) as u8,
                            (g * 255.0) as u8,
                            (b * 255.0) as u8,
                            (a * 255.0) as u8,
                        )
                    },
                })
            },
        );

        let result = renderer.prepare(
            device,
            queue,
            font_system,
            &mut self.atlas,
            glyphon::Resolution {
                width: target_size.width,
                height: target_size.height,
            },
            text_areas,
            &mut glyphon::SwashCache::new(),
        );

        match result {
            Ok(()) => {
                self.prepare_layer += 1;
            }
            Err(glyphon::PrepareError::AtlasFull) => {
                // If the atlas cannot grow, then all bets are off.
                // Instead of panicking, we will just pray that the result
                // will be somewhat readable...
            }
        }
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
        self.cache.get_mut().trim();

        self.prepare_layer = 0;
    }
}
