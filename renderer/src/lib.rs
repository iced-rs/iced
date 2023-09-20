#![forbid(rust_2018_idioms)]
#![deny(unsafe_code, unused_results, rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
pub mod compositor;

#[cfg(feature = "geometry")]
pub mod geometry;

mod settings;

pub use iced_graphics as graphics;
pub use iced_graphics::core;

pub use compositor::Compositor;
pub use settings::Settings;

#[cfg(feature = "geometry")]
pub use geometry::Geometry;

use crate::core::renderer;
use crate::core::text::{self, Text};
use crate::core::{
    Background, Color, Font, Pixels, Point, Rectangle, Size, Vector,
};
use crate::graphics::text::Paragraph;
use crate::graphics::Mesh;

use std::borrow::Cow;

/// The default graphics renderer for [`iced`].
///
/// [`iced`]: https://github.com/iced-rs/iced
pub enum Renderer<Theme> {
    TinySkia(iced_tiny_skia::Renderer<Theme>),
    #[cfg(feature = "wgpu")]
    Wgpu(iced_wgpu::Renderer<Theme>),
}

macro_rules! delegate {
    ($renderer:expr, $name:ident, $body:expr) => {
        match $renderer {
            Self::TinySkia($name) => $body,
            #[cfg(feature = "wgpu")]
            Self::Wgpu($name) => $body,
        }
    };
}

impl<T> Renderer<T> {
    pub fn draw_mesh(&mut self, mesh: Mesh) {
        match self {
            Self::TinySkia(_) => {
                log::warn!("Unsupported mesh primitive: {mesh:?}");
            }
            #[cfg(feature = "wgpu")]
            Self::Wgpu(renderer) => {
                renderer.draw_primitive(iced_wgpu::Primitive::Custom(
                    iced_wgpu::primitive::Custom::Mesh(mesh),
                ));
            }
        }
    }
}

impl<T> core::Renderer for Renderer<T> {
    type Theme = T;

    fn with_layer(&mut self, bounds: Rectangle, f: impl FnOnce(&mut Self)) {
        match self {
            Self::TinySkia(renderer) => {
                let primitives = renderer.start_layer();

                f(self);

                match self {
                    Self::TinySkia(renderer) => {
                        renderer.end_layer(primitives, bounds);
                    }
                    #[cfg(feature = "wgpu")]
                    _ => unreachable!(),
                }
            }
            #[cfg(feature = "wgpu")]
            Self::Wgpu(renderer) => {
                let primitives = renderer.start_layer();

                f(self);

                match self {
                    #[cfg(feature = "wgpu")]
                    Self::Wgpu(renderer) => {
                        renderer.end_layer(primitives, bounds);
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    fn with_translation(
        &mut self,
        translation: Vector,
        f: impl FnOnce(&mut Self),
    ) {
        match self {
            Self::TinySkia(renderer) => {
                let primitives = renderer.start_translation();

                f(self);

                match self {
                    Self::TinySkia(renderer) => {
                        renderer.end_translation(primitives, translation);
                    }
                    #[cfg(feature = "wgpu")]
                    _ => unreachable!(),
                }
            }
            #[cfg(feature = "wgpu")]
            Self::Wgpu(renderer) => {
                let primitives = renderer.start_translation();

                f(self);

                match self {
                    #[cfg(feature = "wgpu")]
                    Self::Wgpu(renderer) => {
                        renderer.end_translation(primitives, translation);
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    fn fill_quad(
        &mut self,
        quad: renderer::Quad,
        background: impl Into<Background>,
    ) {
        delegate!(self, renderer, renderer.fill_quad(quad, background));
    }

    fn clear(&mut self) {
        delegate!(self, renderer, renderer.clear());
    }
}

impl<T> text::Renderer for Renderer<T> {
    type Font = Font;
    type Paragraph = Paragraph;

    const ICON_FONT: Font = iced_tiny_skia::Renderer::<T>::ICON_FONT;
    const CHECKMARK_ICON: char = iced_tiny_skia::Renderer::<T>::CHECKMARK_ICON;
    const ARROW_DOWN_ICON: char =
        iced_tiny_skia::Renderer::<T>::ARROW_DOWN_ICON;

    fn default_font(&self) -> Self::Font {
        delegate!(self, renderer, renderer.default_font())
    }

    fn default_size(&self) -> Pixels {
        delegate!(self, renderer, renderer.default_size())
    }

    fn create_paragraph(&self, text: Text<'_, Self::Font>) -> Self::Paragraph {
        delegate!(self, renderer, renderer.create_paragraph(text))
    }

    fn resize_paragraph(
        &self,
        paragraph: &mut Self::Paragraph,
        new_bounds: Size,
    ) {
        delegate!(
            self,
            renderer,
            renderer.resize_paragraph(paragraph, new_bounds)
        );
    }

    fn load_font(&mut self, bytes: Cow<'static, [u8]>) {
        delegate!(self, renderer, renderer.load_font(bytes));
    }

    fn fill_paragraph(
        &mut self,
        text: &Self::Paragraph,
        position: Point,
        color: Color,
    ) {
        delegate!(
            self,
            renderer,
            renderer.fill_paragraph(text, position, color)
        );
    }

    fn fill_text(
        &mut self,
        text: Text<'_, Self::Font>,
        position: Point,
        color: Color,
    ) {
        delegate!(self, renderer, renderer.fill_text(text, position, color));
    }
}

#[cfg(feature = "image")]
impl<T> crate::core::image::Renderer for Renderer<T> {
    type Handle = crate::core::image::Handle;

    fn dimensions(&self, handle: &crate::core::image::Handle) -> Size<u32> {
        delegate!(self, renderer, renderer.dimensions(handle))
    }

    fn draw(&mut self, handle: crate::core::image::Handle, bounds: Rectangle) {
        delegate!(self, renderer, renderer.draw(handle, bounds));
    }
}

#[cfg(feature = "svg")]
impl<T> crate::core::svg::Renderer for Renderer<T> {
    fn dimensions(&self, handle: &crate::core::svg::Handle) -> Size<u32> {
        delegate!(self, renderer, renderer.dimensions(handle))
    }

    fn draw(
        &mut self,
        handle: crate::core::svg::Handle,
        color: Option<crate::core::Color>,
        bounds: Rectangle,
    ) {
        delegate!(self, renderer, renderer.draw(handle, color, bounds));
    }
}

#[cfg(feature = "geometry")]
impl<T> crate::graphics::geometry::Renderer for Renderer<T> {
    type Geometry = crate::Geometry;

    fn draw(&mut self, layers: Vec<Self::Geometry>) {
        match self {
            Self::TinySkia(renderer) => {
                for layer in layers {
                    match layer {
                        crate::Geometry::TinySkia(primitive) => {
                            renderer.draw_primitive(primitive);
                        }
                        crate::Geometry::Wgpu(_) => unreachable!(),
                    }
                }
            }
            #[cfg(feature = "wgpu")]
            Self::Wgpu(renderer) => {
                for layer in layers {
                    match layer {
                        crate::Geometry::Wgpu(primitive) => {
                            renderer.draw_primitive(primitive);
                        }
                        crate::Geometry::TinySkia(_) => unreachable!(),
                    }
                }
            }
        }
    }
}
