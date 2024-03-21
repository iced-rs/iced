use crate::core::image;
use crate::core::renderer;
use crate::core::svg;
use crate::core::text::Text;
use crate::core::{
    Background, Color, Font, Pixels, Point, Rectangle, Size, Transformation,
};
use crate::graphics::compositor;
use crate::graphics::text::{Editor, Paragraph};
use crate::graphics::{Mesh, Viewport};

#[cfg(feature = "geometry")]
use crate::graphics::geometry::{self, Fill, Path, Stroke};

use std::borrow::Cow;

pub trait Renderer {
    fn draw_mesh(&mut self, mesh: Mesh);

    fn start_layer(&mut self);

    fn end_layer(&mut self, bounds: Rectangle);

    fn start_transformation(&mut self);

    fn end_transformation(&mut self, transformation: Transformation);

    fn fill_quad(&mut self, quad: renderer::Quad, background: Background);

    fn clear(&mut self);

    fn default_font(&self) -> Font;

    fn default_size(&self) -> Pixels;

    fn load_font(&mut self, bytes: Cow<'static, [u8]>);

    fn fill_paragraph(
        &mut self,
        paragraph: &Paragraph,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    );

    fn fill_editor(
        &mut self,
        editor: &Editor,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    );

    fn fill_text(
        &mut self,
        text: Text<'_, Font>,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    );

    fn measure_image(&self, handle: &image::Handle) -> Size<u32>;

    fn draw_image(
        &mut self,
        handle: image::Handle,
        filter_method: image::FilterMethod,
        bounds: Rectangle,
    );

    fn measure_svg(&self, handle: &svg::Handle) -> Size<u32>;

    fn draw_svg(
        &mut self,
        handle: crate::core::svg::Handle,
        color: Option<crate::core::Color>,
        bounds: Rectangle,
    );

    #[cfg(feature = "geometry")]
    fn new_frame(&self, size: Size) -> Box<dyn Frame>;

    #[cfg(feature = "geometry")]
    fn draw_geometry(&mut self, geometry: Box<dyn Geometry>);

    fn present(
        &mut self,
        surface: &mut dyn Surface,
        viewport: &Viewport,
        background_color: Color,
        compositor: &mut dyn Compositor,
    ) -> Result<(), compositor::SurfaceError>;
}

#[cfg(feature = "geometry")]
pub trait Frame: std::any::Any {
    fn new(&self, size: Size) -> Box<dyn Frame>;

    fn width(&self) -> f32;

    fn height(&self) -> f32;

    fn size(&self) -> Size;

    fn center(&self) -> Point;

    fn fill(&mut self, path: &Path, fill: Fill);

    fn fill_rectangle(&mut self, top_left: Point, size: Size, fill: Fill);

    fn stroke<'a>(&mut self, path: &Path, stroke: Stroke<'a>);

    fn fill_text(&mut self, text: geometry::Text);

    fn translate(&mut self, translation: crate::core::Vector);

    fn rotate(&mut self, angle: crate::core::Radians);

    fn scale(&mut self, scale: f32);

    fn scale_nonuniform(&mut self, scale: crate::core::Vector);

    fn push_transform(&mut self);

    fn pop_transform(&mut self);

    fn clip(&mut self, frame: Box<dyn Frame>, origin: Point);

    fn into_geometry(self: Box<Self>) -> Box<dyn Geometry>;
}

#[cfg(feature = "geometry")]
pub trait Geometry: std::any::Any + std::fmt::Debug {
    fn transform(
        self: Box<Self>,
        transformation: Transformation,
    ) -> Box<dyn Geometry>;

    fn cache(self: Box<Self>) -> std::sync::Arc<dyn Geometry>;

    fn load(self: std::sync::Arc<Self>) -> Box<dyn Geometry>;
}

pub trait Compositor: std::any::Any {
    fn create_renderer(&self) -> Box<dyn Renderer>;

    fn create_surface(
        &mut self,
        window: Box<dyn compositor::Window>,
        width: u32,
        height: u32,
    ) -> Box<dyn Surface>;

    fn configure_surface(
        &mut self,
        surface: &mut dyn Surface,
        width: u32,
        height: u32,
    );

    fn fetch_information(&self) -> compositor::Information;
}

pub trait Surface: std::any::Any {}
