use crate::core::image;
use crate::core::renderer;
use crate::core::svg;
use crate::core::{
    self, Background, Color, Point, Rectangle, Size, Transformation,
};
use crate::graphics;
use crate::graphics::compositor;
use crate::graphics::mesh;

pub enum Renderer<L, R>
where
    L: core::Renderer,
    R: core::Renderer,
{
    Left(L),
    Right(R),
}

macro_rules! delegate {
    ($renderer:expr, $name:ident, $body:expr) => {
        match $renderer {
            Self::Left($name) => $body,
            Self::Right($name) => $body,
        }
    };
}

impl<L, R> Renderer<L, R>
where
    L: core::Renderer,
    R: core::Renderer,
{
    #[cfg(feature = "geometry")]
    pub fn draw_geometry<Geometry>(
        &mut self,
        layers: impl IntoIterator<Item = Geometry>,
    ) where
        L: graphics::geometry::Renderer,
        R: graphics::geometry::Renderer,

        Geometry: Into<geometry::Geometry<L::Geometry, R::Geometry>>,
    {
        use graphics::geometry::Renderer;

        for layer in layers {
            <Self as Renderer>::draw_geometry(self, layer.into());
        }
    }
}

impl<L, R> core::Renderer for Renderer<L, R>
where
    L: core::Renderer,
    R: core::Renderer,
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

    fn start_layer(&mut self) {
        delegate!(self, renderer, renderer.start_layer());
    }

    fn end_layer(&mut self, bounds: Rectangle) {
        delegate!(self, renderer, renderer.end_layer(bounds));
    }

    fn start_transformation(&mut self) {
        delegate!(self, renderer, renderer.start_transformation());
    }

    fn end_transformation(&mut self, transformation: Transformation) {
        delegate!(self, renderer, renderer.end_transformation(transformation));
    }
}

impl<L, R> core::text::Renderer for Renderer<L, R>
where
    L: core::text::Renderer,
    R: core::text::Renderer<
        Font = L::Font,
        Paragraph = L::Paragraph,
        Editor = L::Editor,
    >,
{
    type Font = L::Font;
    type Paragraph = L::Paragraph;
    type Editor = L::Editor;

    const ICON_FONT: Self::Font = L::ICON_FONT;
    const CHECKMARK_ICON: char = L::CHECKMARK_ICON;
    const ARROW_DOWN_ICON: char = L::ARROW_DOWN_ICON;

    fn default_font(&self) -> Self::Font {
        delegate!(self, renderer, renderer.default_font())
    }

    fn default_size(&self) -> core::Pixels {
        delegate!(self, renderer, renderer.default_size())
    }

    fn load_font(&mut self, font: std::borrow::Cow<'static, [u8]>) {
        delegate!(self, renderer, renderer.load_font(font));
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
        text: core::Text<'_, Self::Font>,
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

impl<L, R> image::Renderer for Renderer<L, R>
where
    L: image::Renderer,
    R: image::Renderer<Handle = L::Handle>,
{
    type Handle = L::Handle;

    fn measure_image(&self, handle: &Self::Handle) -> Size<u32> {
        delegate!(self, renderer, renderer.measure_image(handle))
    }

    fn draw_image(
        &mut self,
        handle: Self::Handle,
        filter_method: image::FilterMethod,
        bounds: Rectangle,
    ) {
        delegate!(
            self,
            renderer,
            renderer.draw_image(handle, filter_method, bounds)
        );
    }
}

impl<L, R> svg::Renderer for Renderer<L, R>
where
    L: svg::Renderer,
    R: svg::Renderer,
{
    fn measure_svg(&self, handle: &svg::Handle) -> Size<u32> {
        delegate!(self, renderer, renderer.measure_svg(handle))
    }

    fn draw_svg(
        &mut self,
        handle: svg::Handle,
        color: Option<Color>,
        bounds: Rectangle,
    ) {
        delegate!(self, renderer, renderer.draw_svg(handle, color, bounds));
    }
}

impl<L, R> mesh::Renderer for Renderer<L, R>
where
    L: mesh::Renderer,
    R: mesh::Renderer,
{
    fn draw_mesh(&mut self, mesh: graphics::Mesh) {
        delegate!(self, renderer, renderer.draw_mesh(mesh));
    }
}

pub enum Compositor<L, R>
where
    L: graphics::Compositor,
    R: graphics::Compositor,
{
    Left(L),
    Right(R),
}

pub enum Surface<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> graphics::Compositor for Compositor<L, R>
where
    L: graphics::Compositor,
    R: graphics::Compositor,
    L::Settings: From<crate::Settings>,
    R::Settings: From<crate::Settings>,
{
    type Settings = crate::Settings;
    type Renderer = Renderer<L::Renderer, R::Renderer>;
    type Surface = Surface<L::Surface, R::Surface>;

    async fn new<W: compositor::Window + Clone>(
        settings: Self::Settings,
        compatible_window: W,
    ) -> Result<Self, graphics::Error> {
        if let Ok(left) = L::new(settings.into(), compatible_window.clone())
            .await
            .map(Self::Left)
        {
            return Ok(left);
        }

        R::new(settings.into(), compatible_window)
            .await
            .map(Self::Right)
    }

    fn create_renderer(&self) -> Self::Renderer {
        match self {
            Self::Left(compositor) => {
                Renderer::Left(compositor.create_renderer())
            }
            Self::Right(compositor) => {
                Renderer::Right(compositor.create_renderer())
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
            Self::Left(compositor) => {
                Surface::Left(compositor.create_surface(window, width, height))
            }
            Self::Right(compositor) => {
                Surface::Right(compositor.create_surface(window, width, height))
            }
        }
    }

    fn configure_surface(
        &mut self,
        surface: &mut Self::Surface,
        width: u32,
        height: u32,
    ) {
        match (self, surface) {
            (Self::Left(compositor), Surface::Left(surface)) => {
                compositor.configure_surface(surface, width, height);
            }
            (Self::Right(compositor), Surface::Right(surface)) => {
                compositor.configure_surface(surface, width, height);
            }
            _ => unreachable!(),
        }
    }

    fn fetch_information(&self) -> compositor::Information {
        delegate!(self, compositor, compositor.fetch_information())
    }

    fn present<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        surface: &mut Self::Surface,
        viewport: &graphics::Viewport,
        background_color: Color,
        overlay: &[T],
    ) -> Result<(), compositor::SurfaceError> {
        match (self, renderer, surface) {
            (
                Self::Left(compositor),
                Renderer::Left(renderer),
                Surface::Left(surface),
            ) => compositor.present(
                renderer,
                surface,
                viewport,
                background_color,
                overlay,
            ),
            (
                Self::Right(compositor),
                Renderer::Right(renderer),
                Surface::Right(surface),
            ) => compositor.present(
                renderer,
                surface,
                viewport,
                background_color,
                overlay,
            ),
            _ => unreachable!(),
        }
    }

    fn screenshot<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        surface: &mut Self::Surface,
        viewport: &graphics::Viewport,
        background_color: Color,
        overlay: &[T],
    ) -> Vec<u8> {
        match (self, renderer, surface) {
            (
                Self::Left(compositor),
                Renderer::Left(renderer),
                Surface::Left(surface),
            ) => compositor.screenshot(
                renderer,
                surface,
                viewport,
                background_color,
                overlay,
            ),
            (
                Self::Right(compositor),
                Renderer::Right(renderer),
                Surface::Right(surface),
            ) => compositor.screenshot(
                renderer,
                surface,
                viewport,
                background_color,
                overlay,
            ),
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "wgpu")]
impl<L, R> iced_wgpu::primitive::pipeline::Renderer for Renderer<L, R>
where
    L: iced_wgpu::primitive::pipeline::Renderer,
    R: core::Renderer,
{
    fn draw_pipeline_primitive(
        &mut self,
        bounds: Rectangle,
        primitive: impl iced_wgpu::primitive::pipeline::Primitive,
    ) {
        match self {
            Self::Left(renderer) => {
                renderer.draw_pipeline_primitive(bounds, primitive);
            }
            Self::Right(_) => {
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
    use crate::core::{Point, Radians, Size, Vector};
    use crate::graphics::geometry::{self, Fill, Path, Stroke, Text};

    impl<L, R> geometry::Renderer for Renderer<L, R>
    where
        L: geometry::Renderer,
        R: geometry::Renderer,
    {
        type Geometry = Geometry<L::Geometry, R::Geometry>;
        type Frame = Frame<L::Frame, R::Frame>;

        fn new_frame(&self, size: iced_graphics::core::Size) -> Self::Frame {
            match self {
                Self::Left(renderer) => Frame::Left(renderer.new_frame(size)),
                Self::Right(renderer) => Frame::Right(renderer.new_frame(size)),
            }
        }

        fn draw_geometry(&mut self, geometry: Self::Geometry) {
            match (self, geometry) {
                (Self::Left(renderer), Geometry::Left(geometry)) => {
                    renderer.draw_geometry(geometry);
                }
                (Self::Right(renderer), Geometry::Right(geometry)) => {
                    renderer.draw_geometry(geometry);
                }
                _ => unreachable!(),
            }
        }
    }

    pub enum Geometry<L, R> {
        Left(L),
        Right(R),
    }

    impl<L, R> geometry::Geometry for Geometry<L, R>
    where
        L: geometry::Geometry,
        R: geometry::Geometry,
    {
        type Cache = Geometry<L::Cache, R::Cache>;

        fn load(cache: &Self::Cache) -> Self {
            match cache {
                Geometry::Left(cache) => Self::Left(L::load(cache)),
                Geometry::Right(cache) => Self::Right(R::load(cache)),
            }
        }

        fn cache(self) -> Self::Cache {
            match self {
                Self::Left(geometry) => Geometry::Left(geometry.cache()),
                Self::Right(geometry) => Geometry::Right(geometry.cache()),
            }
        }
    }

    pub enum Frame<L, R> {
        Left(L),
        Right(R),
    }

    impl<L, R> geometry::frame::Backend for Frame<L, R>
    where
        L: geometry::frame::Backend,
        R: geometry::frame::Backend,
    {
        type Geometry = Geometry<L::Geometry, R::Geometry>;

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

        fn fill_text(&mut self, text: impl Into<Text>) {
            delegate!(self, frame, frame.fill_text(text));
        }

        fn push_transform(&mut self) {
            delegate!(self, frame, frame.push_transform());
        }

        fn pop_transform(&mut self) {
            delegate!(self, frame, frame.pop_transform());
        }

        fn draft(&mut self, size: Size) -> Self {
            match self {
                Self::Left(frame) => Self::Left(frame.draft(size)),
                Self::Right(frame) => Self::Right(frame.draft(size)),
            }
        }

        fn paste(&mut self, frame: Self, at: Point) {
            match (self, frame) {
                (Self::Left(target), Self::Left(source)) => {
                    target.paste(source, at);
                }
                (Self::Right(target), Self::Right(source)) => {
                    target.paste(source, at);
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
                Frame::Left(frame) => Geometry::Left(frame.into_geometry()),
                Frame::Right(frame) => Geometry::Right(frame.into_geometry()),
            }
        }
    }
}
