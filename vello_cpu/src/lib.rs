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

use crate::core::backend;
use crate::core::border;
use crate::core::image;
use crate::core::renderer;
use crate::core::{Background, Color, Font, Gradient, Rectangle, Size, Transformation};
use crate::graphics::compositor;
use crate::graphics::mesh;
use crate::graphics::text::{Editor, Paragraph};
use crate::graphics::{Shell, Text, Viewport};

use std::collections::VecDeque;
use std::num::NonZeroU32;

const ACCURACY: f64 = 0.1;

pub struct Renderer {
    settings: renderer::Settings,
    layers: layer::Stack,
    text: text::Pipeline,
    #[cfg(feature = "image")]
    raster: raster::Pipeline,
    #[cfg(feature = "svg")]
    vector: vector::Pipeline,
    scale: Option<renderer::Scale>,
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
            scale: None,
        }
    }

    pub fn layers(&mut self) -> &[layer::Layer] {
        self.layers.flush();
        self.layers.as_slice()
    }

    pub fn draw(
        &mut self,
        renderer: &mut vello_cpu::RenderContext,
        viewport: &Viewport,
        background_color: Color,
    ) {
        use vello_cpu::kurbo::Shape;

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

                #[cfg(not(feature = "multithreading"))]
                {
                    let shadow = quad.shadow;

                    if shadow.color.a > 0.0 {
                        renderer.set_filter_effect(
                            vello_cpu::filter_effects::Filter::from_primitive(
                                vello_cpu::filter_effects::FilterPrimitive::DropShadow {
                                    dx: shadow.offset.x,
                                    dy: shadow.offset.y,
                                    std_deviation: shadow.blur_radius,
                                    color: into_color(shadow.color),
                                    edge_mode: vello_cpu::filter_effects::EdgeMode::None,
                                },
                            ),
                        );
                    }
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
                    renderer.fill_path(&into_rounded_rect(quad.bounds, quad.border.radius));

                    if quad.border.width > 0.0 && quad.border.color.a > 0.0 {
                        renderer.set_paint(into_color(quad.border.color));
                        renderer.set_stroke(vello_cpu::kurbo::Stroke::new(f64::from(
                            quad.border.width,
                        )));

                        let border_rect = into_rounded_rect(
                            quad.bounds.shrink(quad.border.width / 2.0),
                            quad.border.radius,
                        );

                        renderer.stroke_path(&border_rect);
                    }
                }

                #[cfg(not(feature = "multithreading"))]
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
                renderer.pop_clip();
            }

            for image in &layer.images {
                match image {
                    #[cfg(feature = "image")]
                    iced_graphics::Image::Raster {
                        image,
                        bounds,
                        clip_bounds,
                    } => {
                        renderer.push_clip_path(&into_rounded_rect(
                            *clip_bounds * viewport.scale_factor(),
                            image.border_radius * viewport.scale_factor(),
                        ));

                        self.raster
                            .draw(image, *bounds, renderer, viewport.scale_factor());

                        renderer.pop_clip();
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

                        renderer.pop_clip();
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

                            renderer.pop_clip();
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

                            renderer.pop_clip();
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

                            renderer.pop_clip();
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

                            renderer.pop_clip();
                        }
                    }
                }
            }

            renderer.pop_clip();
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
                let (start, end) = bounds.chord(gradient.angle);

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

fn into_rounded_rect(rectangle: Rectangle, radius: border::Radius) -> vello_cpu::kurbo::BezPath {
    use vello_cpu::kurbo::Shape;

    into_rect(rectangle)
        .to_rounded_rect((
            f64::from(radius.top_left),
            f64::from(radius.top_right),
            f64::from(radius.bottom_right),
            f64::from(radius.bottom_left),
        ))
        .to_path(ACCURACY)
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
        &self,
        _handle: &image::Handle,
        _callback: impl FnOnce(Result<image::Allocation, image::Error>) + Send + 'static,
    ) {
        // TODO: Concurrency
        #[cfg(feature = "image")]
        _callback(self.raster.load(_handle));
    }

    fn hint(&mut self, scale: renderer::Scale) {
        self.scale = Some(scale);
    }

    fn scale(&self) -> Option<renderer::Scale> {
        self.scale
    }

    fn reset(&mut self, new_bounds: Rectangle) {
        self.layers.reset(new_bounds);
    }

    fn settings(&self) -> renderer::Settings {
        self.settings
    }
}

impl core::text::Renderer for Renderer {
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
        text: core::Text<String>,
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
    settings: backend::Settings,
}

#[derive(Debug, Clone)]
struct Frame {
    background: Color,
    layers: Vec<layer::Layer>,
}

pub struct Surface {
    window: softbuffer::Surface<Box<dyn compositor::Display>, Box<dyn compositor::Window>>,
    renderer: vello_cpu::RenderContext,
    resources: vello_cpu::Resources,
    /// The frames drawn into the back buffers, indexed by buffer age.
    frames: VecDeque<Frame>,
    /// The maximum buffer age seen so far; the history is truncated to this bound.
    max_age: u8,
}

impl graphics::Compositor for Compositor {
    type Renderer = Renderer;
    type Surface = Surface;

    async fn new(
        settings: backend::Settings,
        display: impl compositor::Display + Clone,
        _compatible_window: impl compositor::Window + Clone,
        _shell: Shell,
    ) -> Result<Self, backend::Error> {
        if !settings.backend.is_software() && !settings.backend.matches("vello-cpu") {
            return Err(backend::Error::GraphicsAdapterNotFound {
                backend: "vello-cpu",
                reason: backend::Reason::DidNotMatch {
                    preferred_backend: settings.backend,
                },
            });
        }

        #[allow(unsafe_code)]
        let context =
            softbuffer::Context::new(Box::new(display) as _).expect("Create softbuffer context");

        Ok(Self { context, settings })
    }

    fn create_renderer(&self, settings: renderer::Settings) -> Renderer {
        Renderer::new(settings)
    }

    fn create_surface(&mut self, window: impl compositor::Window + Clone) -> Self::Surface {
        let window = softbuffer::Surface::new(&self.context, Box::new(window.clone()) as _)
            .expect("Create softbuffer surface for window");

        Surface {
            window,
            renderer: vello_cpu::RenderContext::new(1, 1),
            resources: vello_cpu::Resources::new(),
            frames: VecDeque::new(),
            max_age: 0,
        }
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

        // The back buffers are reallocated, so all frame history is invalid.
        surface.frames.clear();
        surface.max_age = 0;
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

        let last_frame = {
            let age = buffer.age();

            surface.max_age = surface.max_age.max(age);
            surface.frames.truncate(surface.max_age as usize);

            if age > 0 {
                surface.frames.get(age as usize - 1)
            } else {
                None
            }
        };

        let damage = last_frame
            .and_then(|last_frame| {
                (last_frame.background == background_color).then(|| {
                    graphics::damage::diff(
                        &last_frame.layers,
                        renderer.layers(),
                        |layer| vec![layer.bounds],
                        layer::Layer::damage,
                    )
                })
            })
            .unwrap_or_else(|| vec![Rectangle::with_size(viewport.logical_size())]);

        if damage.is_empty() {
            if let Some(last_frame) = last_frame {
                surface.frames.push_front(last_frame.clone());
            }

            on_pre_present();
            return buffer.present().map_err(|_| compositor::SurfaceError::Lost);
        }

        surface.frames.push_front(Frame {
            background: background_color,
            layers: renderer.layers().to_vec(),
        });

        if background_color.a < 1.0 {
            // Transparent window: incremental rendering can't preserve
            // history, so render the whole scene and present it in full.
            draw_to_buffer(
                renderer,
                &mut surface.renderer,
                &mut surface.resources,
                viewport,
                background_color,
                bytemuck::cast_slice_mut(&mut buffer),
                &[],
            );

            on_pre_present();
            return buffer.present().map_err(|_| compositor::SurfaceError::Lost);
        }

        let damage = graphics::damage::group(damage, Rectangle::with_size(viewport.logical_size()));
        let physical: Vec<_> = physical_damage(&damage, viewport).collect();

        draw_to_buffer(
            renderer,
            &mut surface.renderer,
            &mut surface.resources,
            viewport,
            background_color,
            bytemuck::cast_slice_mut(&mut buffer),
            &physical,
        );

        on_pre_present();

        buffer
            .present_with_damage(&physical)
            .map_err(|_| compositor::SurfaceError::Lost)
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
        if backend.is_some_and(|backend| !["vello-cpu", "vello_cpu", "software"].contains(&backend))
        {
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
            &Viewport::with_physical_size(
                size,
                renderer::Scale {
                    window: 1.0,
                    application: scale_factor,
                },
            ),
            background_color,
        )
    }
}

/// Draws the scene into `buffer`, optionally clipping it to the given damage
/// regions (logical coordinates).
///
/// When `damage` is `None` the whole scene is rendered and the target is
/// cleared first. When it is `Some` the scene is clipped to the union of the
/// regions and rasterized with `SrcOver`, so that pixels outside the damaged
/// regions are preserved.
fn draw_to_buffer(
    renderer: &mut Renderer,
    vello: &mut vello_cpu::RenderContext,
    resources: &mut vello_cpu::Resources,
    viewport: &Viewport,
    background_color: Color,
    buffer: &mut [u8],
    damage: &[softbuffer::Rect],
) {
    use vello_cpu::kurbo::Shape;

    vello.reset();

    // Clip the scene to the union of the damaged regions. The clip is pushed
    // *before* the scale transform is applied, so the (physical, integer)
    // damage rects are used directly in device space — avoiding the subpixel
    // offsets a logical -> physical transform of the clip edges would
    // introduce.
    if !damage.is_empty() {
        let mut clip = vello_cpu::kurbo::BezPath::new();

        for region in damage {
            clip.extend(
                vello_cpu::kurbo::Rect {
                    x0: f64::from(region.x),
                    y0: f64::from(region.y),
                    x1: f64::from(region.x + u32::from(region.width)),
                    y1: f64::from(region.y + u32::from(region.height)),
                }
                .to_path(ACCURACY),
            );
        }
        vello.push_clip_path(&clip);
    }

    let scale = vello_cpu::kurbo::Affine::scale(f64::from(viewport.scale_factor()));
    vello.set_transform(scale);

    renderer.draw(vello, viewport, background_color);

    if !damage.is_empty() {
        vello.pop_clip();
    }
    vello.flush();

    let pixmap = vello_cpu::PixmapMut::new(
        viewport.physical_width() as u16,
        viewport.physical_height() as u16,
        buffer,
    )
    .expect("Buffer size matches renderer dimensions");

    // Undamaged pixels are only preserved when rasterizing with `SrcOver`; a
    // full render clears the target first.
    if damage.is_empty() {
        vello.render(pixmap, resources);
    } else {
        vello.render_with(
            pixmap,
            resources,
            vello_cpu::RasterizerSettings {
                target_init: vello_cpu::TargetInit::SrcOver,
                ..Default::default()
            },
        );
    }
}

fn physical_damage(
    damage: &[Rectangle],
    viewport: &Viewport,
) -> impl Iterator<Item = softbuffer::Rect> {
    let scale = viewport.scale_factor();
    let Size { width, height } = viewport.physical_size();

    damage
        .iter()
        .map(move |region| *region * scale)
        .filter_map(move |region| {
            let region = region.expand(2).snap()?;
            let left = region.x.min(width);
            let top = region.y.min(height);
            let right = (region.x + region.width).min(width);
            let bottom = (region.y + region.height).min(height);

            match (NonZeroU32::new(right - left), NonZeroU32::new(bottom - top)) {
                (Some(width), Some(height)) => Some(softbuffer::Rect {
                    x: left,
                    y: top,
                    width,
                    height,
                }),
                _ => None,
            }
        })
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

    let pixmap = vello_cpu::PixmapMut::new(
        viewport.physical_width() as u16,
        viewport.physical_height() as u16,
        &mut screenshot,
    )
    .expect("Screenshot buffer size matches viewport dimensions");

    vello.render(pixmap, &mut vello_cpu::Resources::new());

    for i in 0..screenshot.len() / 4 {
        screenshot.swap(i * 4, i * 4 + 2);
    }

    screenshot
}

#[cfg(test)]
mod damage_clip {
    use super::*;
    use iced_graphics::core::Renderer as _;
    use iced_graphics::core::renderer::Quad;

    fn scene() -> Renderer {
        let mut renderer = Renderer::new(renderer::Settings::default());

        renderer.start_layer(Rectangle {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        });

        // Red quad, top-left.
        renderer.fill_quad(
            Quad {
                bounds: Rectangle {
                    x: 10.0,
                    y: 10.0,
                    width: 30.0,
                    height: 30.0,
                },
                ..Default::default()
            },
            Background::Color(Color::from_rgb(1.0, 0.0, 0.0)),
        );

        // Green quad, bottom-right.
        renderer.fill_quad(
            Quad {
                bounds: Rectangle {
                    x: 60.0,
                    y: 60.0,
                    width: 30.0,
                    height: 30.0,
                },
                ..Default::default()
            },
            Background::Color(Color::from_rgb(0.0, 1.0, 0.0)),
        );

        renderer.end_layer();
        renderer
    }

    fn pixel(buffer: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * width + x) as usize) * 4;
        [buffer[i], buffer[i + 1], buffer[i + 2], buffer[i + 3]]
    }

    fn assert_incremental(scale: f32) {
        let physical = (100.0 * scale) as u32;
        let viewport = Viewport::with_physical_size(
            Size::new(physical, physical),
            renderer::Scale {
                window: 1.0,
                application: scale,
            },
        );
        let background = Color::from_rgb(0.1, 0.1, 0.1);

        let mut vello = vello_cpu::RenderContext::new(physical as u16, physical as u16);

        // Full render (ground truth).
        let mut full = vec![0; (physical * physical) as usize * 4];
        draw_to_buffer(
            &mut scene(),
            &mut vello,
            &mut vello_cpu::Resources::new(),
            &viewport,
            background,
            &mut full,
            &[],
        );

        // Back buffer holding a distinct previous frame.
        let mut back = vec![0; (physical * physical) as usize * 4];
        for chunk in back.as_chunks_mut::<4>().0 {
            *chunk = [0, 0, 255, 255];
        }

        // Incrementally render the region around the red quad. The damage is
        // specified in logical coordinates and converted to the physical rects
        // that `draw_to_buffer` clips to (via the same helper `present()` uses).
        let damage = vec![Rectangle {
            x: 5.0,
            y: 5.0,
            width: 50.0,
            height: 50.0,
        }];
        let regions: Vec<_> = physical_damage(&damage, &viewport).collect();

        draw_to_buffer(
            &mut scene(),
            &mut vello,
            &mut vello_cpu::Resources::new(),
            &viewport,
            background,
            &mut back,
            &regions,
        );

        // Sample points, in physical coordinates.
        let red_center = (30.0 * scale) as u32; // inside the red quad
        let background_point = (8.0 * scale) as u32; // inside the clip, plain background
        let green_center = (75.0 * scale) as u32; // inside the green quad

        // The damaged region matches a full render.
        assert_eq!(
            pixel(&back, physical, red_center, red_center),
            pixel(&full, physical, red_center, red_center)
        );
        assert_eq!(
            pixel(&back, physical, background_point, background_point),
            pixel(&full, physical, background_point, background_point)
        );

        // The undamaged region (green quad) keeps the previous content.
        assert_eq!(
            pixel(&back, physical, green_center, green_center),
            [0, 0, 255, 255]
        );
    }

    #[test]
    fn incremental_render_preserves_undamaged_pixels() {
        assert_incremental(1.0);
        assert_incremental(2.0);
    }

    #[test]
    fn physical_damage_scales_clamps_and_drops() {
        // 100x100 physical at scale 2.0 => 50x50 logical.
        let viewport = Viewport::with_physical_size(
            Size::new(100, 100),
            renderer::Scale {
                window: 1.0,
                application: 2.0,
            },
        );

        fn as_tuple(rects: impl Iterator<Item = softbuffer::Rect>) -> Vec<(u32, u32, u32, u32)> {
            rects
                .map(|r| (r.x, r.y, r.width.get(), r.height.get()))
                .collect()
        }

        // Scaled then expanded by the 2px margin: (10,10,20,20)*2 = (20,20,40,40),
        // expanded to (18,18,44,44).
        let damage = vec![Rectangle {
            x: 10.0,
            y: 10.0,
            width: 20.0,
            height: 20.0,
        }];
        assert_eq!(
            as_tuple(physical_damage(&damage, &viewport)),
            vec![(18, 18, 44, 44)]
        );

        // Clamped at the top-left (the expand pushes it negative; the clipped
        // part is dropped, so the width shrinks accordingly).
        let damage = vec![Rectangle {
            x: 0.0,
            y: 0.0,
            width: 5.0,
            height: 5.0,
        }];
        assert_eq!(
            as_tuple(physical_damage(&damage, &viewport)),
            vec![(0, 0, 12, 12)]
        );

        // Clamped at the bottom-right (extends past the viewport).
        let damage = vec![Rectangle {
            x: 45.0,
            y: 45.0,
            width: 20.0,
            height: 20.0,
        }];
        assert_eq!(
            as_tuple(physical_damage(&damage, &viewport)),
            vec![(88, 88, 12, 12)]
        );

        // Entirely outside the viewport => dropped (zero area after clamping).
        let damage = vec![Rectangle {
            x: 55.0,
            y: 55.0,
            width: 10.0,
            height: 10.0,
        }];
        assert_eq!(physical_damage(&damage, &viewport).count(), 0);
    }
}
