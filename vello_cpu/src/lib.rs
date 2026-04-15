#![allow(missing_docs)]
#![allow(dead_code)]

use iced_graphics as graphics;
use iced_graphics::core;

mod layer;
mod text;

#[cfg(feature = "image")]
mod raster;

#[cfg(feature = "svg")]
mod vector;

#[cfg(feature = "geometry")]
mod geometry;

use crate::core::border;
use crate::core::image;
use crate::core::renderer;
use crate::core::{Background, Color, Font, Gradient, Pixels, Rectangle, Size, Transformation};
use crate::graphics::compositor;
use crate::graphics::error;
use crate::graphics::mesh;
use crate::graphics::text::{Editor, Paragraph};
use crate::graphics::{Error, Shell, Text, Viewport};

use std::num::NonZeroU32;

pub struct Renderer {
    settings: renderer::Settings,
    layers: layer::Stack,
    text: text::Pipeline,
    #[cfg(feature = "image")]
    raster: raster::Pipeline,
    #[cfg(feature = "svg")]
    vector: vector::Pipeline,
    scale_factor: Option<f32>,
}

impl Renderer {
    pub fn new(settings: renderer::Settings) -> Self {
        Self {
            settings,
            layers: layer::Stack::new(),
            text: text::Pipeline::new(),
            #[cfg(feature = "image")]
            raster: raster::Pipeline::new(),
            #[cfg(feature = "svg")]
            vector: vector::Pipeline::new(),
            scale_factor: None,
        }
    }

    pub fn draw(
        &mut self,
        renderer: &mut vello_cpu::RenderContext,
        viewport: &Viewport,
        background_color: Color,
    ) {
        use vello_cpu::kurbo::Shape;

        const ACCURACY: f64 = 0.1;

        let scale = vello_cpu::kurbo::Affine::scale(f64::from(viewport.scale_factor()));

        renderer.set_transform(scale);
        renderer.set_paint(into_color(background_color));
        renderer.fill_rect(&into_rect(Rectangle::with_size(viewport.logical_size())));

        self.layers.merge();

        for layer in self.layers.iter() {
            renderer.set_transform(scale);
            renderer.push_clip_path(&into_rect(layer.bounds).to_path(ACCURACY));

            for (quad, background) in &layer.quads {
                renderer.set_paint(into_background(background, quad.bounds));

                let shadow = quad.shadow;

                if shadow.color.a > 0.0 {
                    renderer.set_filter_effect(
                        vello_common::filter_effects::Filter::from_primitive(
                            vello_common::filter_effects::FilterPrimitive::DropShadow {
                                dx: shadow.offset.x,
                                dy: shadow.offset.y,
                                std_deviation: shadow.blur_radius,
                                color: into_color(shadow.color),
                                edge_mode: vello_common::filter_effects::EdgeMode::None,
                            },
                        ),
                    );
                }

                if quad.border.radius == border::Radius::default() {
                    renderer.fill_rect(&into_rect(quad.bounds));

                    if quad.border.width > 0.0 && quad.border.color.a > 0.0 {
                        renderer.set_paint(into_color(quad.border.color));
                        renderer.set_stroke(vello_cpu::kurbo::Stroke::new(f64::from(
                            quad.border.width,
                        )));

                        renderer
                            .stroke_rect(&into_rect(quad.bounds.shrink(quad.border.width / 2.0)));
                    }
                } else {
                    let rounded_rect = into_rect(quad.bounds)
                        .to_rounded_rect((
                            f64::from(quad.border.radius.top_left),
                            f64::from(quad.border.radius.top_right),
                            f64::from(quad.border.radius.bottom_right),
                            f64::from(quad.border.radius.bottom_left),
                        ))
                        .to_path(ACCURACY);

                    renderer.fill_path(&rounded_rect);

                    if quad.border.width > 0.0 && quad.border.color.a > 0.0 {
                        renderer.set_paint(into_color(quad.border.color));
                        renderer.set_stroke(vello_cpu::kurbo::Stroke::new(f64::from(
                            quad.border.width,
                        )));

                        let border_rect = into_rect(quad.bounds.shrink(quad.border.width / 2.0))
                            .to_rounded_rect((
                                f64::from(quad.border.radius.top_left),
                                f64::from(quad.border.radius.top_right),
                                f64::from(quad.border.radius.bottom_right),
                                f64::from(quad.border.radius.bottom_left),
                            ))
                            .to_path(ACCURACY);

                        renderer.stroke_path(&border_rect);
                    }
                }

                renderer.reset_filter_effect();
            }

            renderer.reset_transform();

            #[cfg(feature = "geometry")]
            for group in &layer.primitives {
                use vello_cpu::kurbo;

                let Some(clip_bounds) = group.clip_bounds().intersection(&layer.bounds) else {
                    continue;
                };

                renderer.push_clip_path(
                    &into_rect(clip_bounds * viewport.scale_factor()).to_path(ACCURACY),
                );
                renderer.set_transform(
                    kurbo::Affine::scale(f64::from(viewport.scale_factor())).pre_translate(
                        kurbo::Vec2 {
                            x: f64::from(clip_bounds.x),
                            y: f64::from(clip_bounds.y),
                        },
                    ),
                );

                for primitive in group.as_slice() {
                    match primitive {
                        geometry::Primitive::Fill { path, paint, rule } => {
                            renderer.set_paint(paint.clone());
                            renderer.set_fill_rule(*rule);
                            renderer.fill_path(path);
                        }
                        geometry::Primitive::Stroke {
                            path,
                            paint,
                            stroke,
                        } => {
                            renderer.set_paint(paint.clone());
                            renderer.set_stroke(stroke.clone());
                            renderer.stroke_path(path);
                        }
                    }
                }

                renderer.reset_transform();
                renderer.pop_clip_path();
            }

            for image in &layer.images {
                match image {
                    #[cfg(feature = "image")]
                    iced_graphics::Image::Raster {
                        image,
                        bounds,
                        clip_bounds,
                    } => {
                        renderer.push_clip_path(
                            &into_rect(*clip_bounds * viewport.scale_factor()).to_path(ACCURACY),
                        );

                        self.raster
                            .draw(image, *bounds, renderer, viewport.scale_factor());

                        renderer.pop_clip_path();
                    }
                    #[cfg(feature = "svg")]
                    iced_graphics::Image::Vector {
                        svg,
                        bounds,
                        clip_bounds,
                    } => {
                        renderer.push_clip_path(
                            &into_rect(*clip_bounds * viewport.scale_factor()).to_path(ACCURACY),
                        );

                        self.vector
                            .draw(svg, *bounds, renderer, viewport.scale_factor());

                        renderer.pop_clip_path();
                    }
                    #[cfg(not(feature = "image"))]
                    iced_graphics::Image::Raster { .. } => {}
                    #[cfg(not(feature = "svg"))]
                    iced_graphics::Image::Vector { .. } => {}
                }
            }

            for item in &layer.text {
                for text in item.as_slice() {
                    match text {
                        Text::Paragraph {
                            paragraph,
                            position,
                            color,
                            clip_bounds,
                            transformation,
                        } => {
                            let transformation =
                                Transformation::scale(viewport.scale_factor()) * *transformation;

                            renderer.push_clip_path(
                                &into_rect(*clip_bounds * transformation).to_path(ACCURACY),
                            );

                            self.text.draw_paragraph(
                                paragraph,
                                *position,
                                *color,
                                renderer,
                                transformation,
                            );

                            renderer.pop_clip_path();
                        }
                        Text::Editor {
                            editor,
                            position,
                            color,
                            clip_bounds,
                            transformation,
                        } => {
                            let transformation =
                                Transformation::scale(viewport.scale_factor()) * *transformation;

                            renderer.push_clip_path(
                                &into_rect(*clip_bounds * transformation).to_path(ACCURACY),
                            );

                            self.text.draw_editor(
                                editor,
                                *position,
                                *color,
                                renderer,
                                transformation,
                            );

                            renderer.pop_clip_path();
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
                            clip_bounds,
                        } => {
                            let transformation = Transformation::scale(viewport.scale_factor())
                                * item.transformation();

                            let Some(clip_bounds) = (*clip_bounds * transformation)
                                .intersection(&(layer.bounds * transformation))
                            else {
                                continue;
                            };

                            renderer.push_clip_path(&into_rect(clip_bounds).to_path(ACCURACY));

                            self.text.draw_cached(
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
                                renderer,
                                transformation,
                            );

                            renderer.pop_clip_path();
                        }
                        Text::Raw {
                            raw,
                            transformation,
                        } => {
                            let Some(buffer) = raw.buffer.upgrade() else {
                                return;
                            };

                            let transformation =
                                Transformation::scale(viewport.scale_factor()) * *transformation;

                            let (width, height) = buffer.size();

                            let clip_bounds = Rectangle::new(
                                raw.position,
                                Size::new(
                                    width.unwrap_or(layer.bounds.width),
                                    height.unwrap_or(layer.bounds.height),
                                ),
                            );

                            renderer.push_clip_path(
                                &into_rect(clip_bounds * transformation).to_path(ACCURACY),
                            );

                            self.text.draw_raw(
                                &buffer,
                                raw.position,
                                raw.color,
                                renderer,
                                transformation,
                            );

                            renderer.pop_clip_path();
                        }
                    }
                }
            }

            renderer.pop_clip_path();
        }

        self.text.trim_cache();
    }
}

fn into_color(Color { r, g, b, a }: Color) -> vello_cpu::color::AlphaColor<vello_cpu::color::Srgb> {
    vello_cpu::color::AlphaColor::<vello_cpu::color::Srgb>::new([b, g, r, a])
}

fn into_background(background: &Background, bounds: Rectangle) -> vello_cpu::PaintType {
    match background {
        Background::Color(color) => vello_cpu::PaintType::Solid(into_color(*color)),
        Background::Gradient(gradient) => vello_cpu::PaintType::Gradient(match gradient {
            Gradient::Linear(gradient) => {
                let (start, end) = gradient.angle.to_distance(&bounds);

                vello_cpu::peniko::Gradient {
                    kind: vello_cpu::peniko::GradientKind::Linear(
                        vello_cpu::peniko::LinearGradientPosition::new(
                            (start.x, start.y),
                            (end.x, end.y),
                        ),
                    ),
                    stops: vello_cpu::peniko::ColorStops(
                        gradient
                            .stops
                            .into_iter()
                            .filter_map(|stop| {
                                let stop = stop?;

                                Some(vello_cpu::peniko::ColorStop {
                                    offset: stop.offset,
                                    color: vello_cpu::color::DynamicColor {
                                        cs: vello_cpu::color::ColorSpaceTag::Srgb,
                                        flags: vello_cpu::color::Flags::default(),
                                        components: [
                                            stop.color.b,
                                            stop.color.g,
                                            stop.color.r,
                                            stop.color.a,
                                        ],
                                    },
                                })
                            })
                            .collect(),
                    ),
                    ..vello_cpu::peniko::Gradient::default()
                }
            }
        }),
    }
}

fn into_rect(rectangle: Rectangle) -> vello_cpu::kurbo::Rect {
    vello_cpu::kurbo::Rect {
        x0: f64::from(rectangle.x),
        y0: f64::from(rectangle.y),
        x1: f64::from(rectangle.x + rectangle.width),
        y1: f64::from(rectangle.y + rectangle.height),
    }
}

impl core::Renderer for Renderer {
    fn start_layer(&mut self, bounds: Rectangle) {
        self.layers.push_clip(bounds);
    }

    fn end_layer(&mut self) {
        self.layers.pop_clip();
    }

    fn start_transformation(&mut self, transformation: Transformation) {
        self.layers.push_transformation(transformation);
    }

    fn end_transformation(&mut self) {
        self.layers.pop_transformation();
    }

    fn fill_quad(&mut self, quad: renderer::Quad, background: impl Into<Background>) {
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_quad(quad, background.into(), transformation);
    }

    fn allocate_image(
        &mut self,
        _handle: &image::Handle,
        _callback: impl FnOnce(Result<image::Allocation, image::Error>) + Send + 'static,
    ) {
        // TODO: Concurrency
        #[cfg(feature = "image")]
        _callback(self.raster.load(_handle));
    }

    fn hint(&mut self, scale_factor: f32) {
        self.scale_factor = Some(scale_factor);
    }

    fn scale_factor(&self) -> Option<f32> {
        self.scale_factor
    }

    fn reset(&mut self, new_bounds: Rectangle) {
        self.layers.reset(new_bounds);
    }
}

impl core::text::Renderer for Renderer {
    type Font = Font;
    type Paragraph = Paragraph;
    type Editor = Editor;

    const ICON_FONT: Font = Font::new("Iced-Icons");
    const CHECKMARK_ICON: char = '\u{f00c}';
    const ARROW_DOWN_ICON: char = '\u{e800}';
    const ICED_LOGO: char = '\u{e801}';
    const SCROLL_UP_ICON: char = '\u{e802}';
    const SCROLL_DOWN_ICON: char = '\u{e803}';
    const SCROLL_LEFT_ICON: char = '\u{e804}';
    const SCROLL_RIGHT_ICON: char = '\u{e805}';

    fn default_font(&self) -> Font {
        self.settings.default_font
    }

    fn default_size(&self) -> Pixels {
        self.settings.default_text_size
    }

    fn fill_paragraph(
        &mut self,
        paragraph: &Self::Paragraph,
        position: core::Point,
        color: core::Color,
        clip_bounds: Rectangle,
    ) {
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_paragraph(paragraph, position, color, clip_bounds, transformation);
    }

    fn fill_editor(
        &mut self,
        editor: &Self::Editor,
        position: core::Point,
        color: core::Color,
        clip_bounds: Rectangle,
    ) {
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_editor(editor, position, color, clip_bounds, transformation);
    }

    fn fill_text(
        &mut self,
        text: core::Text<String, Self::Font>,
        position: core::Point,
        color: core::Color,
        clip_bounds: Rectangle,
    ) {
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_text(text, position, color, clip_bounds, transformation);
    }
}

#[cfg(feature = "geometry")]
impl graphics::geometry::Renderer for Renderer {
    type Geometry = geometry::Geometry;
    type Frame = geometry::Frame;

    fn new_frame(&self, bounds: Rectangle) -> Self::Frame {
        geometry::Frame::new(bounds)
    }

    fn draw_geometry(&mut self, geometry: Self::Geometry) {
        pub use geometry::Geometry;

        let (layer, transformation) = self.layers.current_mut();

        match geometry {
            Geometry::Live {
                primitives,
                images,
                text,
                clip_bounds,
            } => {
                layer.draw_primitive_group(primitives, clip_bounds, transformation);

                for image in images {
                    layer.draw_image(image, transformation);
                }

                layer.draw_text_group(text, clip_bounds, transformation);
            }
            Geometry::Cache(cache) => {
                layer.draw_primitive_cache(cache.primitives, cache.clip_bounds, transformation);

                for image in cache.images.iter() {
                    layer.draw_image(image.clone(), transformation);
                }

                layer.draw_text_cache(cache.text, cache.clip_bounds, transformation);
            }
        }
    }
}

#[cfg(feature = "image")]
impl image::Renderer for Renderer {
    type Handle = image::Handle;

    fn load_image(&self, handle: &image::Handle) -> Result<image::Allocation, image::Error> {
        self.raster.load(handle)
    }

    fn measure_image(&self, handle: &image::Handle) -> Option<core::Size<u32>> {
        self.raster.dimensions(handle)
    }

    fn draw_image(&mut self, image: core::Image, bounds: Rectangle, clip_bounds: Rectangle) {
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_raster(image, bounds, clip_bounds, transformation);
    }
}

#[cfg(feature = "svg")]
impl core::svg::Renderer for Renderer {
    fn measure_svg(&self, handle: &core::svg::Handle) -> core::Size<u32> {
        self.vector.viewport_dimensions(handle)
    }

    fn draw_svg(&mut self, svg: core::Svg, bounds: Rectangle, clip_bounds: Rectangle) {
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_svg(svg, bounds, clip_bounds, transformation);
    }
}

impl mesh::Renderer for Renderer {
    fn draw_mesh(&mut self, _mesh: mesh::Mesh) {
        log::warn!("iced_vello_cpu does not support drawing meshes");
    }

    fn draw_mesh_cache(&mut self, _cache: mesh::Cache) {
        log::warn!("iced_vello_cpu does not support drawing meshes");
    }
}

impl compositor::Default for Renderer {
    type Compositor = Compositor;
}

pub struct Compositor {
    context: softbuffer::Context<Box<dyn compositor::Display>>,
    settings: compositor::Settings,
}

pub struct Surface {
    window: softbuffer::Surface<Box<dyn compositor::Display>, Box<dyn compositor::Window>>,
    renderer: vello_cpu::RenderContext,
}

impl graphics::Compositor for Compositor {
    type Renderer = Renderer;
    type Surface = Surface;

    async fn with_backend(
        settings: compositor::Settings,
        display: impl compositor::Display + Clone,
        _compatible_window: impl compositor::Window + Clone,
        _shell: Shell,
        backend: Option<&str>,
    ) -> Result<Self, Error> {
        match backend {
            None | Some("vello-cpu") | Some("vello_cpu") => {
                #[allow(unsafe_code)]
                let context = softbuffer::Context::new(Box::new(display) as _)
                    .expect("Create softbuffer context");

                Ok(Self { context, settings })
            }
            Some(backend) => Err(Error::GraphicsAdapterNotFound {
                backend: "vello-cpu",
                reason: error::Reason::DidNotMatch {
                    preferred_backend: backend.to_owned(),
                },
            }),
        }
    }

    fn create_renderer(&self, settings: renderer::Settings) -> Renderer {
        Renderer::new(settings)
    }

    fn create_surface<W: compositor::Window + Clone>(
        &mut self,
        window: W,
        width: u32,
        height: u32,
    ) -> Self::Surface {
        let window = softbuffer::Surface::new(&self.context, Box::new(window.clone()) as _)
            .expect("Create softbuffer surface for window");

        let mut surface = Surface {
            window,
            renderer: vello_cpu::RenderContext::new(1, 1),
        };

        if width > 0 && height > 0 {
            self.configure_surface(&mut surface, width, height);
        }

        surface
    }

    fn configure_surface(&mut self, surface: &mut Self::Surface, width: u32, height: u32) {
        surface
            .window
            .resize(
                NonZeroU32::new(width).expect("Non-zero width"),
                NonZeroU32::new(height).expect("Non-zero height"),
            )
            .expect("Resize surface");

        surface.renderer = vello_cpu::RenderContext::new(width as u16, height as u16);
    }

    fn information(&self) -> compositor::Information {
        compositor::Information {
            adapter: String::from("CPU"),
            backend: String::from("vello-cpu"),
        }
    }

    fn present(
        &mut self,
        renderer: &mut Self::Renderer,
        surface: &mut Self::Surface,
        viewport: &iced_graphics::Viewport,
        background_color: core::Color,
        on_pre_present: impl FnOnce(),
    ) -> Result<(), compositor::SurfaceError> {
        let mut buffer = surface
            .window
            .buffer_mut()
            .map_err(|_| compositor::SurfaceError::Lost)?;

        surface.renderer.reset();
        renderer.draw(&mut surface.renderer, viewport, background_color);
        surface.renderer.flush();

        surface.renderer.render_to_buffer(
            bytemuck::cast_slice_mut(&mut buffer),
            surface.renderer.width(),
            surface.renderer.height(),
            vello_cpu::RenderMode::OptimizeSpeed,
        );

        on_pre_present();
        buffer.present().map_err(|_| compositor::SurfaceError::Lost)
    }

    fn screenshot(
        &mut self,
        renderer: &mut Self::Renderer,
        viewport: &iced_graphics::Viewport,
        background_color: core::Color,
    ) -> Vec<u8> {
        screenshot(renderer, viewport, background_color)
    }
}

impl renderer::Headless for Renderer {
    async fn new(settings: renderer::Settings, backend: Option<&str>) -> Option<Self>
    where
        Self: Sized,
    {
        if backend.is_some_and(|backend| !["vello-cpu", "vello_cpu"].contains(&backend)) {
            return None;
        }

        Some(Self::new(settings))
    }

    fn name(&self) -> String {
        "vello_cpu".to_owned()
    }

    fn screenshot(
        &mut self,
        size: core::Size<u32>,
        scale_factor: f32,
        background_color: core::Color,
    ) -> Vec<u8> {
        screenshot(
            self,
            &Viewport::with_physical_size(size, scale_factor),
            background_color,
        )
    }
}

fn screenshot(renderer: &mut Renderer, viewport: &Viewport, background_color: Color) -> Vec<u8> {
    let mut vello = vello_cpu::RenderContext::new(
        viewport.physical_width() as u16,
        viewport.physical_height() as u16,
    );

    renderer.draw(&mut vello, viewport, background_color);
    vello.flush();

    let mut screenshot =
        vec![0; (viewport.physical_width() * viewport.physical_height()) as usize * 4];

    vello.render_to_buffer(
        &mut screenshot,
        viewport.physical_width() as u16,
        viewport.physical_height() as u16,
        vello_cpu::RenderMode::OptimizeQuality,
    );

    for i in 0..screenshot.len() / 4 {
        screenshot.swap(i * 4, i * 4 + 2);
    }

    screenshot
}
