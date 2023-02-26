use crate::{Color, Font, Settings, Size, Viewport};

use iced_graphics::alignment;
use iced_graphics::backend;
use iced_graphics::text;
use iced_graphics::{Background, Primitive, Rectangle, Vector};

use std::borrow::Cow;

pub struct Backend {
    default_font: Font,
    default_text_size: f32,
    text_pipeline: crate::text::Pipeline,
}

impl Backend {
    pub fn new(settings: Settings) -> Self {
        Self {
            default_font: settings.default_font,
            default_text_size: settings.default_text_size,
            text_pipeline: crate::text::Pipeline::new(),
        }
    }

    pub fn draw<T: AsRef<str>>(
        &mut self,
        pixels: &mut tiny_skia::PixmapMut<'_>,
        primitives: &[Primitive],
        viewport: &Viewport,
        background_color: Color,
        overlay: &[T],
    ) {
        pixels.fill(into_color(background_color));

        let scale_factor = viewport.scale_factor() as f32;

        for primitive in primitives {
            self.draw_primitive(
                primitive,
                pixels,
                None,
                scale_factor,
                Vector::ZERO,
            );
        }

        for (i, text) in overlay.iter().enumerate() {
            const OVERLAY_TEXT_SIZE: f32 = 20.0;

            self.draw_primitive(
                &Primitive::Text {
                    content: text.as_ref().to_owned(),
                    size: OVERLAY_TEXT_SIZE,
                    bounds: Rectangle {
                        x: 10.0,
                        y: 10.0 + i as f32 * OVERLAY_TEXT_SIZE * 1.2,
                        width: f32::INFINITY,
                        height: f32::INFINITY,
                    },
                    color: Color::BLACK,
                    font: Font::Monospace,
                    horizontal_alignment: alignment::Horizontal::Left,
                    vertical_alignment: alignment::Vertical::Top,
                },
                pixels,
                None,
                scale_factor,
                Vector::ZERO,
            );
        }

        self.text_pipeline.end_frame();
    }

    fn draw_primitive(
        &mut self,
        primitive: &Primitive,
        pixels: &mut tiny_skia::PixmapMut<'_>,
        clip_mask: Option<&tiny_skia::ClipMask>,
        scale_factor: f32,
        translation: Vector,
    ) {
        match primitive {
            Primitive::None => {}
            Primitive::Quad {
                bounds,
                background,
                border_radius,
                border_width,
                border_color,
            } => {
                let transform = tiny_skia::Transform::from_translate(
                    translation.x,
                    translation.y,
                )
                .post_scale(scale_factor, scale_factor);

                let path = rounded_rectangle(*bounds, *border_radius);

                pixels.fill_path(
                    &path,
                    &tiny_skia::Paint {
                        shader: match background {
                            Background::Color(color) => {
                                tiny_skia::Shader::SolidColor(into_color(
                                    *color,
                                ))
                            }
                        },
                        anti_alias: true,
                        ..tiny_skia::Paint::default()
                    },
                    tiny_skia::FillRule::EvenOdd,
                    transform,
                    clip_mask,
                );

                if *border_width > 0.0 {
                    pixels.stroke_path(
                        &path,
                        &tiny_skia::Paint {
                            shader: tiny_skia::Shader::SolidColor(into_color(
                                *border_color,
                            )),
                            anti_alias: true,
                            ..tiny_skia::Paint::default()
                        },
                        &tiny_skia::Stroke {
                            width: *border_width,
                            ..tiny_skia::Stroke::default()
                        },
                        transform,
                        clip_mask,
                    );
                }
            }
            Primitive::Text {
                content,
                bounds,
                color,
                size,
                font,
                horizontal_alignment,
                vertical_alignment,
            } => {
                self.text_pipeline.draw(
                    content,
                    (*bounds + translation) * scale_factor,
                    *color,
                    *size * scale_factor,
                    *font,
                    *horizontal_alignment,
                    *vertical_alignment,
                    pixels,
                    clip_mask,
                );
            }
            Primitive::Image { .. } => {
                // TODO
            }
            Primitive::Svg { .. } => {
                // TODO
            }
            Primitive::Group { primitives } => {
                for primitive in primitives {
                    self.draw_primitive(
                        primitive,
                        pixels,
                        clip_mask,
                        scale_factor,
                        translation,
                    );
                }
            }
            Primitive::Translate {
                translation: offset,
                content,
            } => {
                self.draw_primitive(
                    content,
                    pixels,
                    clip_mask,
                    scale_factor,
                    translation + *offset,
                );
            }
            Primitive::Clip { bounds, content } => {
                self.draw_primitive(
                    content,
                    pixels,
                    Some(&rectangular_clip_mask(
                        pixels,
                        *bounds * scale_factor,
                    )),
                    scale_factor,
                    translation,
                );
            }
            Primitive::Cached { cache } => {
                self.draw_primitive(
                    cache,
                    pixels,
                    clip_mask,
                    scale_factor,
                    translation,
                );
            }
            Primitive::SolidMesh { .. } | Primitive::GradientMesh { .. } => {}
        }
    }
}

fn into_color(color: Color) -> tiny_skia::Color {
    tiny_skia::Color::from_rgba(color.b, color.g, color.r, color.a)
        .expect("Convert color from iced to tiny_skia")
}

fn rounded_rectangle(
    bounds: Rectangle,
    border_radius: [f32; 4],
) -> tiny_skia::Path {
    let [top_left, top_right, bottom_right, bottom_left] = border_radius;

    if top_left == top_right
        && top_left == bottom_right
        && top_left == bottom_left
        && top_left == bounds.width / 2.0
        && top_left == bounds.height / 2.0
    {
        return tiny_skia::PathBuilder::from_circle(
            bounds.x + bounds.width / 2.0,
            bounds.y + bounds.height / 2.0,
            top_left,
        )
        .expect("Build circle path");
    }

    let mut builder = tiny_skia::PathBuilder::new();

    builder.move_to(bounds.x + top_left, bounds.y);
    builder.line_to(bounds.x + bounds.width - top_right, bounds.y);

    if top_right > 0.0 {
        builder.quad_to(
            bounds.x + bounds.width,
            bounds.y,
            bounds.x + bounds.width,
            bounds.y + top_right,
        );
    }

    builder.line_to(
        bounds.x + bounds.width,
        bounds.y + bounds.height - bottom_right,
    );

    if bottom_right > 0.0 {
        builder.quad_to(
            bounds.x + bounds.width,
            bounds.y + bounds.height,
            bounds.x + bounds.width - bottom_right,
            bounds.y + bounds.height,
        );
    }

    builder.line_to(bounds.x + bottom_left, bounds.y + bounds.height);

    if bottom_right > 0.0 {
        builder.quad_to(
            bounds.x,
            bounds.y + bounds.height,
            bounds.x,
            bounds.y + bounds.height - bottom_left,
        );
    }

    builder.line_to(bounds.x, bounds.y + top_left);

    if top_left > 0.0 {
        builder.quad_to(bounds.x, bounds.y, bounds.x + top_left, bounds.y);
    }

    builder.finish().expect("Build rounded rectangle path")
}

fn rectangular_clip_mask(
    pixels: &tiny_skia::PixmapMut<'_>,
    bounds: Rectangle,
) -> tiny_skia::ClipMask {
    let mut clip_mask = tiny_skia::ClipMask::new();

    let path = {
        let mut builder = tiny_skia::PathBuilder::new();
        builder.push_rect(bounds.x, bounds.y, bounds.width, bounds.height);

        builder.finish().unwrap()
    };

    clip_mask
        .set_path(
            pixels.width(),
            pixels.height(),
            &path,
            tiny_skia::FillRule::EvenOdd,
            true,
        )
        .expect("Set path of clipping area");

    clip_mask
}

impl iced_graphics::Backend for Backend {
    fn trim_measurements(&mut self) {
        self.text_pipeline.trim_measurement_cache();
    }
}

impl backend::Text for Backend {
    const ICON_FONT: Font = Font::Name("Iced-Icons");
    const CHECKMARK_ICON: char = '\u{f00c}';
    const ARROW_DOWN_ICON: char = '\u{e800}';

    fn default_font(&self) -> Font {
        self.default_font
    }

    fn default_size(&self) -> f32 {
        self.default_text_size
    }

    fn measure(
        &self,
        contents: &str,
        size: f32,
        font: Font,
        bounds: Size,
    ) -> (f32, f32) {
        self.text_pipeline.measure(contents, size, font, bounds)
    }

    fn hit_test(
        &self,
        contents: &str,
        size: f32,
        font: Font,
        bounds: Size,
        point: iced_native::Point,
        nearest_only: bool,
    ) -> Option<text::Hit> {
        self.text_pipeline.hit_test(
            contents,
            size,
            font,
            bounds,
            point,
            nearest_only,
        )
    }

    fn load_font(&mut self, font: Cow<'static, [u8]>) {
        self.text_pipeline.load_font(font);
    }
}

#[cfg(feature = "image")]
impl backend::Image for Backend {
    fn dimensions(&self, _handle: &iced_native::image::Handle) -> Size<u32> {
        // TODO
        Size::new(0, 0)
    }
}

#[cfg(feature = "svg")]
impl backend::Svg for Backend {
    fn viewport_dimensions(
        &self,
        _handle: &iced_native::svg::Handle,
    ) -> Size<u32> {
        // TODO
        Size::new(0, 0)
    }
}
