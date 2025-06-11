//! Compose existing renderers and create type-safe fallback strategies.
use crate::core::image;
use crate::core::renderer;
use crate::core::svg;
use crate::core::{
    self, Background, Color, Font, Image, Pixels, Point, Rectangle, Size, Svg,
    Transformation,
};
use crate::graphics;
use crate::graphics::compositor;
use crate::graphics::mesh;

use std::borrow::Cow;

/// A renderer `A` with a fallback strategy `B`.
///
/// This type can be used to easily compose existing renderers and
/// create custom, type-safe fallback strategies.
#[derive(Debug)]
pub enum Renderer<A, B> {
    /// The primary rendering option.
    Primary(A),
    /// The secondary (or fallback) rendering option.
    Secondary(B),
}

macro_rules! delegate {
    ($renderer:expr, $name:ident, $body:expr) => {
        match $renderer {
            Self::Primary($name) => $body,
            Self::Secondary($name) => $body,
        }
    };
}

impl<A, B> core::Renderer for Renderer<A, B>
where
    A: core::Renderer,
    B: core::Renderer,
{
    fn fill_quad(
        &mut self,
        quad: renderer::Quad,
        background: impl Into<Background>,
    ) {
        delegate!(self, renderer, renderer.fill_quad(quad, background.into()));
    }

    fn clear(&mut self) {
        delegate!(self, renderer, renderer.clear());
    }

    fn start_layer(&mut self, bounds: Rectangle) {
        delegate!(self, renderer, renderer.start_layer(bounds));
    }

    fn end_layer(&mut self) {
        delegate!(self, renderer, renderer.end_layer());
    }

    fn start_transformation(&mut self, transformation: Transformation) {
        delegate!(
            self,
            renderer,
            renderer.start_transformation(transformation)
        );
    }

    fn end_transformation(&mut self) {
        delegate!(self, renderer, renderer.end_transformation());
    }
}

impl<A, B> core::text::Renderer for Renderer<A, B>
where
    A: core::text::Renderer,
    B: core::text::Renderer<
            Font = A::Font,
            Paragraph = A::Paragraph,
            Editor = A::Editor,
        >,
{
    type Font = A::Font;
    type Paragraph = A::Paragraph;
    type Editor = A::Editor;

    const MONOSPACE_FONT: Self::Font = A::MONOSPACE_FONT;
    const ICON_FONT: Self::Font = A::ICON_FONT;
    const CHECKMARK_ICON: char = A::CHECKMARK_ICON;
    const ARROW_DOWN_ICON: char = A::ARROW_DOWN_ICON;

    fn default_font(&self) -> Self::Font {
        delegate!(self, renderer, renderer.default_font())
    }

    fn default_size(&self) -> core::Pixels {
        delegate!(self, renderer, renderer.default_size())
    }

    fn fill_paragraph(
        &mut self,
        text: &Self::Paragraph,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        delegate!(
            self,
            renderer,
            renderer.fill_paragraph(text, position, color, clip_bounds)
        );
    }

    fn fill_editor(
        &mut self,
        editor: &Self::Editor,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        delegate!(
            self,
            renderer,
            renderer.fill_editor(editor, position, color, clip_bounds)
        );
    }

    fn fill_text(
        &mut self,
        text: core::Text<String, Self::Font>,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        delegate!(
            self,
            renderer,
            renderer.fill_text(text, position, color, clip_bounds)
        );
    }
}

impl<A, B> image::Renderer for Renderer<A, B>
where
    A: image::Renderer,
    B: image::Renderer<Handle = A::Handle>,
{
    type Handle = A::Handle;

    fn measure_image(&self, handle: &Self::Handle) -> Size<u32> {
        delegate!(self, renderer, renderer.measure_image(handle))
    }

    fn draw_image(&mut self, image: Image<A::Handle>, bounds: Rectangle) {
        delegate!(self, renderer, renderer.draw_image(image, bounds));
    }
}

impl<A, B> svg::Renderer for Renderer<A, B>
where
    A: svg::Renderer,
    B: svg::Renderer,
{
    fn measure_svg(&self, handle: &svg::Handle) -> Size<u32> {
        delegate!(self, renderer, renderer.measure_svg(handle))
    }

    fn draw_svg(&mut self, svg: Svg, bounds: Rectangle) {
        delegate!(self, renderer, renderer.draw_svg(svg, bounds));
    }
}

impl<A, B> mesh::Renderer for Renderer<A, B>
where
    A: mesh::Renderer,
    B: mesh::Renderer,
{
    fn draw_mesh(&mut self, mesh: graphics::Mesh) {
        delegate!(self, renderer, renderer.draw_mesh(mesh));
    }
}

/// A compositor `A` with a fallback strategy `B`.
///
/// It works analogously to [`Renderer`].
#[derive(Debug)]
pub enum Compositor<A, B>
where
    A: graphics::Compositor,
    B: graphics::Compositor,
{
    /// The primary compositing option.
    Primary(A),
    /// The secondary (or fallback) compositing option.
    Secondary(B),
}

/// A surface `A` with a fallback strategy `B`.
///
/// It works analogously to [`Renderer`].
#[derive(Debug)]
pub enum Surface<A, B> {
    /// The primary surface option.
    Primary(A),
    /// The secondary (or fallback) surface option.
    Secondary(B),
}

impl<A, B> graphics::Compositor for Compositor<A, B>
where
    A: graphics::Compositor,
    B: graphics::Compositor,
{
    type Renderer = Renderer<A::Renderer, B::Renderer>;
    type Surface = Surface<A::Surface, B::Surface>;

    async fn with_backend<W: compositor::Window + Clone>(
        settings: graphics::Settings,
        compatible_window: W,
        backend: Option<&str>,
    ) -> Result<Self, graphics::Error> {
        use std::env;

        let backends = backend
            .map(str::to_owned)
            .or_else(|| env::var("ICED_BACKEND").ok());

        let mut candidates: Vec<_> = backends
            .map(|backends| {
                backends
                    .split(',')
                    .filter(|candidate| !candidate.is_empty())
                    .map(str::to_owned)
                    .map(Some)
                    .collect()
            })
            .unwrap_or_default();

        if candidates.is_empty() {
            candidates.push(None);
        }

        let mut errors = vec![];

        for backend in candidates.iter().map(Option::as_deref) {
            match A::with_backend(settings, compatible_window.clone(), backend)
                .await
            {
                Ok(compositor) => return Ok(Self::Primary(compositor)),
                Err(error) => {
                    errors.push(error);
                }
            }

            match B::with_backend(settings, compatible_window.clone(), backend)
                .await
            {
                Ok(compositor) => return Ok(Self::Secondary(compositor)),
                Err(error) => {
                    errors.push(error);
                }
            }
        }

        Err(graphics::Error::List(errors))
    }

    fn create_renderer(&self) -> Self::Renderer {
        match self {
            Self::Primary(compositor) => {
                Renderer::Primary(compositor.create_renderer())
            }
            Self::Secondary(compositor) => {
                Renderer::Secondary(compositor.create_renderer())
            }
        }
    }

    fn create_surface<W: compositor::Window + Clone>(
        &mut self,
        window: W,
        width: u32,
        height: u32,
    ) -> Self::Surface {
        match self {
            Self::Primary(compositor) => Surface::Primary(
                compositor.create_surface(window, width, height),
            ),
            Self::Secondary(compositor) => Surface::Secondary(
                compositor.create_surface(window, width, height),
            ),
        }
    }

    fn configure_surface(
        &mut self,
        surface: &mut Self::Surface,
        width: u32,
        height: u32,
    ) {
        match (self, surface) {
            (Self::Primary(compositor), Surface::Primary(surface)) => {
                compositor.configure_surface(surface, width, height);
            }
            (Self::Secondary(compositor), Surface::Secondary(surface)) => {
                compositor.configure_surface(surface, width, height);
            }
            _ => unreachable!(),
        }
    }

    fn load_font(&mut self, font: Cow<'static, [u8]>) {
        delegate!(self, compositor, compositor.load_font(font));
    }

    fn fetch_information(&self) -> compositor::Information {
        delegate!(self, compositor, compositor.fetch_information())
    }

    fn present(
        &mut self,
        renderer: &mut Self::Renderer,
        surface: &mut Self::Surface,
        viewport: &graphics::Viewport,
        background_color: Color,
        on_pre_present: impl FnOnce(),
    ) -> Result<(), compositor::SurfaceError> {
        match (self, renderer, surface) {
            (
                Self::Primary(compositor),
                Renderer::Primary(renderer),
                Surface::Primary(surface),
            ) => compositor.present(
                renderer,
                surface,
                viewport,
                background_color,
                on_pre_present,
            ),
            (
                Self::Secondary(compositor),
                Renderer::Secondary(renderer),
                Surface::Secondary(surface),
            ) => compositor.present(
                renderer,
                surface,
                viewport,
                background_color,
                on_pre_present,
            ),
            _ => unreachable!(),
        }
    }

    fn screenshot(
        &mut self,
        renderer: &mut Self::Renderer,
        viewport: &graphics::Viewport,
        background_color: Color,
    ) -> Vec<u8> {
        match (self, renderer) {
            (Self::Primary(compositor), Renderer::Primary(renderer)) => {
                compositor.screenshot(renderer, viewport, background_color)
            }
            (Self::Secondary(compositor), Renderer::Secondary(renderer)) => {
                compositor.screenshot(renderer, viewport, background_color)
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "wgpu")]
impl<A, B> iced_wgpu::primitive::Renderer for Renderer<A, B>
where
    A: iced_wgpu::primitive::Renderer,
    B: core::Renderer,
{
    fn draw_primitive(
        &mut self,
        bounds: Rectangle,
        primitive: impl iced_wgpu::Primitive,
    ) {
        match self {
            Self::Primary(renderer) => {
                renderer.draw_primitive(bounds, primitive);
            }
            Self::Secondary(_) => {
                log::warn!(
                    "Custom shader primitive is not supported with this renderer."
                );
            }
        }
    }
}

#[cfg(feature = "geometry")]
mod geometry {
    use super::Renderer;
    use crate::core::{Point, Radians, Rectangle, Size, Svg, Vector};
    use crate::graphics::cache::{self, Cached};
    use crate::graphics::geometry::{self, Fill, Image, Path, Stroke, Text};

    impl<A, B> geometry::Renderer for Renderer<A, B>
    where
        A: geometry::Renderer,
        B: geometry::Renderer,
    {
        type Geometry = Geometry<A::Geometry, B::Geometry>;
        type Frame = Frame<A::Frame, B::Frame>;

        fn new_frame(&self, size: iced_graphics::core::Size) -> Self::Frame {
            match self {
                Self::Primary(renderer) => {
                    Frame::Primary(renderer.new_frame(size))
                }
                Self::Secondary(renderer) => {
                    Frame::Secondary(renderer.new_frame(size))
                }
            }
        }

        fn draw_geometry(&mut self, geometry: Self::Geometry) {
            match (self, geometry) {
                (Self::Primary(renderer), Geometry::Primary(geometry)) => {
                    renderer.draw_geometry(geometry);
                }
                (Self::Secondary(renderer), Geometry::Secondary(geometry)) => {
                    renderer.draw_geometry(geometry);
                }
                _ => unreachable!(),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub enum Geometry<A, B> {
        Primary(A),
        Secondary(B),
    }

    impl<A, B> Cached for Geometry<A, B>
    where
        A: Cached,
        B: Cached,
    {
        type Cache = Geometry<A::Cache, B::Cache>;

        fn load(cache: &Self::Cache) -> Self {
            match cache {
                Geometry::Primary(cache) => Self::Primary(A::load(cache)),
                Geometry::Secondary(cache) => Self::Secondary(B::load(cache)),
            }
        }

        fn cache(
            self,
            group: cache::Group,
            previous: Option<Self::Cache>,
        ) -> Self::Cache {
            match (self, previous) {
                (
                    Self::Primary(geometry),
                    Some(Geometry::Primary(previous)),
                ) => Geometry::Primary(geometry.cache(group, Some(previous))),
                (Self::Primary(geometry), None) => {
                    Geometry::Primary(geometry.cache(group, None))
                }
                (
                    Self::Secondary(geometry),
                    Some(Geometry::Secondary(previous)),
                ) => Geometry::Secondary(geometry.cache(group, Some(previous))),
                (Self::Secondary(geometry), None) => {
                    Geometry::Secondary(geometry.cache(group, None))
                }
                _ => unreachable!(),
            }
        }
    }

    #[derive(Debug)]
    pub enum Frame<A, B> {
        Primary(A),
        Secondary(B),
    }

    impl<A, B> geometry::frame::Backend for Frame<A, B>
    where
        A: geometry::frame::Backend,
        B: geometry::frame::Backend,
    {
        type Geometry = Geometry<A::Geometry, B::Geometry>;

        fn width(&self) -> f32 {
            delegate!(self, frame, frame.width())
        }

        fn height(&self) -> f32 {
            delegate!(self, frame, frame.height())
        }

        fn size(&self) -> Size {
            delegate!(self, frame, frame.size())
        }

        fn center(&self) -> Point {
            delegate!(self, frame, frame.center())
        }

        fn fill(&mut self, path: &Path, fill: impl Into<Fill>) {
            delegate!(self, frame, frame.fill(path, fill));
        }

        fn fill_rectangle(
            &mut self,
            top_left: Point,
            size: Size,
            fill: impl Into<Fill>,
        ) {
            delegate!(self, frame, frame.fill_rectangle(top_left, size, fill));
        }

        fn stroke<'a>(&mut self, path: &Path, stroke: impl Into<Stroke<'a>>) {
            delegate!(self, frame, frame.stroke(path, stroke));
        }

        fn stroke_rectangle<'a>(
            &mut self,
            top_left: Point,
            size: Size,
            stroke: impl Into<Stroke<'a>>,
        ) {
            delegate!(
                self,
                frame,
                frame.stroke_rectangle(top_left, size, stroke)
            );
        }

        fn stroke_text<'a>(
            &mut self,
            text: impl Into<Text>,
            stroke: impl Into<Stroke<'a>>,
        ) {
            delegate!(self, frame, frame.stroke_text(text, stroke));
        }

        fn fill_text(&mut self, text: impl Into<Text>) {
            delegate!(self, frame, frame.fill_text(text));
        }

        fn draw_image(&mut self, bounds: Rectangle, image: impl Into<Image>) {
            delegate!(self, frame, frame.draw_image(bounds, image));
        }

        fn draw_svg(&mut self, bounds: Rectangle, svg: impl Into<Svg>) {
            delegate!(self, frame, frame.draw_svg(bounds, svg));
        }

        fn push_transform(&mut self) {
            delegate!(self, frame, frame.push_transform());
        }

        fn pop_transform(&mut self) {
            delegate!(self, frame, frame.pop_transform());
        }

        fn draft(&mut self, bounds: Rectangle) -> Self {
            match self {
                Self::Primary(frame) => Self::Primary(frame.draft(bounds)),
                Self::Secondary(frame) => Self::Secondary(frame.draft(bounds)),
            }
        }

        fn paste(&mut self, frame: Self) {
            match (self, frame) {
                (Self::Primary(target), Self::Primary(source)) => {
                    target.paste(source);
                }
                (Self::Secondary(target), Self::Secondary(source)) => {
                    target.paste(source);
                }
                _ => unreachable!(),
            }
        }

        fn translate(&mut self, translation: Vector) {
            delegate!(self, frame, frame.translate(translation));
        }

        fn rotate(&mut self, angle: impl Into<Radians>) {
            delegate!(self, frame, frame.rotate(angle));
        }

        fn scale(&mut self, scale: impl Into<f32>) {
            delegate!(self, frame, frame.scale(scale));
        }

        fn scale_nonuniform(&mut self, scale: impl Into<Vector>) {
            delegate!(self, frame, frame.scale_nonuniform(scale));
        }

        fn into_geometry(self) -> Self::Geometry {
            match self {
                Frame::Primary(frame) => {
                    Geometry::Primary(frame.into_geometry())
                }
                Frame::Secondary(frame) => {
                    Geometry::Secondary(frame.into_geometry())
                }
            }
        }
    }
}

impl<A, B> renderer::Headless for Renderer<A, B>
where
    A: renderer::Headless,
    B: renderer::Headless,
{
    async fn new(
        default_font: Font,
        default_text_size: Pixels,
        backend: Option<&str>,
    ) -> Option<Self> {
        if let Some(renderer) =
            A::new(default_font, default_text_size, backend).await
        {
            return Some(Self::Primary(renderer));
        }

        B::new(default_font, default_text_size, backend)
            .await
            .map(Self::Secondary)
    }

    fn name(&self) -> String {
        delegate!(self, renderer, renderer.name())
    }

    fn screenshot(
        &mut self,
        size: Size<u32>,
        scale_factor: f32,
        background_color: Color,
    ) -> Vec<u8> {
        match self {
            crate::fallback::Renderer::Primary(renderer) => {
                renderer.screenshot(size, scale_factor, background_color)
            }
            crate::fallback::Renderer::Secondary(renderer) => {
                renderer.screenshot(size, scale_factor, background_color)
            }
        }
    }
}

impl<A, B> compositor::Default for Renderer<A, B>
where
    A: compositor::Default,
    B: compositor::Default,
{
    type Compositor = Compositor<A::Compositor, B::Compositor>;
}
