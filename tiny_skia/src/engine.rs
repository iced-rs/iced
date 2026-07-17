use crate::Primitive;
use crate::core::renderer::Quad;
use crate::core::{Background, Color, Gradient, Rectangle, Size, Transformation, Vector};
use crate::graphics::{Image, Text};
use crate::text;

#[derive(Debug)]
pub struct Engine {
    text_pipeline: text::Pipeline,

    #[cfg(feature = "image")]
    pub(crate) raster_pipeline: crate::raster::Pipeline,
    #[cfg(feature = "svg")]
    pub(crate) vector_pipeline: crate::vector::Pipeline,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            text_pipeline: text::Pipeline::new(),
            #[cfg(feature = "image")]
            raster_pipeline: crate::raster::Pipeline::new(),
            #[cfg(feature = "svg")]
            vector_pipeline: crate::vector::Pipeline::new(),
        }
    }

    pub fn draw_quad(
        &mut self,
        quad: &Quad,
        background: &Background,
        transformation: Transformation,
        pixels: &mut tiny_skia::PixmapMut<'_>,
        clip_mask: &mut tiny_skia::Mask,
        clip_bounds: Rectangle,
    ) {
        let physical_bounds = quad.bounds * transformation;

        if !clip_bounds.intersects(&physical_bounds) {
            return;
        }

        let transform = into_transform(transformation);

        let border_colors = [
            quad.border.top.color.unwrap_or(quad.border.color),
            quad.border.right.color.unwrap_or(quad.border.color),
            quad.border.bottom.color.unwrap_or(quad.border.color),
            quad.border.left.color.unwrap_or(quad.border.color),
        ];
        let mut border_widths = [
            quad.border.top.width.unwrap_or(quad.border.width).max(0.0),
            quad.border
                .right
                .width
                .unwrap_or(quad.border.width)
                .max(0.0),
            quad.border
                .bottom
                .width
                .unwrap_or(quad.border.width)
                .max(0.0),
            quad.border.left.width.unwrap_or(quad.border.width).max(0.0),
        ];

        let horizontal = border_widths[1] + border_widths[3];
        if horizontal > quad.bounds.width {
            let factor = quad.bounds.width / horizontal;
            border_widths[1] *= factor;
            border_widths[3] *= factor;
        }
        let vertical = border_widths[0] + border_widths[2];
        if vertical > quad.bounds.height {
            let factor = quad.bounds.height / vertical;
            border_widths[0] *= factor;
            border_widths[2] *= factor;
        }

        let uniform_border = border_colors.iter().all(|color| *color == border_colors[0])
            && border_widths.iter().all(|width| *width == border_widths[0]);
        let border_width = border_widths[0];

        let mut fill_border_radius = <[f32; 4]>::from(quad.border.radius);

        for radius in &mut fill_border_radius {
            *radius = (*radius)
                .min(quad.bounds.width / 2.0)
                .min(quad.bounds.height / 2.0);
        }

        let path = rounded_rectangle(quad.bounds, fill_border_radius);

        let shadow = quad.shadow;

        if shadow.color.a > 0.0 {
            let shadow_bounds = Rectangle {
                x: quad.bounds.x + shadow.offset.x - shadow.blur_radius,
                y: quad.bounds.y + shadow.offset.y - shadow.blur_radius,
                width: quad.bounds.width + shadow.blur_radius * 2.0,
                height: quad.bounds.height + shadow.blur_radius * 2.0,
            } * transformation;

            let radii = fill_border_radius
                .into_iter()
                .map(|radius| radius * transformation.scale_factor())
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
                .flat_map(|y| (x..x + width).map(move |x| (x as f32, y as f32)))
                .filter_map(|(x, y)| {
                    tiny_skia::Size::from_wh(half_width, half_height).map(|size| {
                        let shadow_distance = rounded_box_sdf(
                            Vector::new(
                                x - physical_bounds.position().x
                                    - (shadow.offset.x * transformation.scale_factor())
                                    - half_width,
                                y - physical_bounds.position().y
                                    - (shadow.offset.y * transformation.scale_factor())
                                    - half_height,
                            ),
                            size,
                            &radii,
                        )
                        .max(0.0);
                        let shadow_alpha = 1.0
                            - smoothstep(
                                -shadow.blur_radius * transformation.scale_factor(),
                                shadow.blur_radius * transformation.scale_factor(),
                                shadow_distance,
                            );

                        let mut color = into_color(shadow.color);
                        color.apply_opacity(shadow_alpha);

                        color.to_color_u8().premultiply()
                    })
                })
                .collect();

            if let Some(pixmap) = tiny_skia::IntSize::from_wh(width, height)
                .and_then(|size| tiny_skia::Pixmap::from_vec(bytemuck::cast_vec(colors), size))
            {
                pixels.draw_pixmap(
                    x as i32,
                    y as i32,
                    pixmap.as_ref(),
                    &tiny_skia::PixmapPaint::default(),
                    tiny_skia::Transform::default(),
                    Some(clip_mask),
                );
            }
        }

        let clip_mask = (!physical_bounds.is_within(&clip_bounds)).then_some(clip_mask as &_);

        pixels.fill_path(
            &path,
            &tiny_skia::Paint {
                shader: match background {
                    Background::Color(color) => tiny_skia::Shader::SolidColor(into_color(*color)),
                    Background::Gradient(Gradient::Linear(linear)) => {
                        let (start, end) = quad.bounds.chord(linear.angle);

                        let stops: Vec<tiny_skia::GradientStop> = linear
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
                                vec![tiny_skia::GradientStop::new(0.0, tiny_skia::Color::BLACK)]
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

        if uniform_border && border_width > 0.0 {
            // Border path is offset by half the border width
            let border_bounds = Rectangle {
                x: quad.bounds.x + border_width / 2.0,
                y: quad.bounds.y + border_width / 2.0,
                width: quad.bounds.width - border_width,
                height: quad.bounds.height - border_width,
            };

            // Make sure the border radius is correct
            let mut border_radius = <[f32; 4]>::from(quad.border.radius);
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
                let border_path = rounded_rectangle(border_bounds, border_radius);

                pixels.stroke_path(
                    &border_path,
                    &tiny_skia::Paint {
                        shader: tiny_skia::Shader::SolidColor(into_color(quad.border.color)),
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
                let mut temp_pixmap =
                    tiny_skia::Pixmap::new(quad.bounds.width as u32, quad.bounds.height as u32)
                        .unwrap();

                let mut quad_mask =
                    tiny_skia::Mask::new(quad.bounds.width as u32, quad.bounds.height as u32)
                        .unwrap();

                let zero_bounds = Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: quad.bounds.width,
                    height: quad.bounds.height,
                };
                let path = rounded_rectangle(zero_bounds, fill_border_radius);

                quad_mask.fill_path(&path, tiny_skia::FillRule::EvenOdd, true, transform);
                let path_bounds = Rectangle {
                    x: border_width / 2.0,
                    y: border_width / 2.0,
                    width: quad.bounds.width - border_width,
                    height: quad.bounds.height - border_width,
                };

                let border_radius_path = rounded_rectangle(path_bounds, border_radius);

                temp_pixmap.stroke_path(
                    &border_radius_path,
                    &tiny_skia::Paint {
                        shader: tiny_skia::Shader::SolidColor(into_color(quad.border.color)),
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
                    quad.bounds.x as i32,
                    quad.bounds.y as i32,
                    temp_pixmap.as_ref(),
                    &tiny_skia::PixmapPaint::default(),
                    transform,
                    clip_mask,
                );
            }
        }

        if !uniform_border && border_widths.into_iter().any(|width| width > 0.0) {
            draw_asymmetric_border(
                quad.bounds,
                physical_bounds,
                fill_border_radius,
                border_widths,
                border_colors,
                transform,
                pixels,
                clip_mask,
            );
        }
    }

    pub fn draw_text(
        &mut self,
        text: &Text,
        transformation: Transformation,
        pixels: &mut tiny_skia::PixmapMut<'_>,
        clip_mask: &mut tiny_skia::Mask,
        clip_bounds: Rectangle,
    ) {
        match text {
            Text::Paragraph {
                paragraph,
                position,
                color,
                clip_bounds: local_clip_bounds,
                transformation: local_transformation,
            } => {
                let transformation = transformation * *local_transformation;
                let Some(clip_bounds) =
                    clip_bounds.intersection(&(*local_clip_bounds * transformation))
                else {
                    return;
                };

                let physical_bounds =
                    Rectangle::new(*position, paragraph.min_bounds) * transformation;

                if !clip_bounds.intersects(&physical_bounds) {
                    return;
                }

                let clip_mask = match physical_bounds.is_within(&clip_bounds) {
                    true => None,
                    false => {
                        adjust_clip_mask(clip_mask, clip_bounds);
                        Some(clip_mask as &_)
                    }
                };

                self.text_pipeline.draw_paragraph(
                    paragraph,
                    *position,
                    *color,
                    pixels,
                    clip_mask,
                    transformation,
                );
            }
            Text::Editor {
                editor,
                position,
                color,
                clip_bounds: local_clip_bounds,
                transformation: local_transformation,
            } => {
                let transformation = transformation * *local_transformation;

                let Some(clip_bounds) =
                    clip_bounds.intersection(&(*local_clip_bounds * transformation))
                else {
                    return;
                };

                adjust_clip_mask(clip_mask, clip_bounds);

                self.text_pipeline.draw_editor(
                    editor,
                    *position,
                    *color,
                    pixels,
                    Some(clip_mask),
                    transformation,
                );
            }
            Text::Cached {
                content,
                bounds,
                color,
                size,
                line_height,
                font,
                align_x,
                align_y,
                shaping,
                wrapping,
                ellipsis,
                clip_bounds: local_clip_bounds,
            } => {
                let physical_bounds = *local_clip_bounds * transformation;

                if !clip_bounds.intersects(&physical_bounds) {
                    return;
                }

                let clip_mask = match physical_bounds.is_within(&clip_bounds) {
                    true => None,
                    false => {
                        adjust_clip_mask(clip_mask, clip_bounds);
                        Some(clip_mask as &_)
                    }
                };

                self.text_pipeline.draw_cached(
                    content,
                    *bounds,
                    *color,
                    *size,
                    *line_height,
                    *font,
                    *align_x,
                    *align_y,
                    *shaping,
                    *wrapping,
                    *ellipsis,
                    pixels,
                    clip_mask,
                    transformation,
                );
            }
            Text::Raw {
                raw,
                transformation: local_transformation,
            } => {
                let Some(buffer) = raw.buffer.upgrade() else {
                    return;
                };

                let transformation = transformation * *local_transformation;
                let (width, height) = buffer.size();

                let physical_bounds = Rectangle::new(
                    raw.position,
                    Size::new(
                        width.unwrap_or(clip_bounds.width),
                        height.unwrap_or(clip_bounds.height),
                    ),
                ) * transformation;

                if !clip_bounds.intersects(&physical_bounds) {
                    return;
                }

                let clip_mask =
                    (!physical_bounds.is_within(&clip_bounds)).then_some(clip_mask as &_);

                self.text_pipeline.draw_raw(
                    &buffer,
                    raw.position,
                    raw.color,
                    pixels,
                    clip_mask,
                    transformation,
                );
            }
        }
    }

    pub fn draw_primitive(
        &mut self,
        primitive: &Primitive,
        transformation: Transformation,
        pixels: &mut tiny_skia::PixmapMut<'_>,
        clip_mask: &mut tiny_skia::Mask,
        clip_bounds: Rectangle,
    ) {
        match primitive {
            Primitive::Fill { path, paint, rule } => {
                let physical_bounds = {
                    let bounds = path.bounds();

                    Rectangle {
                        x: bounds.x(),
                        y: bounds.y(),
                        width: bounds.width(),
                        height: bounds.height(),
                    } * transformation
                };

                if !clip_bounds.intersects(&physical_bounds) {
                    return;
                }

                let clip_mask =
                    (!physical_bounds.is_within(&clip_bounds)).then_some(clip_mask as &_);

                pixels.fill_path(
                    path,
                    paint,
                    *rule,
                    into_transform(transformation),
                    clip_mask,
                );
            }
            Primitive::Stroke {
                path,
                paint,
                stroke,
            } => {
                let physical_bounds = {
                    let bounds = path.bounds();

                    Rectangle {
                        x: bounds.x() - stroke.width / 2.0,
                        y: bounds.y() - stroke.width / 2.0,
                        width: bounds.width() + stroke.width,
                        height: bounds.height() + stroke.width,
                    } * transformation
                };

                if !clip_bounds.intersects(&physical_bounds) {
                    return;
                }

                let clip_mask =
                    (!physical_bounds.is_within(&clip_bounds)).then_some(clip_mask as &_);

                pixels.stroke_path(
                    path,
                    paint,
                    stroke,
                    into_transform(transformation),
                    clip_mask,
                );
            }
        }
    }

    pub fn draw_image(
        &mut self,
        image: &Image,
        _transformation: Transformation,
        _pixels: &mut tiny_skia::PixmapMut<'_>,
        _clip_mask: &mut tiny_skia::Mask,
        _clip_bounds: Rectangle,
    ) {
        match image {
            #[cfg(feature = "image")]
            Image::Raster {
                image,
                bounds,
                clip_bounds: local_clip_bounds,
            } => {
                let physical_bounds = *local_clip_bounds * _transformation;

                let Some(clip_bounds) = physical_bounds.intersection(&_clip_bounds) else {
                    return;
                };

                // TODO: Border radius
                adjust_clip_mask(_clip_mask, clip_bounds);

                let center = bounds.center();
                let radians = f32::from(image.rotation);

                let transform = into_transform(_transformation).pre_rotate_at(
                    radians.to_degrees(),
                    center.x,
                    center.y,
                );

                self.raster_pipeline.draw(
                    &image.handle,
                    image.filter_method,
                    *bounds,
                    image.opacity,
                    _pixels,
                    transform,
                    Some(_clip_mask),
                );
            }
            #[cfg(feature = "svg")]
            Image::Vector { svg, bounds, .. } => {
                let physical_bounds = *bounds * _transformation;

                if !_clip_bounds.intersects(&physical_bounds) {
                    return;
                }

                let clip_mask =
                    (!physical_bounds.is_within(&_clip_bounds)).then_some(_clip_mask as &_);

                let center = physical_bounds.center();
                let radians = f32::from(svg.rotation);

                let transform = into_transform(_transformation).post_rotate_at(
                    radians.to_degrees(),
                    center.x,
                    center.y,
                );

                self.vector_pipeline.draw(
                    &svg.handle,
                    svg.color,
                    *bounds,
                    svg.opacity,
                    _pixels,
                    transform,
                    clip_mask,
                );
            }
            #[cfg(not(feature = "image"))]
            Image::Raster { .. } => {
                log::warn!("Unsupported primitive in `iced_tiny_skia`: {image:?}",);
            }
            #[cfg(not(feature = "svg"))]
            Image::Vector { .. } => {
                log::warn!("Unsupported primitive in `iced_tiny_skia`: {image:?}",);
            }
        }
    }

    pub fn trim(&mut self) {
        self.text_pipeline.trim_cache();

        #[cfg(feature = "image")]
        self.raster_pipeline.trim_cache();

        #[cfg(feature = "svg")]
        self.vector_pipeline.trim_cache();
    }
}

pub fn into_color(color: Color) -> tiny_skia::Color {
    tiny_skia::Color::from_rgba(color.b, color.g, color.r, color.a)
        .expect("Convert color from iced to tiny_skia")
}

fn into_transform(transformation: Transformation) -> tiny_skia::Transform {
    let translation = transformation.translation();

    tiny_skia::Transform {
        sx: transformation.scale_factor(),
        kx: 0.0,
        ky: 0.0,
        sy: transformation.scale_factor(),
        tx: translation.x,
        ty: translation.y,
    }
}

fn draw_asymmetric_border(
    bounds: Rectangle,
    physical_bounds: Rectangle,
    outer_radius: [f32; 4],
    widths: [f32; 4],
    colors: [Color; 4],
    transform: tiny_skia::Transform,
    pixels: &mut tiny_skia::PixmapMut<'_>,
    clip_mask: Option<&tiny_skia::Mask>,
) {
    // Render only the transformed quad bounds. Allocating masks at the size of
    // the entire destination surface made every asymmetric border temporarily
    // consume a full render target.
    let x = (physical_bounds.x.floor() - 1.0)
        .max(0.0)
        .min(pixels.width() as f32) as u32;
    let y = (physical_bounds.y.floor() - 1.0)
        .max(0.0)
        .min(pixels.height() as f32) as u32;
    let right = (physical_bounds.x + physical_bounds.width).ceil() + 1.0;
    let bottom = (physical_bounds.y + physical_bounds.height).ceil() + 1.0;
    let right = right.max(0.0).min(pixels.width() as f32) as u32;
    let bottom = bottom.max(0.0).min(pixels.height() as f32) as u32;

    if x >= right || y >= bottom {
        return;
    }

    let width = right - x;
    let height = bottom - y;
    let transform = tiny_skia::Transform {
        tx: transform.tx - x as f32,
        ty: transform.ty - y as f32,
        ..transform
    };
    let [top, right, bottom, left] = widths;
    let inner_bounds = Rectangle {
        x: bounds.x + left,
        y: bounds.y + top,
        width: bounds.width - left - right,
        height: bounds.height - top - bottom,
    };

    let mut ring_mask = tiny_skia::Mask::new(width, height).expect("Create border mask");
    ring_mask.fill_path(
        &rounded_rectangle(bounds, outer_radius),
        tiny_skia::FillRule::Winding,
        true,
        transform,
    );

    if inner_bounds.width > 0.0 && inner_bounds.height > 0.0 {
        let [top_left, top_right, bottom_right, bottom_left] = outer_radius;
        let inner_radius = [
            ((top_left - left).max(0.0), (top_left - top).max(0.0)),
            ((top_right - right).max(0.0), (top_right - top).max(0.0)),
            (
                (bottom_right - right).max(0.0),
                (bottom_right - bottom).max(0.0),
            ),
            (
                (bottom_left - left).max(0.0),
                (bottom_left - bottom).max(0.0),
            ),
        ];
        let mut inner_mask = tiny_skia::Mask::new(width, height).expect("Create inner border mask");
        inner_mask.fill_path(
            &rounded_rectangle_elliptical(inner_bounds, inner_radius),
            tiny_skia::FillRule::Winding,
            true,
            transform,
        );

        for (outer, inner) in ring_mask.data_mut().iter_mut().zip(inner_mask.data()) {
            *outer = (u16::from(*outer) * u16::from(255 - *inner) / 255) as u8;
        }
    }

    let x0 = bounds.x;
    let y0 = bounds.y;
    let r = bounds.x + bounds.width;
    let b = bounds.y + bounds.height;
    let side_paths = [
        quadrilateral_path([
            (x0, y0),
            (r, y0),
            (r - right, y0 + top),
            (x0 + left, y0 + top),
        ]),
        quadrilateral_path([
            (r, y0),
            (r, b),
            (r - right, b - bottom),
            (r - right, y0 + top),
        ]),
        quadrilateral_path([
            (r, b),
            (x0, b),
            (x0 + left, b - bottom),
            (r - right, b - bottom),
        ]),
        quadrilateral_path([
            (x0, b),
            (x0, y0),
            (x0 + left, y0 + top),
            (x0 + left, b - bottom),
        ]),
    ];

    let mut border = tiny_skia::Pixmap::new(width, height).expect("Create border pixmap");
    for (path, color) in side_paths.iter().zip(colors) {
        border.fill_path(
            path,
            &tiny_skia::Paint {
                shader: tiny_skia::Shader::SolidColor(into_color(color)),
                anti_alias: true,
                ..tiny_skia::Paint::default()
            },
            tiny_skia::FillRule::Winding,
            transform,
            Some(&ring_mask),
        );
    }

    pixels.draw_pixmap(
        x as i32,
        y as i32,
        border.as_ref(),
        &tiny_skia::PixmapPaint::default(),
        tiny_skia::Transform::identity(),
        clip_mask,
    );
}

fn quadrilateral_path(points: [(f32, f32); 4]) -> tiny_skia::Path {
    let mut builder = tiny_skia::PathBuilder::new();
    builder.move_to(points[0].0, points[0].1);
    for (x, y) in points.into_iter().skip(1) {
        builder.line_to(x, y);
    }
    builder.close();
    builder.finish().expect("Build border side")
}

fn rounded_rectangle(bounds: Rectangle, border_radius: [f32; 4]) -> tiny_skia::Path {
    let [top_left, top_right, bottom_right, bottom_left] = border_radius;

    if top_left == 0.0 && top_right == 0.0 && bottom_right == 0.0 && bottom_left == 0.0 {
        return tiny_skia::PathBuilder::from_rect(
            tiny_skia::Rect::from_xywh(bounds.x, bounds.y, bounds.width, bounds.height)
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

fn rounded_rectangle_elliptical(
    bounds: Rectangle,
    border_radius: [(f32, f32); 4],
) -> tiny_skia::Path {
    let [top_left, top_right, bottom_right, bottom_left] = border_radius;
    let clamp = |(x, y): (f32, f32)| (x.min(bounds.width / 2.0), y.min(bounds.height / 2.0));
    let (tlx, tly) = clamp(top_left);
    let (trx, try_) = clamp(top_right);
    let (brx, bry) = clamp(bottom_right);
    let (blx, bly) = clamp(bottom_left);

    let mut builder = tiny_skia::PathBuilder::new();
    builder.move_to(bounds.x + tlx, bounds.y);
    builder.line_to(bounds.x + bounds.width - trx, bounds.y);
    arc_to_ellipse(
        &mut builder,
        bounds.x + bounds.width - trx,
        bounds.y,
        bounds.x + bounds.width,
        bounds.y + try_,
        trx,
        try_,
    );
    builder.line_to(bounds.x + bounds.width, bounds.y + bounds.height - bry);
    arc_to_ellipse(
        &mut builder,
        bounds.x + bounds.width,
        bounds.y + bounds.height - bry,
        bounds.x + bounds.width - brx,
        bounds.y + bounds.height,
        brx,
        bry,
    );
    builder.line_to(bounds.x + blx, bounds.y + bounds.height);
    arc_to_ellipse(
        &mut builder,
        bounds.x + blx,
        bounds.y + bounds.height,
        bounds.x,
        bounds.y + bounds.height - bly,
        blx,
        bly,
    );
    builder.line_to(bounds.x, bounds.y + tly);
    arc_to_ellipse(
        &mut builder,
        bounds.x,
        bounds.y + tly,
        bounds.x + tlx,
        bounds.y,
        tlx,
        tly,
    );
    builder.close();
    builder.finish().expect("Build asymmetric inner border")
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

fn arc_to_ellipse(
    path: &mut tiny_skia::PathBuilder,
    x_from: f32,
    y_from: f32,
    x_to: f32,
    y_to: f32,
    radius_x: f32,
    radius_y: f32,
) {
    if radius_x == 0.0 || radius_y == 0.0 {
        path.line_to(x_to, y_to);
        return;
    }

    let svg_arc = kurbo::SvgArc {
        from: kurbo::Point::new(f64::from(x_from), f64::from(y_from)),
        to: kurbo::Point::new(f64::from(x_to), f64::from(y_to)),
        radii: kurbo::Vec2::new(f64::from(radius_x), f64::from(radius_y)),
        x_rotation: 0.0,
        large_arc: false,
        sweep: true,
    };

    match kurbo::Arc::from_svg_arc(&svg_arc) {
        Some(arc) => arc.to_cubic_beziers(0.1, |p1, p2, p| {
            path.cubic_to(
                p1.x as f32,
                p1.y as f32,
                p2.x as f32,
                p2.y as f32,
                p.x as f32,
                p.y as f32,
            );
        }),
        None => path.line_to(x_to, y_to),
    }
}

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let x = ((x - a) / (b - a)).clamp(0.0, 1.0);

    x * x * (3.0 - 2.0 * x)
}

fn rounded_box_sdf(to_center: Vector, size: tiny_skia::Size, radii: &[f32]) -> f32 {
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

pub fn adjust_clip_mask(clip_mask: &mut tiny_skia::Mask, bounds: Rectangle) {
    clip_mask.clear();

    let path = tiny_skia::PathBuilder::from_rect(
        tiny_skia::Rect::from_xywh(bounds.x, bounds.y, bounds.width, bounds.height)
            .expect("Create clip rectangle"),
    );

    clip_mask.fill_path(
        &path,
        tiny_skia::FillRule::EvenOdd,
        false,
        tiny_skia::Transform::default(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::border::Side;
    use crate::core::{Border, Shadow};

    #[test]
    fn asymmetric_border_uses_each_side_and_allows_a_disabled_edge() {
        let mut engine = Engine::new();
        let mut pixmap = tiny_skia::Pixmap::new(32, 32).expect("Create pixmap");
        pixmap.fill(tiny_skia::Color::BLACK);
        let mut mask = tiny_skia::Mask::new(32, 32).expect("Create clip mask");
        let quad = Quad {
            bounds: Rectangle {
                x: 4.0,
                y: 4.0,
                width: 24.0,
                height: 24.0,
            },
            border: Border::default()
                .rounded(5)
                .width(3)
                .top(Side::default().color(Color::from_rgb(1.0, 0.0, 0.0)))
                .right(Side::default().color(Color::from_rgb(0.0, 1.0, 0.0)))
                .bottom(Side::default().width(0))
                .left(
                    Side::default()
                        .color(Color::from_rgb(0.0, 0.0, 1.0))
                        .width(5),
                ),
            shadow: Shadow::default(),
            snap: false,
        };

        engine.draw_quad(
            &quad,
            &Background::Color(Color::WHITE),
            Transformation::IDENTITY,
            &mut pixmap.as_mut(),
            &mut mask,
            Rectangle {
                x: 0.0,
                y: 0.0,
                width: 32.0,
                height: 32.0,
            },
        );

        assert_ne!(pixmap.pixel(16, 4), pixmap.pixel(27, 16));
        assert_eq!(pixmap.pixel(16, 27), pixmap.pixel(16, 16));
    }
}
