#![allow(missing_docs)]
#![allow(dead_code)]

use iced_graphics as graphics;
use iced_graphics::core;

mod layer;

use crate::core::border;
use crate::core::image;
use crate::core::renderer;
use crate::core::text;
use crate::core::{Background, Color, Font, Image, Pixels, Rectangle, Transformation};
use crate::graphics::compositor;
use crate::graphics::error;
use crate::graphics::mesh;
use crate::graphics::text::{Editor, Paragraph};
use crate::graphics::{Error, Shell, Viewport};

use std::num::NonZeroU32;

pub struct Renderer {
    settings: Settings,
    layers: layer::Stack,
}

impl Renderer {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings,
            layers: layer::Stack::new(),
        }
    }

    pub fn draw(
        &mut self,
        renderer: &mut vello_cpu::RenderContext,
        viewport: &Viewport,
        background_color: Color,
    ) {
        const ACCURACY: f64 = 0.1;

        let scale = vello_cpu::kurbo::Affine::scale(f64::from(viewport.scale_factor()));

        renderer.set_transform(scale);
        renderer.set_paint(into_color(background_color));
        renderer.fill_rect(&into_rect(Rectangle::with_size(viewport.logical_size())));

        self.layers.merge();

        for layer in self.layers.iter() {
            for (quad, background) in &layer.quads {
                renderer.set_paint(into_background(background));

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
                    use vello_cpu::kurbo::Shape;

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

                // TODO: Shadows
            }
        }
    }
}

fn into_color(Color { r, g, b, a }: Color) -> vello_cpu::color::AlphaColor<vello_cpu::color::Srgb> {
    vello_cpu::color::AlphaColor::<vello_cpu::color::Srgb>::new([b, g, r, a])
}

fn into_background(background: &Background) -> vello_cpu::PaintType {
    match background {
        Background::Color(color) => vello_cpu::PaintType::Solid(into_color(*color)),
        Background::Gradient(_gradient) => todo!(),
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
        // TODO
    }

    fn hint(&mut self, _scale_factor: f32) {
        // TODO
    }

    fn scale_factor(&self) -> Option<f32> {
        None
    }

    fn reset(&mut self, new_bounds: Rectangle) {
        self.layers.reset(new_bounds);
    }
}

impl text::Renderer for Renderer {
    type Font = Font;
    type Paragraph = Paragraph;
    type Editor = Editor;

    const ICON_FONT: Font = Font::with_name("Iced-Icons");
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
        _text: &Self::Paragraph,
        _position: core::Point,
        _color: core::Color,
        _clip_bounds: Rectangle,
    ) {
    }

    fn fill_editor(
        &mut self,
        _editor: &Self::Editor,
        _position: core::Point,
        _color: core::Color,
        _clip_bounds: Rectangle,
    ) {
    }

    fn fill_text(
        &mut self,
        _text: core::Text<String, Self::Font>,
        _position: core::Point,
        _color: core::Color,
        _clip_bounds: Rectangle,
    ) {
    }
}

#[cfg(feature = "geometry")]
impl graphics::geometry::Renderer for Renderer {
    type Geometry = ();
    type Frame = ();

    fn new_frame(&self, _bounds: Rectangle) -> Self::Frame {
        todo!()
    }

    fn draw_geometry(&mut self, _geometry: Self::Geometry) {
        todo!()
    }
}

#[cfg(feature = "image")]
impl image::Renderer for Renderer {
    type Handle = image::Handle;

    fn load_image(&self, _handle: &image::Handle) -> Result<image::Allocation, image::Error> {
        todo!()
    }

    fn measure_image(&self, _handle: &image::Handle) -> Option<core::Size<u32>> {
        todo!()
    }

    fn draw_image(&mut self, _image: Image, _bounds: Rectangle, _clip_bounds: Rectangle) {
        todo!()
    }
}

#[cfg(feature = "svg")]
impl core::svg::Renderer for Renderer {
    fn measure_svg(&self, _handle: &core::svg::Handle) -> core::Size<u32> {
        todo!()
    }

    fn draw_svg(&mut self, _svg: core::Svg, _bounds: Rectangle, _clip_bounds: Rectangle) {
        todo!()
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
    settings: Settings,
}

pub struct Surface {
    window: softbuffer::Surface<Box<dyn compositor::Display>, Box<dyn compositor::Window>>,
    pixmap: vello_cpu::Pixmap,
    renderer: vello_cpu::RenderContext,
}

impl graphics::Compositor for Compositor {
    type Renderer = Renderer;
    type Surface = Surface;

    async fn with_backend(
        settings: graphics::Settings,
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

                Ok(Self {
                    context,
                    settings: Settings::from(settings),
                })
            }
            Some(backend) => Err(Error::GraphicsAdapterNotFound {
                backend: "vello-cpu",
                reason: error::Reason::DidNotMatch {
                    preferred_backend: backend.to_owned(),
                },
            }),
        }
    }

    fn create_renderer(&self) -> Renderer {
        Renderer::new(self.settings)
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
            pixmap: vello_cpu::Pixmap::new(1, 1),
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

        surface.pixmap.resize(width as u16, height as u16);

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
        surface.renderer.render_to_pixmap(&mut surface.pixmap);

        {
            let buffer = bytemuck::cast_slice_mut(&mut buffer);
            buffer.copy_from_slice(surface.pixmap.data_as_u8_slice());
        }

        on_pre_present();
        buffer.present().map_err(|_| compositor::SurfaceError::Lost)
    }

    fn screenshot(
        &mut self,
        _renderer: &mut Self::Renderer,
        _viewport: &iced_graphics::Viewport,
        _background_color: core::Color,
    ) -> Vec<u8> {
        todo!()
    }
}

impl renderer::Headless for Renderer {
    async fn new(
        default_font: Font,
        default_text_size: core::Pixels,
        backend: Option<&str>,
    ) -> Option<Self>
    where
        Self: Sized,
    {
        if backend.is_some_and(|backend| !["vello-cpu", "vello_cpu"].contains(&backend)) {
            return None;
        }

        Some(Self::new(Settings {
            default_font,
            default_text_size,
        }))
    }

    fn name(&self) -> String {
        "vello_cpu".to_owned()
    }

    fn screenshot(
        &mut self,
        _size: core::Size<u32>,
        _scale_factor: f32,
        _background_color: core::Color,
    ) -> Vec<u8> {
        todo!()
    }
}

/// The settings of a [`Compositor`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    /// The default [`Font`] to use.
    pub default_font: Font,

    /// The default size of text.
    ///
    /// By default, it will be set to `16.0`.
    pub default_text_size: Pixels,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            default_font: Font::default(),
            default_text_size: Pixels(16.0),
        }
    }
}

impl From<graphics::Settings> for Settings {
    fn from(settings: graphics::Settings) -> Self {
        Self {
            default_font: settings.default_font,
            default_text_size: settings.default_text_size,
        }
    }
}
