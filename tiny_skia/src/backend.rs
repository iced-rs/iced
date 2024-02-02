use tiny_skia::Size;

use crate::core::{Background, Color, Gradient, Rectangle, Vector};
use crate::graphics::backend;
use crate::graphics::text;
use crate::graphics::Viewport;
use crate::primitive::{self, Primitive};

use std::borrow::Cow;

pub struct Backend {
    text_pipeline: crate::text::Pipeline,

    #[cfg(feature = "image")]
    raster_pipeline: crate::raster::Pipeline,

    #[cfg(feature = "svg")]
    vector_pipeline: crate::vector::Pipeline,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            text_pipeline: crate::text::Pipeline::new(),

            #[cfg(feature = "image")]
            raster_pipeline: crate::raster::Pipeline::new(),

            #[cfg(feature = "svg")]
            vector_pipeline: crate::vector::Pipeline::new(),
        }
    }

    pub fn draw<T: AsRef<str>>(
        &mut self,
        pixels: &mut tiny_skia::PixmapMut<'_>,
        clip_mask: &mut tiny_skia::Mask,
        primitives: &[Primitive],
        viewport: &Viewport,
        damage: &[Rectangle],
        background_color: Color,
        overlay: &[T],
    ) {
        let physical_size = viewport.physical_size();
        let scale_factor = viewport.scale_factor() as f32;

        if !overlay.is_empty() {
            let path = tiny_skia::PathBuilder::from_rect(
                tiny_skia::Rect::from_xywh(
                    0.0,
                    0.0,
                    physical_size.width as f32,
                    physical_size.height as f32,
                )
                .expect("Create damage rectangle"),
            );

            pixels.fill_path(
                &path,
                &tiny_skia::Paint {
                    shader: tiny_skia::Shader::SolidColor(into_color(Color {
                        a: 0.1,
                        ..background_color
                    })),
                    anti_alias: false,
                    ..Default::default()
                },
                tiny_skia::FillRule::default(),
                tiny_skia::Transform::identity(),
                None,
            );
        }

        for &region in damage {
            let path = tiny_skia::PathBuilder::from_rect(
                tiny_skia::Rect::from_xywh(
                    region.x,
                    region.y,
                    region.width,
                    region.height,
                )
                .expect("Create damage rectangle"),
            );

            pixels.fill_path(
                &path,
                &tiny_skia::Paint {
                    shader: tiny_skia::Shader::SolidColor(into_color(
                        background_color,
                    )),
                    anti_alias: false,
                    blend_mode: tiny_skia::BlendMode::Source,
                    ..Default::default()
                },
                tiny_skia::FillRule::default(),
                tiny_skia::Transform::identity(),
                None,
            );

            adjust_clip_mask(clip_mask, region);

            for primitive in primitives {
                self.draw_primitive(
                    primitive,
                    pixels,
                    clip_mask,
                    region,
                    scale_factor,
                    Vector::ZERO,
                );
            }

            if !overlay.is_empty() {
                pixels.stroke_path(
                    &path,
                    &tiny_skia::Paint {
                        shader: tiny_skia::Shader::SolidColor(into_color(
                            Color::from_rgb(1.0, 0.0, 0.0),
                        )),
                        anti_alias: false,
                        ..tiny_skia::Paint::default()
                    },
                    &tiny_skia::Stroke {
                        width: 1.0,
                        ..tiny_skia::Stroke::default()
                    },
                    tiny_skia::Transform::identity(),
                    None,
                );
            }
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
        clip_mask: &mut tiny_skia::Mask,
        clip_bounds: Rectangle,
        scale_factor: f32,
        translation: Vector,
    ) {
        match primitive {
            Primitive::Quad {
                bounds,
                background,
                border,
                shadow,
            } => {
                debug_assert!(
                    bounds.width.is_normal(),
                    "Quad with non-normal width!"
                );
                debug_assert!(
                    bounds.height.is_normal(),
                    "Quad with non-normal height!"
                );

                let physical_bounds = (*bounds + translation) * scale_factor;

                if !clip_bounds.intersects(&physical_bounds) {
                    return;
                }

                let clip_mask = (!physical_bounds.is_within(&clip_bounds))
                    .then_some(clip_mask as &_);

                let transform = tiny_skia::Transform::from_translate(
                    translation.x,
                    translation.y,
                )
                .post_scale(scale_factor, scale_factor);

                // Make sure the border radius is not larger than the bounds
                let border_width = border
                    .width
                    .min(bounds.width / 2.0)
                    .min(bounds.height / 2.0);

                let mut fill_border_radius = <[f32; 4]>::from(border.radius);
                for radius in &mut fill_border_radius {
                    *radius = (*radius)
                        .min(bounds.width / 2.0)
                        .min(bounds.height / 2.0);
                }
                let path = rounded_rectangle(*bounds, fill_border_radius);

                if shadow.color.a > 0.0 {
                    let shadow_bounds = (Rectangle {
                        x: bounds.x + shadow.offset.x - shadow.blur_radius,
                        y: bounds.y + shadow.offset.y - shadow.blur_radius,
                        width: bounds.width + shadow.blur_radius * 2.0,
                        height: bounds.height + shadow.blur_radius * 2.0,
                    } + translation)
                        * scale_factor;

                    let radii = fill_border_radius
                        .into_iter()
                        .map(|radius| radius * scale_factor)
                        .collect::<Vec<_>>();
                    let (x, y, width, height) = (
                        shadow_bounds.x as u32,
                        shadow_bounds.y as u32,
                        shadow_bounds.width as u32,
                        shadow_bounds.height as u32,
                    );
                    let half_width = physical_bounds.width / 2.0;
                    let half_height = physical_bounds.height / 2.0;

                    let colors = (y..y + height)
                        .flat_map(|y| {
                            (x..x + width).map(move |x| (x as f32, y as f32))
                        })
                        .filter_map(|(x, y)| {
                            Size::from_wh(half_width, half_height).map(|size| {
                                let shadow_distance = rounded_box_sdf(
                                    Vector::new(
                                        x - physical_bounds.position().x
                                            - (shadow.offset.x * scale_factor)
                                            - half_width,
                                        y - physical_bounds.position().y
                                            - (shadow.offset.y * scale_factor)
                                            - half_height,
                                    ),
                                    size,
                                    &radii,
                                );
                                let shadow_alpha = 1.0
                                    - smoothstep(
                                        -shadow.blur_radius * scale_factor,
                                        shadow.blur_radius * scale_factor,
                                        shadow_distance,
                                    );

                                let mut color = into_color(shadow.color);
                                color.apply_opacity(shadow_alpha);

                                color.to_color_u8().premultiply()
                            })
                        })
                        .collect();

                    if let Some(pixmap) = tiny_skia::IntSize::from_wh(
                        width, height,
                    )
                    .and_then(|size| {
                        tiny_skia::Pixmap::from_vec(
                            bytemuck::cast_vec(colors),
                            size,
                        )
                    }) {
                        pixels.draw_pixmap(
                            x as i32,
                            y as i32,
                            pixmap.as_ref(),
                            &tiny_skia::PixmapPaint::default(),
                            tiny_skia::Transform::default(),
                            None,
                        );
                    }
                }

                pixels.fill_path(
                    &path,
                    &tiny_skia::Paint {
                        shader: match background {
                            Background::Color(color) => {
                                tiny_skia::Shader::SolidColor(into_color(
                                    *color,
                                ))
                            }
                            Background::Gradient(Gradient::Linear(linear)) => {
                                let (start, end) =
                                    linear.angle.to_distance(bounds);

                                let stops: Vec<tiny_skia::GradientStop> =
                                    linear
                                        .stops
                                        .into_iter()
                                        .flatten()
                                        .map(|stop| {
                                            tiny_skia::GradientStop::new(
                                                stop.offset,
                                                tiny_skia::Color::from_rgba(
                                                    stop.color.b,
                                                    stop.color.g,
                                                    stop.color.r,
                                                    stop.color.a,
                                                )
                                                .expect("Create color"),
                                            )
                                        })
                                        .collect();

                                tiny_skia::LinearGradient::new(
                                    tiny_skia::Point {
                                        x: start.x,
                                        y: start.y,
                                    },
                                    tiny_skia::Point { x: end.x, y: end.y },
                                    if stops.is_empty() {
                                        vec![tiny_skia::GradientStop::new(
                                            0.0,
                                            tiny_skia::Color::BLACK,
                                        )]
                                    } else {
                                        stops
                                    },
                                    tiny_skia::SpreadMode::Pad,
                                    tiny_skia::Transform::identity(),
                                )
                                .expect("Create linear gradient")
                            }
                        },
                        anti_alias: true,
                        ..tiny_skia::Paint::default()
                    },
                    tiny_skia::FillRule::EvenOdd,
                    transform,
                    clip_mask,
                );

                if border_width > 0.0 {
                    // Border path is offset by half the border width
                    let border_bounds = Rectangle {
                        x: bounds.x + border_width / 2.0,
                        y: bounds.y + border_width / 2.0,
                        width: bounds.width - border_width,
                        height: bounds.height - border_width,
                    };

                    // Make sure the border radius is correct
                    let mut border_radius = <[f32; 4]>::from(border.radius);
                    let mut is_simple_border = true;

                    for radius in &mut border_radius {
                        *radius = if *radius == 0.0 {
                            // Path should handle this fine
                            0.0
                        } else if *radius > border_width / 2.0 {
                            *radius - border_width / 2.0
                        } else {
                            is_simple_border = false;
                            0.0
                        }
                        .min(border_bounds.width / 2.0)
                        .min(border_bounds.height / 2.0);
                    }

                    // Stroking a path works well in this case
                    if is_simple_border {
                        let border_path =
                            rounded_rectangle(border_bounds, border_radius);

                        pixels.stroke_path(
                            &border_path,
                            &tiny_skia::Paint {
                                shader: tiny_skia::Shader::SolidColor(
                                    into_color(border.color),
                                ),
                                anti_alias: true,
                                ..tiny_skia::Paint::default()
                            },
                            &tiny_skia::Stroke {
                                width: border_width,
                                ..tiny_skia::Stroke::default()
                            },
                            transform,
                            clip_mask,
                        );
                    } else {
                        // Draw corners that have too small border radii as having no border radius,
                        // but mask them with the rounded rectangle with the correct border radius.
                        let mut temp_pixmap = tiny_skia::Pixmap::new(
                            bounds.width as u32,
                            bounds.height as u32,
                        )
                        .unwrap();

                        let mut quad_mask = tiny_skia::Mask::new(
                            bounds.width as u32,
                            bounds.height as u32,
                        )
                        .unwrap();

                        let zero_bounds = Rectangle {
                            x: 0.0,
                            y: 0.0,
                            width: bounds.width,
                            height: bounds.height,
                        };
                        let path =
                            rounded_rectangle(zero_bounds, fill_border_radius);

                        quad_mask.fill_path(
                            &path,
                            tiny_skia::FillRule::EvenOdd,
                            true,
                            transform,
                        );
                        let path_bounds = Rectangle {
                            x: border_width / 2.0,
                            y: border_width / 2.0,
                            width: bounds.width - border_width,
                            height: bounds.height - border_width,
                        };

                        let border_radius_path =
                            rounded_rectangle(path_bounds, border_radius);

                        temp_pixmap.stroke_path(
                            &border_radius_path,
                            &tiny_skia::Paint {
                                shader: tiny_skia::Shader::SolidColor(
                                    into_color(border.color),
                                ),
                                anti_alias: true,
                                ..tiny_skia::Paint::default()
                            },
                            &tiny_skia::Stroke {
                                width: border_width,
                                ..tiny_skia::Stroke::default()
                            },
                            transform,
                            Some(&quad_mask),
                        );

                        pixels.draw_pixmap(
                            bounds.x as i32,
                            bounds.y as i32,
                            temp_pixmap.as_ref(),
                            &tiny_skia::PixmapPaint::default(),
                            transform,
                            clip_mask,
                        );
                    }
                }
            }
            Primitive::Paragraph {
                paragraph,
                position,
                color,
                clip_bounds: text_clip_bounds,
            } => {
                let physical_bounds =
                    (*text_clip_bounds + translation) * scale_factor;

                if !clip_bounds.intersects(&physical_bounds) {
                    return;
                }

                let clip_mask = (!physical_bounds.is_within(&clip_bounds))
                    .then_some(clip_mask as &_);

                self.text_pipeline.draw_paragraph(
                    paragraph,
                    *position + translation,
                    *color,
                    scale_factor,
                    pixels,
                    clip_mask,
                );
            }
            Primitive::Editor {
                editor,
                position,
                color,
                clip_bounds: text_clip_bounds,
            } => {
                let physical_bounds =
                    (*text_clip_bounds + translation) * scale_factor;

                if !clip_bounds.intersects(&physical_bounds) {
                    return;
                }

                let clip_mask = (!physical_bounds.is_within(&clip_bounds))
                    .then_some(clip_mask as &_);

                self.text_pipeline.draw_editor(
                    editor,
                    *position + translation,
                    *color,
                    scale_factor,
                    pixels,
                    clip_mask,
                );
            }
            Primitive::Text {
                content,
                bounds,
                color,
                size,
                line_height,
                font,
                horizontal_alignment,
                vertical_alignment,
                shaping,
                clip_bounds: text_clip_bounds,
            } => {
                let physical_bounds =
                    (*text_clip_bounds + translation) * scale_factor;

                if !clip_bounds.intersects(&physical_bounds) {
                    return;
                }

                let clip_mask = (!physical_bounds.is_within(&clip_bounds))
                    .then_some(clip_mask as &_);

                self.text_pipeline.draw_cached(
                    content,
                    *bounds + translation,
                    *color,
                    *size,
                    *line_height,
                    *font,
                    *horizontal_alignment,
                    *vertical_alignment,
                    *shaping,
                    scale_factor,
                    pixels,
                    clip_mask,
                );
            }
            Primitive::RawText(text::Raw {
                buffer,
                position,
                color,
                clip_bounds: text_clip_bounds,
            }) => {
                let Some(buffer) = buffer.upgrade() else {
                    return;
                };

                let physical_bounds =
                    (*text_clip_bounds + translation) * scale_factor;

                if !clip_bounds.intersects(&physical_bounds) {
                    return;
                }

                let clip_mask = (!physical_bounds.is_within(&clip_bounds))
                    .then_some(clip_mask as &_);

                self.text_pipeline.draw_raw(
                    &buffer,
                    *position + translation,
                    *color,
                    scale_factor,
                    pixels,
                    clip_mask,
                );
            }
            #[cfg(feature = "image")]
            Primitive::Image {
                handle,
                filter_method,
                bounds,
            } => {
                let physical_bounds = (*bounds + translation) * scale_factor;

                if !clip_bounds.intersects(&physical_bounds) {
                    return;
                }

                let clip_mask = (!physical_bounds.is_within(&clip_bounds))
                    .then_some(clip_mask as &_);

                let transform = tiny_skia::Transform::from_translate(
                    translation.x,
                    translation.y,
                )
                .post_scale(scale_factor, scale_factor);

                self.raster_pipeline.draw(
                    handle,
                    *filter_method,
                    *bounds,
                    pixels,
                    transform,
                    clip_mask,
                );
            }
            #[cfg(not(feature = "image"))]
            Primitive::Image { .. } => {
                log::warn!(
                    "Unsupported primitive in `iced_tiny_skia`: {primitive:?}",
                );
            }
            #[cfg(feature = "svg")]
            Primitive::Svg {
                handle,
                bounds,
                color,
            } => {
                let physical_bounds = (*bounds + translation) * scale_factor;

                if !clip_bounds.intersects(&physical_bounds) {
                    return;
                }

                let clip_mask = (!physical_bounds.is_within(&clip_bounds))
                    .then_some(clip_mask as &_);

                self.vector_pipeline.draw(
                    handle,
                    *color,
                    (*bounds + translation) * scale_factor,
                    pixels,
                    clip_mask,
                );
            }
            #[cfg(not(feature = "svg"))]
            Primitive::Svg { .. } => {
                log::warn!(
                    "Unsupported primitive in `iced_tiny_skia`: {primitive:?}",
                );
            }
            Primitive::Custom(primitive::Custom::Fill {
                path,
                paint,
                rule,
            }) => {
                let bounds = path.bounds();

                let physical_bounds = (Rectangle {
                    x: bounds.x(),
                    y: bounds.y(),
                    width: bounds.width(),
                    height: bounds.height(),
                } + translation)
                    * scale_factor;

                if !clip_bounds.intersects(&physical_bounds) {
                    return;
                }

                let clip_mask = (!physical_bounds.is_within(&clip_bounds))
                    .then_some(clip_mask as &_);

                pixels.fill_path(
                    path,
                    paint,
                    *rule,
                    tiny_skia::Transform::from_translate(
                        translation.x,
                        translation.y,
                    )
                    .post_scale(scale_factor, scale_factor),
                    clip_mask,
                );
            }
            Primitive::Custom(primitive::Custom::Stroke {
                path,
                paint,
                stroke,
            }) => {
                let bounds = path.bounds();

                let physical_bounds = (Rectangle {
                    x: bounds.x(),
                    y: bounds.y(),
                    width: bounds.width(),
                    height: bounds.height(),
                } + translation)
                    * scale_factor;

                if !clip_bounds.intersects(&physical_bounds) {
                    return;
                }

                let clip_mask = (!physical_bounds.is_within(&clip_bounds))
                    .then_some(clip_mask as &_);

                pixels.stroke_path(
                    path,
                    paint,
                    stroke,
                    tiny_skia::Transform::from_translate(
                        translation.x,
                        translation.y,
                    )
                    .post_scale(scale_factor, scale_factor),
                    clip_mask,
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
                );
            }
            Primitive::Clip { bounds, content } => {
                let bounds = (*bounds + translation) * scale_factor;

                if bounds == clip_bounds {
                    self.draw_primitive(
                        content,
                        pixels,
                        clip_mask,
                        bounds,
                        scale_factor,
                        translation,
                    );
                } else if let Some(bounds) = clip_bounds.intersection(&bounds) {
                    if bounds.x + bounds.width <= 0.0
                        || bounds.y + bounds.height <= 0.0
                        || bounds.x as u32 >= pixels.width()
                        || bounds.y as u32 >= pixels.height()
                        || bounds.width <= 1.0
                        || bounds.height <= 1.0
                    {
                        return;
                    }

                    adjust_clip_mask(clip_mask, bounds);

                    self.draw_primitive(
                        content,
                        pixels,
                        clip_mask,
                        bounds,
                        scale_factor,
                        translation,
                    );

                    adjust_clip_mask(clip_mask, clip_bounds);
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
                );
            }
        }
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self::new()
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

    if bottom_left > 0.0 {
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

fn adjust_clip_mask(clip_mask: &mut tiny_skia::Mask, bounds: Rectangle) {
    clip_mask.clear();

    let path = {
        let mut builder = tiny_skia::PathBuilder::new();
        builder.push_rect(
            tiny_skia::Rect::from_xywh(
                bounds.x,
                bounds.y,
                bounds.width,
                bounds.height,
            )
            .unwrap(),
        );

        builder.finish().unwrap()
    };

    clip_mask.fill_path(
        &path,
        tiny_skia::FillRule::EvenOdd,
        false,
        tiny_skia::Transform::default(),
    );
}

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let x = ((x - a) / (b - a)).clamp(0.0, 1.0);

    x * x * (3.0 - 2.0 * x)
}

fn rounded_box_sdf(to_center: Vector, size: Size, radii: &[f32]) -> f32 {
    let radius = match (to_center.x > 0.0, to_center.y > 0.0) {
        (true, true) => radii[2],
        (true, false) => radii[1],
        (false, true) => radii[3],
        (false, false) => radii[0],
    };

    let x = (to_center.x.abs() - size.width() + radius).max(0.0);
    let y = (to_center.y.abs() - size.height() + radius).max(0.0);

    (x.powf(2.0) + y.powf(2.0)).sqrt() - radius
}

impl iced_graphics::Backend for Backend {
    type Primitive = primitive::Custom;
}

impl backend::Text for Backend {
    fn load_font(&mut self, font: Cow<'static, [u8]>) {
        self.text_pipeline.load_font(font);
    }
}

#[cfg(feature = "image")]
impl backend::Image for Backend {
    fn dimensions(
        &self,
        handle: &crate::core::image::Handle,
    ) -> crate::core::Size<u32> {
        self.raster_pipeline.dimensions(handle)
    }
}

#[cfg(feature = "svg")]
impl backend::Svg for Backend {
    fn viewport_dimensions(
        &self,
        handle: &crate::core::svg::Handle,
    ) -> crate::core::Size<u32> {
        self.vector_pipeline.viewport_dimensions(handle)
    }
}
