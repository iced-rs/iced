use crate::quad;
use crate::text;
use crate::triangle;
use crate::{Settings, Transformation, Viewport};
use iced_graphics::backend;
use iced_graphics::font;
use iced_graphics::Layer;
use iced_graphics::Primitive;
use iced_native::mouse;
use iced_native::{Font, HorizontalAlignment, Size, VerticalAlignment};

/// A [`glow`] renderer.
///
/// [`glow`]: https://github.com/grovesNL/glow
#[derive(Debug)]
pub struct Backend {
    quad_pipeline: quad::Pipeline,
    text_pipeline: text::Pipeline,
    triangle_pipeline: triangle::Pipeline,
}

impl Backend {
    /// Creates a new [`Renderer`].
    ///
    /// [`Renderer`]: struct.Renderer.html
    pub fn new(gl: &glow::Context, settings: Settings) -> Self {
        let text_pipeline = text::Pipeline::new(gl, settings.default_font);
        let quad_pipeline = quad::Pipeline::new(gl);
        let triangle_pipeline =
            triangle::Pipeline::new(gl, settings.antialiasing);

        Self {
            quad_pipeline,
            text_pipeline,
            triangle_pipeline,
        }
    }

    /// Draws the provided primitives in the given [`Target`].
    ///
    /// The text provided as overlay will be renderer on top of the primitives.
    /// This is useful for rendering debug information.
    ///
    /// [`Target`]: struct.Target.html
    pub fn draw<T: AsRef<str>>(
        &mut self,
        gl: &glow::Context,
        viewport: &Viewport,
        (primitive, mouse_interaction): &(Primitive, mouse::Interaction),
        scale_factor: f64,
        overlay_text: &[T],
    ) -> mouse::Interaction {
        let (width, height) = viewport.dimensions();
        let scale_factor = scale_factor as f32;
        let transformation = viewport.transformation();

        let mut layers = Layer::generate(primitive, viewport);
        layers.push(Layer::overlay(overlay_text, viewport));

        for layer in layers {
            self.flush(
                gl,
                viewport,
                scale_factor,
                transformation,
                &layer,
                width,
                height,
            );
        }

        *mouse_interaction
    }

    fn flush(
        &mut self,
        gl: &glow::Context,
        viewport: &Viewport,
        scale_factor: f32,
        transformation: Transformation,
        layer: &Layer<'_>,
        target_width: u32,
        target_height: u32,
    ) {
        let bounds = layer.bounds * scale_factor;

        if !layer.quads.is_empty() {
            self.quad_pipeline.draw(
                gl,
                viewport,
                &layer.quads,
                transformation,
                scale_factor,
                bounds,
            );
        }

        if !layer.meshes.is_empty() {
            let scaled = transformation
                * Transformation::scale(scale_factor, scale_factor);

            self.triangle_pipeline.draw(
                gl,
                target_width,
                target_height,
                scaled,
                scale_factor,
                &layer.meshes,
            );
        }

        if !layer.text.is_empty() {
            for text in layer.text.iter() {
                // Target physical coordinates directly to avoid blurry text
                let text = glow_glyph::Section {
                    text: text.content,
                    // TODO: We `round` here to avoid rerasterizing text when
                    // its position changes slightly. This can make text feel a
                    // bit "jumpy". We may be able to do better once we improve
                    // our text rendering/caching pipeline.
                    screen_position: (
                        (text.bounds.x * scale_factor).round(),
                        (text.bounds.y * scale_factor).round(),
                    ),
                    // TODO: Fix precision issues with some scale factors.
                    //
                    // The `ceil` here can cause some words to render on the
                    // same line when they should not.
                    //
                    // Ideally, `wgpu_glyph` should be able to compute layout
                    // using logical positions, and then apply the proper
                    // scaling when rendering. This would ensure that both
                    // measuring and rendering follow the same layout rules.
                    bounds: (
                        (text.bounds.width * scale_factor).ceil(),
                        (text.bounds.height * scale_factor).ceil(),
                    ),
                    scale: glow_glyph::Scale {
                        x: text.size * scale_factor,
                        y: text.size * scale_factor,
                    },
                    color: text.color,
                    font_id: self.text_pipeline.find_font(text.font),
                    layout: glow_glyph::Layout::default()
                        .h_align(match text.horizontal_alignment {
                            HorizontalAlignment::Left => {
                                glow_glyph::HorizontalAlign::Left
                            }
                            HorizontalAlignment::Center => {
                                glow_glyph::HorizontalAlign::Center
                            }
                            HorizontalAlignment::Right => {
                                glow_glyph::HorizontalAlign::Right
                            }
                        })
                        .v_align(match text.vertical_alignment {
                            VerticalAlignment::Top => {
                                glow_glyph::VerticalAlign::Top
                            }
                            VerticalAlignment::Center => {
                                glow_glyph::VerticalAlign::Center
                            }
                            VerticalAlignment::Bottom => {
                                glow_glyph::VerticalAlign::Bottom
                            }
                        }),
                    ..Default::default()
                };

                self.text_pipeline.queue(text);
            }

            self.text_pipeline.draw_queued(
                gl,
                transformation,
                glow_glyph::Region {
                    x: bounds.x,
                    y: viewport.height()
                        - (bounds.y + bounds.height).min(viewport.height()),
                    width: bounds.width,
                    height: bounds.height,
                },
            );
        }
    }
}

impl iced_graphics::Backend for Backend {
    fn trim_measurements(&mut self) {
        self.text_pipeline.trim_measurement_cache()
    }
}

impl backend::Text for Backend {
    const ICON_FONT: Font = font::ICONS;
    const CHECKMARK_ICON: char = font::CHECKMARK_ICON;

    fn measure(
        &self,
        contents: &str,
        size: f32,
        font: Font,
        bounds: Size,
    ) -> (f32, f32) {
        self.text_pipeline.measure(contents, size, font, bounds)
    }

    fn space_width(&self, size: f32) -> f32 {
        self.text_pipeline.space_width(size)
    }
}
