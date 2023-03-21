use crate::core::alignment;
use crate::core::text;
use crate::core::{Background, Color, Font, Point, Rectangle, Size, Vector};
use crate::graphics::backend;
use crate::graphics::{Primitive, Viewport};
use crate::Settings;

use std::borrow::Cow;

pub struct Backend {
    default_font: Font,
    default_text_size: f32,
    text_pipeline: crate::text::Pipeline,

    #[cfg(feature = "image")]
    raster_pipeline: crate::raster::Pipeline,

    #[cfg(feature = "svg")]
    vector_pipeline: crate::vector::Pipeline,

    last_primitives: Vec<Primitive>,
    last_background_color: Color,
}

impl Backend {
    pub fn new(settings: Settings) -> Self {
        Self {
            default_font: settings.default_font,
            default_text_size: settings.default_text_size,
            text_pipeline: crate::text::Pipeline::new(),

            #[cfg(feature = "image")]
            raster_pipeline: crate::raster::Pipeline::new(),

            #[cfg(feature = "svg")]
            vector_pipeline: crate::vector::Pipeline::new(),

            last_primitives: Vec::new(),
            last_background_color: Color::BLACK,
        }
    }

    pub fn draw<T: AsRef<str>>(
        &mut self,
        pixels: &mut tiny_skia::PixmapMut<'_>,
        clip_mask: &mut tiny_skia::ClipMask,
        primitives: &[Primitive],
        viewport: &Viewport,
        background_color: Color,
        overlay: &[T],
    ) {
        let damage = if self.last_background_color == background_color {
            Primitive::damage_list(&self.last_primitives, primitives)
        } else {
            vec![Rectangle::with_size(viewport.logical_size())]
        };

        if damage.is_empty() {
            return;
        }

        self.last_primitives = primitives.to_vec();
        self.last_background_color = background_color;

        let scale_factor = viewport.scale_factor() as f32;

        dbg!(&damage);

        for region in &damage {
            let region = *region * scale_factor;

            pixels.fill_path(
                &tiny_skia::PathBuilder::from_rect(
                    tiny_skia::Rect::from_xywh(
                        region.x,
                        region.y,
                        region.width.min(viewport.physical_width() as f32),
                        region.height.min(viewport.physical_height() as f32),
                    )
                    .expect("Create damage rectangle"),
                ),
                &tiny_skia::Paint {
                    shader: tiny_skia::Shader::SolidColor(into_color(
                        background_color,
                    )),
                    ..Default::default()
                },
                tiny_skia::FillRule::default(),
                tiny_skia::Transform::identity(),
                None,
            );
        }

        for primitive in primitives {
            self.draw_primitive(
                primitive,
                pixels,
                clip_mask,
                None,
                scale_factor,
                Vector::ZERO,
                &damage,
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
                    font: Font::MONOSPACE,
                    horizontal_alignment: alignment::Horizontal::Left,
                    vertical_alignment: alignment::Vertical::Top,
                },
                pixels,
                clip_mask,
                None,
                scale_factor,
                Vector::ZERO,
                &[],
            );
        }

        self.text_pipeline.trim_cache();

        #[cfg(feature = "image")]
        self.raster_pipeline.trim_cache();

        #[cfg(feature = "svg")]
        self.vector_pipeline.trim_cache();
    }

    fn draw_primitive(
        &mut self,
        primitive: &Primitive,
        pixels: &mut tiny_skia::PixmapMut<'_>,
        clip_mask: &mut tiny_skia::ClipMask,
        clip_bounds: Option<Rectangle>,
        scale_factor: f32,
        translation: Vector,
        damage: &[Rectangle],
    ) {
        match primitive {
            Primitive::Quad {
                bounds,
                background,
                border_radius,
                border_width,
                border_color,
            } => {
                if !damage.iter().any(|damage| damage.intersects(bounds)) {
                    return;
                }

                let transform = tiny_skia::Transform::from_translate(
                    translation.x,
                    translation.y,
                )
                .post_scale(scale_factor, scale_factor);

                let path = rounded_rectangle(*bounds, *border_radius);
                let clip_mask = clip_bounds.map(|_| clip_mask as &_);

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
                if !damage
                    .iter()
                    .any(|damage| damage.intersects(&primitive.bounds()))
                {
                    return;
                }

                self.text_pipeline.draw(
                    content,
                    (*bounds + translation) * scale_factor,
                    *color,
                    *size * scale_factor,
                    *font,
                    *horizontal_alignment,
                    *vertical_alignment,
                    pixels,
                    clip_bounds.map(|_| clip_mask as &_),
                );
            }
            #[cfg(feature = "image")]
            Primitive::Image { handle, bounds } => {
                if !damage.iter().any(|damage| damage.intersects(bounds)) {
                    return;
                }

                let transform = tiny_skia::Transform::from_translate(
                    translation.x,
                    translation.y,
                )
                .post_scale(scale_factor, scale_factor);

                self.raster_pipeline.draw(
                    handle,
                    *bounds,
                    pixels,
                    transform,
                    clip_bounds.map(|_| clip_mask as &_),
                );
            }
            #[cfg(feature = "svg")]
            Primitive::Svg {
                handle,
                bounds,
                color,
            } => {
                self.vector_pipeline.draw(
                    handle,
                    *color,
                    (*bounds + translation) * scale_factor,
                    pixels,
                    clip_bounds.map(|_| clip_mask as &_),
                );
            }
            Primitive::Fill {
                path,
                paint,
                rule,
                transform,
            } => {
                pixels.fill_path(
                    path,
                    paint,
                    *rule,
                    transform
                        .post_translate(translation.x, translation.y)
                        .post_scale(scale_factor, scale_factor),
                    clip_bounds.map(|_| clip_mask as &_),
                );
            }
            Primitive::Stroke {
                path,
                paint,
                stroke,
                transform,
            } => {
                pixels.stroke_path(
                    path,
                    paint,
                    stroke,
                    transform
                        .post_translate(translation.x, translation.y)
                        .post_scale(scale_factor, scale_factor),
                    clip_bounds.map(|_| clip_mask as &_),
                );
            }
            Primitive::Group { primitives } => {
                for primitive in primitives {
                    self.draw_primitive(
                        primitive,
                        pixels,
                        clip_mask,
                        clip_bounds,
                        scale_factor,
                        translation,
                        damage,
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
                    clip_bounds,
                    scale_factor,
                    translation + *offset,
                    damage,
                );
            }
            Primitive::Clip { bounds, content } => {
                let bounds = (*bounds + translation) * scale_factor;

                if bounds.x + bounds.width <= 0.0
                    || bounds.y + bounds.height <= 0.0
                    || bounds.x as u32 >= pixels.width()
                    || bounds.y as u32 >= pixels.height()
                {
                    return;
                }

                adjust_clip_mask(clip_mask, pixels, bounds);

                self.draw_primitive(
                    content,
                    pixels,
                    clip_mask,
                    Some(bounds),
                    scale_factor,
                    translation,
                    damage,
                );

                if let Some(bounds) = clip_bounds {
                    adjust_clip_mask(clip_mask, pixels, bounds);
                } else {
                    clip_mask.clear();
                }
            }
            Primitive::Cache { content } => {
                self.draw_primitive(
                    content,
                    pixels,
                    clip_mask,
                    clip_bounds,
                    scale_factor,
                    translation,
                    damage,
                );
            }
            Primitive::SolidMesh { .. } | Primitive::GradientMesh { .. } => {
                // Not supported!
                // TODO: Draw a placeholder (?) / Log it (?)
            }
            _ => {
                // Not supported!
            }
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

    if top_left == 0.0
        && top_right == 0.0
        && bottom_right == 0.0
        && bottom_left == 0.0
    {
        return tiny_skia::PathBuilder::from_rect(
            tiny_skia::Rect::from_xywh(
                bounds.x,
                bounds.y,
                bounds.width,
                bounds.height,
            )
            .expect("Build quad rectangle"),
        );
    }

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
        arc_to(
            &mut builder,
            bounds.x + bounds.width - top_right,
            bounds.y,
            bounds.x + bounds.width,
            bounds.y + top_right,
            top_right,
        );
    }

    maybe_line_to(
        &mut builder,
        bounds.x + bounds.width,
        bounds.y + bounds.height - bottom_right,
    );

    if bottom_right > 0.0 {
        arc_to(
            &mut builder,
            bounds.x + bounds.width,
            bounds.y + bounds.height - bottom_right,
            bounds.x + bounds.width - bottom_right,
            bounds.y + bounds.height,
            bottom_right,
        );
    }

    maybe_line_to(
        &mut builder,
        bounds.x + bottom_left,
        bounds.y + bounds.height,
    );

    if bottom_right > 0.0 {
        arc_to(
            &mut builder,
            bounds.x + bottom_left,
            bounds.y + bounds.height,
            bounds.x,
            bounds.y + bounds.height - bottom_left,
            bottom_left,
        );
    }

    maybe_line_to(&mut builder, bounds.x, bounds.y + top_left);

    if top_left > 0.0 {
        arc_to(
            &mut builder,
            bounds.x,
            bounds.y + top_left,
            bounds.x + top_left,
            bounds.y,
            top_left,
        );
    }

    builder.finish().expect("Build rounded rectangle path")
}

fn maybe_line_to(path: &mut tiny_skia::PathBuilder, x: f32, y: f32) {
    if path.last_point() != Some(tiny_skia::Point { x, y }) {
        path.line_to(x, y);
    }
}

fn arc_to(
    path: &mut tiny_skia::PathBuilder,
    x_from: f32,
    y_from: f32,
    x_to: f32,
    y_to: f32,
    radius: f32,
) {
    let svg_arc = kurbo::SvgArc {
        from: kurbo::Point::new(f64::from(x_from), f64::from(y_from)),
        to: kurbo::Point::new(f64::from(x_to), f64::from(y_to)),
        radii: kurbo::Vec2::new(f64::from(radius), f64::from(radius)),
        x_rotation: 0.0,
        large_arc: false,
        sweep: true,
    };

    match kurbo::Arc::from_svg_arc(&svg_arc) {
        Some(arc) => {
            arc.to_cubic_beziers(0.1, |p1, p2, p| {
                path.cubic_to(
                    p1.x as f32,
                    p1.y as f32,
                    p2.x as f32,
                    p2.y as f32,
                    p.x as f32,
                    p.y as f32,
                );
            });
        }
        None => {
            path.line_to(x_to, y_to);
        }
    }
}

fn adjust_clip_mask(
    clip_mask: &mut tiny_skia::ClipMask,
    pixels: &tiny_skia::PixmapMut<'_>,
    bounds: Rectangle,
) {
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
}

impl iced_graphics::Backend for Backend {
    fn trim_measurements(&mut self) {
        self.text_pipeline.trim_measurement_cache();
    }
}

impl backend::Text for Backend {
    const ICON_FONT: Font = Font::with_name("Iced-Icons");
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
        point: Point,
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
    fn dimensions(&self, handle: &crate::core::image::Handle) -> Size<u32> {
        self.raster_pipeline.dimensions(handle)
    }
}

#[cfg(feature = "svg")]
impl backend::Svg for Backend {
    fn viewport_dimensions(
        &self,
        handle: &crate::core::svg::Handle,
    ) -> Size<u32> {
        self.vector_pipeline.viewport_dimensions(handle)
    }
}
