use crate::backend::{Backend, CpuStorage, FONT_SYSTEM};

use cosmic_text::{AttrsList, BufferLine, SwashContent};
use iced_graphics::alignment::{Horizontal, Vertical};
#[cfg(feature = "svg")]
use iced_graphics::image::vector;
use iced_graphics::{Background, Gradient, Primitive};
use raqote::{
    DrawOptions, DrawTarget, Image, IntPoint, IntRect, PathBuilder,
    SolidSource, Source, StrokeStyle, Transform,
};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::cmp;
use swbuf::GraphicsContext;

// A software rendering surface
pub struct Surface {
    context: GraphicsContext,
    width: u32,
    height: u32,
    buffer: Vec<u32>,
}

impl Surface {
    pub(crate) fn new<W: HasRawWindowHandle + HasRawDisplayHandle>(
        window: &W,
    ) -> Self {
        let context = match unsafe { GraphicsContext::new(window, window) } {
            Ok(ok) => ok,
            Err(err) => panic!("failed to create swbuf context: {}", err),
        };
        Surface {
            context,
            width: 0,
            height: 0,
            buffer: Vec::new(),
        }
    }

    pub(crate) fn configure(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.buffer = vec![0; self.width as usize * self.height as usize];
    }

    pub(crate) fn present<Theme>(
        &mut self,
        renderer: &mut crate::Renderer<Theme>,
        background: iced_graphics::Color,
    ) {
        {
            let mut draw_target = DrawTarget::from_backing(
                self.width as i32,
                self.height as i32,
                self.buffer.as_mut_slice(),
            );

            draw_target.clear({
                let rgba = background.into_rgba8();
                SolidSource::from_unpremultiplied_argb(
                    rgba[3], rgba[0], rgba[1], rgba[2],
                )
            });

            let draw_options = DrawOptions {
                // Default to antialiasing off, enable it when necessary
                antialias: raqote::AntialiasMode::None,
                ..Default::default()
            };

            // Having at least one clip fixes some font rendering issues
            draw_target.push_clip_rect(IntRect::new(
                IntPoint::new(0, 0),
                IntPoint::new(self.width as i32, self.height as i32),
            ));

            renderer.with_primitives(|backend, primitives| {
                for primitive in primitives.iter() {
                    draw_primitive(
                        &mut draw_target,
                        &draw_options,
                        backend,
                        primitive,
                    );
                }
            });

            draw_target.pop_clip();
        }

        self.context.set_buffer(
            &self.buffer,
            self.width as u16,
            self.height as u16,
        );
    }
}

fn draw_primitive(
    draw_target: &mut DrawTarget<&mut [u32]>,
    draw_options: &DrawOptions,
    backend: &mut Backend,
    primitive: &Primitive,
) {
    match primitive {
        Primitive::None => (),
        Primitive::Group { primitives } => {
            for child in primitives.iter() {
                draw_primitive(draw_target, draw_options, backend, child);
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
            let cosmic_color = {
                let rgba8 = color.into_rgba8();
                cosmic_text::Color::rgba(rgba8[0], rgba8[1], rgba8[2], rgba8[3])
            };

            let (metrics, attrs) = backend.cosmic_metrics_attrs(*size, &font);

            /*
            // Debug bounds in green
            let mut pb = PathBuilder::new();
            pb.move_to(bounds.x, bounds.y);
            pb.line_to(bounds.x + bounds.width, bounds.y);
            pb.line_to(bounds.x + bounds.width, bounds.y + bounds.height);
            pb.line_to(bounds.x, bounds.y + bounds.height);
            pb.close();
            let path = pb.finish();
            draw_target.stroke(
                &path,
                &Source::Solid(SolidSource::from_unpremultiplied_argb(0xFF, 0, 0xFF, 0)),
                &StrokeStyle::default(),
                draw_options
            );
            */

            //TODO: improve implementation
            let mut buffer_line =
                BufferLine::new(content, AttrsList::new(attrs));
            let layout = buffer_line.layout(
                &FONT_SYSTEM,
                metrics.font_size,
                bounds.width as i32,
            );

            let mut line_y = match vertical_alignment {
                Vertical::Top => bounds.y as i32 + metrics.font_size,
                Vertical::Center => {
                    //TODO: why is this so weird?
                    bounds.y as i32 + metrics.font_size
                        - metrics.line_height * layout.len() as i32 / 2
                }
                Vertical::Bottom => {
                    //TODO: why is this so weird?
                    bounds.y as i32 + metrics.font_size
                        - metrics.line_height * layout.len() as i32
                }
            };

            let mut line_width = 0.0;
            for layout_line in layout.iter() {
                for glyph in layout_line.glyphs.iter() {
                    let max_x = if glyph.level.is_rtl() {
                        glyph.x - glyph.w
                    } else {
                        glyph.x + glyph.w
                    };
                    if max_x + 1.0 > line_width {
                        line_width = max_x + 1.0;
                    }
                }
            }

            let line_x = match horizontal_alignment {
                Horizontal::Left => bounds.x as i32,
                Horizontal::Center => {
                    //TODO: why is this so weird?
                    bounds.x as i32 - (line_width / 2.0) as i32
                }
                Horizontal::Right => {
                    //TODO: why is this so weird?
                    bounds.x as i32 - line_width as i32
                }
            };

            /*
            eprintln!(
                "{:?}: {}, {}, {}, {} in {:?} from font size {}, {:?}, {:?}",
                content,
                line_x, line_y,
                line_width, metrics.line_height,
                bounds,
                *size,
                horizontal_alignment,
                vertical_alignment
            );
            */

            for layout_line in layout.iter() {
                /*
                // Debug line placement in blue
                let mut pb = PathBuilder::new();
                pb.move_to(line_x as f32, line_y as f32 - metrics.font_size as f32);
                pb.line_to(line_x as f32 + line_width, line_y as f32 - metrics.font_size as f32);
                pb.line_to(line_x as f32 + line_width, line_y as f32 + metrics.line_height as f32 - metrics.font_size as f32);
                pb.line_to(line_x as f32, line_y as f32 + metrics.line_height as f32 - metrics.font_size as f32);
                pb.close();
                let path = pb.finish();
                draw_target.stroke(
                    &path,
                    &Source::Solid(SolidSource::from_unpremultiplied_argb(0xFF, 0, 0, 0xFF)),
                    &StrokeStyle::default(),
                    draw_options
                );
                */

                //TODO: also clip y, it does not seem to work though because
                // bounds.height < metrics.line_height * layout_lines.len()
                draw_target.push_clip_rect(IntRect::new(
                    IntPoint::new(line_x, 0),
                    IntPoint::new(
                        line_x
                            .checked_add(bounds.width as i32)
                            .unwrap_or_else(i32::max_value),
                        i32::max_value(),
                    ),
                ));

                for glyph in layout_line.glyphs.iter() {
                    let (cache_key, x_int, y_int) =
                        (glyph.cache_key, glyph.x_int, glyph.y_int);

                    let glyph_color = match glyph.color_opt {
                        Some(some) => some,
                        None => cosmic_color,
                    };

                    if let Some(image) =
                        backend.swash_cache.get_image(cache_key)
                    {
                        let x = line_x + x_int + image.placement.left;
                        let y = line_y + y_int + -image.placement.top;

                        /*
                        // Debug glyph placement in red
                        let mut pb = PathBuilder::new();
                        pb.move_to(x as f32, y as f32);
                        pb.line_to(x as f32 + image.placement.width as f32, y as f32);
                        pb.line_to(x as f32 + image.placement.width as f32, y as f32 + image.placement.height as f32);
                        pb.line_to(x as f32, y as f32 + image.placement.height as f32);
                        pb.close();
                        let path = pb.finish();
                        draw_target.stroke(
                            &path,
                            &Source::Solid(SolidSource::from_unpremultiplied_argb(0xFF, 0xFF, 0, 0)),
                            &StrokeStyle::default(),
                            draw_options
                        );
                        */

                        let mut image_data = Vec::with_capacity(
                            image.placement.height as usize
                                * image.placement.width as usize,
                        );
                        match image.content {
                            SwashContent::Mask => {
                                let mut i = 0;
                                for _off_y in 0..image.placement.height as i32 {
                                    for _off_x in
                                        0..image.placement.width as i32
                                    {
                                        //TODO: blend base alpha?
                                        image_data.push(
                                            SolidSource::from_unpremultiplied_argb(
                                                image.data[i],
                                                glyph_color.r(),
                                                glyph_color.g(),
                                                glyph_color.b(),
                                            ).to_u32()
                                        );
                                        i += 1;
                                    }
                                }
                            }
                            SwashContent::Color => {
                                let mut i = 0;
                                for _off_y in 0..image.placement.height as i32 {
                                    for _off_x in
                                        0..image.placement.width as i32
                                    {
                                        //TODO: blend base alpha?
                                        image_data.push(
                                            SolidSource::from_unpremultiplied_argb(
                                                image.data[i + 3],
                                                image.data[i + 0],
                                                image.data[i + 1],
                                                image.data[i + 2],
                                            ).to_u32()
                                        );
                                        i += 4;
                                    }
                                }
                            }
                            SwashContent::SubpixelMask => {
                                eprintln!("Content::SubpixelMask");
                            }
                        }

                        if !image_data.is_empty() {
                            draw_target.draw_image_at(
                                x as f32,
                                y as f32,
                                &Image {
                                    width: image.placement.width as i32,
                                    height: image.placement.height as i32,
                                    data: &image_data,
                                },
                                &draw_options,
                            );
                        }
                    }
                }

                draw_target.pop_clip();

                line_y += metrics.line_height;
            }
        }
        Primitive::Quad {
            bounds,
            background,
            border_radius,
            border_width,
            border_color,
        } => {
            // Ensure radius is not too large
            let clamp_radius = |radius: f32| -> f32 {
                if radius > bounds.width / 2.0 {
                    return bounds.width / 2.0;
                }

                if radius > bounds.height / 2.0 {
                    return bounds.height / 2.0;
                }

                radius
            };

            let mut pb = PathBuilder::new();

            let top_left = clamp_radius(border_radius[0]);
            let top_right = clamp_radius(border_radius[1]);
            let bottom_right = clamp_radius(border_radius[2]);
            let bottom_left = clamp_radius(border_radius[3]);

            // Move to top left corner at start of clockwise arc
            pb.move_to(bounds.x, bounds.y + top_left);
            pb.arc(
                bounds.x + top_left,
                bounds.y + top_left,
                top_left,
                180.0f32.to_radians(),
                90.0f32.to_radians(),
            );

            // Move to top right corner at start of clockwise arc
            pb.line_to(bounds.x + bounds.width - top_right, bounds.y);
            pb.arc(
                bounds.x + bounds.width - top_right,
                bounds.y + top_right,
                top_right,
                270.0f32.to_radians(),
                90.0f32.to_radians(),
            );

            // Move to bottom right corner at start of clockwise arc
            pb.line_to(
                bounds.x + bounds.width,
                bounds.y + bounds.height - bottom_right,
            );
            pb.arc(
                bounds.x + bounds.width - bottom_right,
                bounds.y + bounds.height - bottom_right,
                bottom_right,
                0.0f32.to_radians(),
                90.0f32.to_radians(),
            );

            // Move to bottom left corner at start of clockwise arc
            pb.line_to(bounds.x + bottom_left, bounds.y + bounds.height);
            pb.arc(
                bounds.x + bottom_left,
                bounds.y + bounds.height - bottom_left,
                bottom_left,
                90.0f32.to_radians(),
                90.0f32.to_radians(),
            );

            // Close and finish path
            pb.close();
            let path = pb.finish();

            let background_source = match background {
                Background::Color(color) => {
                    let rgba = color.into_rgba8();
                    Source::Solid(SolidSource::from_unpremultiplied_argb(
                        rgba[3], rgba[0], rgba[1], rgba[2],
                    ))
                }
            };

            draw_target.fill(
                &path,
                &background_source,
                &DrawOptions {
                    // Anti-alias rounded rectangles
                    antialias: raqote::AntialiasMode::Gray,
                    ..*draw_options
                },
            );

            let border_source = {
                let rgba = border_color.into_rgba8();
                Source::Solid(SolidSource::from_unpremultiplied_argb(
                    rgba[3], rgba[0], rgba[1], rgba[2],
                ))
            };

            let style = StrokeStyle {
                width: *border_width,
                ..Default::default()
            };

            draw_target.stroke(
                &path,
                &border_source,
                &style,
                &DrawOptions {
                    // Anti-alias rounded rectangles
                    antialias: raqote::AntialiasMode::Gray,
                    ..*draw_options
                },
            );
        }
        Primitive::Image { handle, bounds } => {
            #[cfg(feature = "image")]
            match backend.raster_cache.borrow_mut().upload(
                handle,
                &mut (),
                &mut CpuStorage,
            ) {
                Some(entry) => {
                    draw_target.draw_image_with_size_at(
                        bounds.width,
                        bounds.height,
                        bounds.x,
                        bounds.y,
                        &Image {
                            width: entry.size.width as i32,
                            height: entry.size.height as i32,
                            data: &entry.data,
                        },
                        draw_options,
                    );
                }
                None => (),
            }
        }
        Primitive::Svg {
            handle,
            bounds,
            color,
        } => {
            #[cfg(feature = "svg")]
            match backend.vector_cache.borrow_mut().upload(
                handle,
                color.clone(),
                [bounds.width, bounds.height],
                1.0, /*TODO: what should scale be?*/
                &mut (),
                &mut CpuStorage,
            ) {
                Some(entry) => {
                    draw_target.draw_image_with_size_at(
                        bounds.width,
                        bounds.height,
                        bounds.x,
                        bounds.y,
                        &Image {
                            width: entry.size.width as i32,
                            height: entry.size.height as i32,
                            data: &entry.data,
                        },
                        draw_options,
                    );
                }
                None => (),
            }
        }
        Primitive::Clip { bounds, content } => {
            draw_target.push_clip_rect(IntRect::new(
                IntPoint::new(bounds.x as i32, bounds.y as i32),
                IntPoint::new(
                    (bounds.x + bounds.width) as i32,
                    (bounds.y + bounds.height) as i32,
                ),
            ));
            draw_primitive(draw_target, draw_options, backend, &content);
            draw_target.pop_clip();
        }
        Primitive::Translate {
            translation,
            content,
        } => {
            draw_target.set_transform(&Transform::translation(
                translation.x,
                translation.y,
            ));
            draw_primitive(draw_target, draw_options, backend, &content);
            draw_target.set_transform(&Transform::identity());
        }
        Primitive::GradientMesh {
            buffers,
            size,
            gradient,
        } => {
            let source = match gradient {
                Gradient::Linear(linear) => {
                    let mut stops = Vec::new();
                    for stop in linear.color_stops.iter() {
                        let rgba8 = stop.color.into_rgba8();
                        stops.push(raqote::GradientStop {
                            position: stop.offset,
                            color: raqote::Color::new(
                                rgba8[3], rgba8[0], rgba8[1], rgba8[2],
                            ),
                        });
                    }
                    Source::new_linear_gradient(
                        raqote::Gradient { stops },
                        raqote::Point::new(linear.start.x, linear.start.y),
                        raqote::Point::new(linear.end.x, linear.end.y),
                        raqote::Spread::Pad, /*TODO: which spread?*/
                    )
                }
            };

            /*
            draw_target.push_clip_rect(IntRect::new(
                IntPoint::new(0, 0),
                IntPoint::new(size.width as i32, size.height as i32),
            ));
            */

            let mut pb = PathBuilder::new();

            for indices in buffers.indices.chunks_exact(3) {
                let a = &buffers.vertices[indices[0] as usize];
                let b = &buffers.vertices[indices[1] as usize];
                let c = &buffers.vertices[indices[2] as usize];

                pb.move_to(a.position[0], a.position[1]);
                pb.line_to(b.position[0], b.position[1]);
                pb.line_to(c.position[0], c.position[1]);
                pb.close();
            }

            let path = pb.finish();
            draw_target.fill(&path, &source, draw_options);

            /*
            draw_target.pop_clip();
            */
        }
        Primitive::SolidMesh { buffers, size } => {
            fn undo_linear_component(linear: f32) -> f32 {
                if linear < 0.0031308 {
                    linear * 12.92
                } else {
                    1.055 * linear.powf(1.0 / 2.4) - 0.055
                }
            }

            fn linear_to_rgba8(color: &[f32; 4]) -> [u8; 4] {
                let r = undo_linear_component(color[0]) * 255.0;
                let g = undo_linear_component(color[1]) * 255.0;
                let b = undo_linear_component(color[2]) * 255.0;
                let a = color[3] * 255.0;
                [
                    cmp::max(0, cmp::min(255, r.round() as i32)) as u8,
                    cmp::max(0, cmp::min(255, g.round() as i32)) as u8,
                    cmp::max(0, cmp::min(255, b.round() as i32)) as u8,
                    cmp::max(0, cmp::min(255, a.round() as i32)) as u8,
                ]
            }

            /*
            draw_target.push_clip_rect(IntRect::new(
                IntPoint::new(0, 0),
                IntPoint::new(size.width as i32, size.height as i32),
            ));
            */

            for indices in buffers.indices.chunks_exact(3) {
                let a = &buffers.vertices[indices[0] as usize];
                let b = &buffers.vertices[indices[1] as usize];
                let c = &buffers.vertices[indices[2] as usize];

                let mut pb = PathBuilder::new();
                pb.move_to(a.position[0], a.position[1]);
                pb.line_to(b.position[0], b.position[1]);
                pb.line_to(c.position[0], c.position[1]);
                pb.close();

                // TODO: Each vertice has its own separate color.
                let rgba8 = linear_to_rgba8(&a.color);
                let source =
                    Source::Solid(SolidSource::from_unpremultiplied_argb(
                        rgba8[3], rgba8[0], rgba8[1], rgba8[2],
                    ));

                let path = pb.finish();
                draw_target.fill(&path, &source, draw_options);
            }

            /*
            draw_target.pop_clip();
            */
        }
        Primitive::Cached { cache } => {
            draw_primitive(draw_target, draw_options, backend, &cache);
        }
    }
}
