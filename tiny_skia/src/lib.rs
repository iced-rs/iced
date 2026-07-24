#![allow(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
pub mod window;

mod engine;
mod layer;
mod primitive;
mod text;

#[cfg(feature = "image")]
mod raster;

#[cfg(feature = "svg")]
mod vector;

#[cfg(feature = "geometry")]
pub mod geometry;

use iced_debug as debug;
pub use iced_graphics as graphics;
pub use iced_graphics::core;

pub use layer::Layer;
pub use primitive::Primitive;

#[cfg(feature = "geometry")]
pub use geometry::Geometry;

use crate::core::renderer;
use crate::core::{Background, Color, Font, Pixels, Point, Rectangle, Size, Transformation};
use crate::engine::Engine;
use crate::graphics::Viewport;
use crate::graphics::compositor;
use crate::graphics::text::{Editor, Paragraph};

/// A [`tiny-skia`] graphics renderer for [`iced`].
///
/// [`tiny-skia`]: https://github.com/RazrFalcon/tiny-skia
/// [`iced`]: https://github.com/iced-rs/iced
#[derive(Debug)]
pub struct Renderer {
    settings: renderer::Settings,
    layers: layer::Stack,
    engine: Engine, // TODO: Shared engine
}

impl Renderer {
    pub fn new(settings: renderer::Settings) -> Self {
        Self {
            settings,
            layers: layer::Stack::new(),
            engine: Engine::new(),
        }
    }

    pub fn layers(&mut self) -> &[Layer] {
        self.layers.flush();
        self.layers.as_slice()
    }

    pub fn draw(
        &mut self,
        pixels: &mut tiny_skia::PixmapMut<'_>,
        clip_mask: &mut tiny_skia::Mask,
        viewport: &Viewport,
        damage: &[Rectangle],
        background_color: Color,
    ) {
        let scale_factor = viewport.scale_factor();
        self.layers.flush();

        let plan = self.layers.opacity_plan();
        let opacity_groups = self.layers.opacity_groups().to_vec();

        for &damage_bounds in damage {
            let damage_bounds = damage_bounds * scale_factor;

            let path = tiny_skia::PathBuilder::from_rect(
                tiny_skia::Rect::from_xywh(
                    damage_bounds.x,
                    damage_bounds.y,
                    damage_bounds.width,
                    damage_bounds.height,
                )
                .expect("Create damage rectangle"),
            );

            pixels.fill_path(
                &path,
                &tiny_skia::Paint {
                    shader: tiny_skia::Shader::SolidColor(engine::into_color(background_color)),
                    anti_alias: false,
                    blend_mode: tiny_skia::BlendMode::Source,
                    ..Default::default()
                },
                tiny_skia::FillRule::default(),
                tiny_skia::Transform::identity(),
                None,
            );

            // Isolated targets for the currently open opacity groups. Each group
            // is composited into its own transparent pixmap and then blended as a
            // whole, so overlapping primitives fade together instead of each on
            // its own. The pixmap is sized to the group's bounds within this
            // damage region (not the whole window), which keeps compositing cheap.
            let mut targets: Vec<GroupTarget> = Vec::new();

            for (index, layer) in self.layers.iter().enumerate() {
                let step = &plan.steps[index];

                for _ in &step.closes {
                    composite_opacity_group(&mut targets, pixels);
                }

                for &group in &step.opens {
                    let opacity_group = opacity_groups[group];
                    let rect = (opacity_group.bounds * scale_factor)
                        .intersection(&damage_bounds)
                        .and_then(Rectangle::snap);

                    targets.push(match rect {
                        Some(rect) if rect.width > 0 && rect.height > 0 => GroupTarget {
                            pixmap: tiny_skia::Pixmap::new(rect.width, rect.height)
                                .expect("Create opacity group pixmap"),
                            mask: tiny_skia::Mask::new(rect.width, rect.height)
                                .expect("Create opacity group mask"),
                            origin: (rect.x as f32, rect.y as f32),
                            opacity: opacity_group.opacity,
                            active: true,
                        },
                        _ => GroupTarget::inactive(opacity_group.opacity),
                    });
                }

                let Some(layer_bounds) = damage_bounds.intersection(&(layer.bounds * scale_factor))
                else {
                    continue;
                };

                match targets.last_mut() {
                    Some(target) if target.active => {
                        let origin = target.origin;
                        let group_rect = Rectangle {
                            x: origin.0,
                            y: origin.1,
                            width: target.pixmap.width() as f32,
                            height: target.pixmap.height() as f32,
                        };

                        let Some(source_bounds) = layer_bounds.intersection(&group_rect) else {
                            continue;
                        };

                        let mut pixmap = target.pixmap.as_mut();
                        Self::render_layer(
                            &mut self.engine,
                            layer,
                            &mut pixmap,
                            &mut target.mask,
                            origin,
                            source_bounds,
                            scale_factor,
                        );
                    }
                    Some(_) => {}
                    None => {
                        Self::render_layer(
                            &mut self.engine,
                            layer,
                            pixels,
                            clip_mask,
                            (0.0, 0.0),
                            layer_bounds,
                            scale_factor,
                        );
                    }
                }
            }

            for _ in &plan.trailing {
                composite_opacity_group(&mut targets, pixels);
            }
        }

        self.engine.trim();
    }

    /// Draws a single layer into the given target pixmap (either the frame or an
    /// isolated opacity-group pixmap).
    ///
    /// `offset` is the physical top-left of the target within the frame; it is
    /// `(0, 0)` for the frame and the group's origin for an opacity group, so
    /// that a group can render into a small, bounds-sized pixmap. `layer_bounds`
    /// is in physical (frame) coordinates.
    fn render_layer(
        engine: &mut Engine,
        layer: &Layer,
        pixels: &mut tiny_skia::PixmapMut<'_>,
        clip_mask: &mut tiny_skia::Mask,
        offset: (f32, f32),
        layer_bounds: Rectangle,
        scale_factor: f32,
    ) {
        let (offset_x, offset_y) = offset;
        let to_local = |bounds: Rectangle| Rectangle {
            x: bounds.x - offset_x,
            y: bounds.y - offset_y,
            ..bounds
        };
        // Logical -> frame-physical (scale), then frame-physical -> target-local
        // (translate by the target's origin).
        let transformation =
            Transformation::translate(-offset_x, -offset_y) * Transformation::scale(scale_factor);
        let layer_bounds = to_local(layer_bounds);

        engine::adjust_clip_mask(clip_mask, layer_bounds);

        if !layer.quads.is_empty() {
            let render_span = debug::render(debug::Primitive::Quad);
            for (quad, background) in &layer.quads {
                engine.draw_quad(
                    quad,
                    background,
                    transformation,
                    pixels,
                    clip_mask,
                    layer_bounds,
                );
            }
            render_span.finish();
        }

        if !layer.primitives.is_empty() {
            let render_span = debug::render(debug::Primitive::Triangle);

            for group in &layer.primitives {
                let Some(group_bounds) =
                    to_local(group.clip_bounds() * scale_factor).intersection(&layer_bounds)
                else {
                    continue;
                };

                engine::adjust_clip_mask(clip_mask, group_bounds);

                for primitive in group.as_slice() {
                    engine.draw_primitive(
                        primitive,
                        transformation * group.transformation(),
                        pixels,
                        clip_mask,
                        group_bounds,
                    );
                }

                engine::adjust_clip_mask(clip_mask, layer_bounds);
            }

            render_span.finish();
        }

        if !layer.images.is_empty() {
            let render_span = debug::render(debug::Primitive::Image);

            for image in &layer.images {
                engine.draw_image(image, transformation, pixels, clip_mask, layer_bounds);
            }

            render_span.finish();
        }

        if !layer.text.is_empty() {
            let render_span = debug::render(debug::Primitive::Image);

            for group in &layer.text {
                for text in group.as_slice() {
                    engine.draw_text(
                        text,
                        transformation * group.transformation(),
                        pixels,
                        clip_mask,
                        layer_bounds,
                    );
                }
            }

            render_span.finish();
        }
    }
}

/// An isolated target for an opacity group, sized to the group's bounds within
/// the current damage region.
struct GroupTarget {
    pixmap: tiny_skia::Pixmap,
    mask: tiny_skia::Mask,
    /// Physical top-left of the group within the frame.
    origin: (f32, f32),
    opacity: f32,
    /// `false` when the group does not intersect the damage region, in which
    /// case it has nothing to render or composite (kept to balance open/close).
    active: bool,
}

impl GroupTarget {
    fn inactive(opacity: f32) -> Self {
        Self {
            pixmap: tiny_skia::Pixmap::new(1, 1).expect("Create pixmap"),
            mask: tiny_skia::Mask::new(1, 1).expect("Create mask"),
            origin: (0.0, 0.0),
            opacity,
            active: false,
        }
    }
}

/// Composites the top-most opacity-group target into its parent (an enclosing
/// group target, or the base frame) at the group's opacity.
///
/// This is what turns opacity into a single flattened layer: the group has
/// already been drawn into its own transparent pixmap, so blending it as a whole
/// yields correct results even when its primitives overlap.
fn composite_opacity_group(targets: &mut Vec<GroupTarget>, base: &mut tiny_skia::PixmapMut<'_>) {
    let Some(group) = targets.pop() else {
        return;
    };

    if !group.active {
        return;
    }

    let paint = tiny_skia::PixmapPaint {
        opacity: group.opacity,
        blend_mode: tiny_skia::BlendMode::SourceOver,
        quality: tiny_skia::FilterQuality::Nearest,
    };

    let (x, y) = group.origin;

    // Composite into the nearest enclosing active group, or the frame. The child
    // is positioned relative to whichever target it lands in.
    match targets.last_mut() {
        Some(parent) if parent.active => {
            parent.pixmap.as_mut().draw_pixmap(
                (x - parent.origin.0) as i32,
                (y - parent.origin.1) as i32,
                group.pixmap.as_ref(),
                &paint,
                tiny_skia::Transform::identity(),
                None,
            );
        }
        _ => {
            base.draw_pixmap(
                x as i32,
                y as i32,
                group.pixmap.as_ref(),
                &paint,
                tiny_skia::Transform::identity(),
                None,
            );
        }
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

    fn start_opacity(&mut self, bounds: Rectangle, opacity: f32) {
        self.layers.push_opacity(opacity, bounds);
    }

    fn end_opacity(&mut self) {
        self.layers.pop_opacity();
    }

    fn fill_quad(&mut self, quad: renderer::Quad, background: impl Into<Background>) {
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_quad(quad, background.into(), transformation);
    }

    fn allocate_image(
        &mut self,
        _handle: &core::image::Handle,
        callback: impl FnOnce(Result<core::image::Allocation, core::image::Error>) + Send + 'static,
    ) {
        #[cfg(feature = "image")]
        #[allow(unsafe_code)]
        // TODO: Concurrency
        callback(self.engine.raster_pipeline.load(_handle));

        #[cfg(not(feature = "image"))]
        callback(Err(core::image::Error::Unsupported));
    }

    fn hint(&mut self, _scale: renderer::Scale) {
        // TODO: No hinting supported
        // We'll replace `tiny-skia` with `vello_cpu` soon
    }

    fn scale(&self) -> Option<renderer::Scale> {
        None
    }

    fn reset(&mut self, new_bounds: Rectangle) {
        self.layers.reset(new_bounds);
    }

    fn settings(&self) -> renderer::Settings {
        self.settings
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

    fn default_font(&self) -> Self::Font {
        self.settings.default_font
    }

    fn default_size(&self) -> Pixels {
        self.settings.default_text_size
    }

    fn fill_paragraph(
        &mut self,
        text: &Self::Paragraph,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        let (layer, transformation) = self.layers.current_mut();

        layer.draw_paragraph(text, position, color, clip_bounds, transformation);
    }

    fn fill_editor(
        &mut self,
        editor: &Self::Editor,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_editor(editor, position, color, clip_bounds, transformation);
    }

    fn fill_text(
        &mut self,
        text: core::Text,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_text(text, position, color, clip_bounds, transformation);
    }
}

impl graphics::text::Renderer for Renderer {
    fn fill_raw(&mut self, raw: graphics::text::Raw) {
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_text_raw(raw, transformation);
    }
}

#[cfg(feature = "geometry")]
impl graphics::geometry::Renderer for Renderer {
    type Geometry = Geometry;
    type Frame = geometry::Frame;

    fn new_frame(&self, bounds: Rectangle) -> Self::Frame {
        geometry::Frame::new(bounds)
    }

    fn draw_geometry(&mut self, geometry: Self::Geometry) {
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

impl graphics::mesh::Renderer for Renderer {
    fn draw_mesh(&mut self, _mesh: graphics::Mesh) {
        log::warn!("iced_tiny_skia does not support drawing meshes");
    }

    fn draw_mesh_cache(&mut self, _cache: iced_graphics::mesh::Cache) {
        log::warn!("iced_tiny_skia does not support drawing meshes");
    }
}

#[cfg(feature = "image")]
impl core::image::Renderer for Renderer {
    type Handle = core::image::Handle;

    fn load_image(
        &self,
        handle: &Self::Handle,
    ) -> Result<core::image::Allocation, core::image::Error> {
        self.engine.raster_pipeline.load(handle)
    }

    fn measure_image(&self, handle: &Self::Handle) -> Option<crate::core::Size<u32>> {
        self.engine.raster_pipeline.dimensions(handle)
    }

    fn draw_image(&mut self, image: core::Image, bounds: Rectangle, clip_bounds: Rectangle) {
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_raster(image, bounds, clip_bounds, transformation);
    }
}

#[cfg(feature = "svg")]
impl core::svg::Renderer for Renderer {
    fn measure_svg(&self, handle: &core::svg::Handle) -> crate::core::Size<u32> {
        self.engine.vector_pipeline.viewport_dimensions(handle)
    }

    fn draw_svg(&mut self, svg: core::Svg, bounds: Rectangle, clip_bounds: Rectangle) {
        let (layer, transformation) = self.layers.current_mut();
        layer.draw_svg(svg, bounds, clip_bounds, transformation);
    }
}

impl compositor::Default for Renderer {
    type Compositor = window::Compositor;
}

impl renderer::Headless for Renderer {
    async fn new(settings: renderer::Settings, backend: Option<&str>) -> Option<Self> {
        if backend.is_some_and(|backend| !["tiny-skia", "tiny_skia", "software"].contains(&backend))
        {
            return None;
        }

        Some(Self::new(settings))
    }

    fn name(&self) -> String {
        "tiny-skia".to_owned()
    }

    fn screenshot(
        &mut self,
        size: Size<u32>,
        scale_factor: f32,
        background_color: Color,
    ) -> Vec<u8> {
        let viewport = Viewport::with_physical_size(
            size,
            renderer::Scale {
                window: 1.0,
                application: scale_factor,
            },
        );

        window::compositor::screenshot(self, &viewport, background_color)
    }
}
