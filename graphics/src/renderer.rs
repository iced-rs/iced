//! Create a renderer from a [`Backend`].
use crate::backend::{self, Backend};
use crate::core;
use crate::core::image;
use crate::core::renderer;
use crate::core::svg;
use crate::core::text::Text;
use crate::core::{
    Background, Color, Font, Pixels, Point, Rectangle, Size, Transformation,
};
use crate::text;
use crate::Primitive;

use std::borrow::Cow;

/// A backend-agnostic renderer that supports all the built-in widgets.
#[derive(Debug)]
pub struct Renderer<B: Backend> {
    backend: B,
    default_font: Font,
    default_text_size: Pixels,
    primitives: Vec<Primitive<B::Primitive>>,
}

impl<B: Backend> Renderer<B> {
    /// Creates a new [`Renderer`] from the given [`Backend`].
    pub fn new(
        backend: B,
        default_font: Font,
        default_text_size: Pixels,
    ) -> Self {
        Self {
            backend,
            default_font,
            default_text_size,
            primitives: Vec::new(),
        }
    }

    /// Returns a reference to the [`Backend`] of the [`Renderer`].
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// Enqueues the given [`Primitive`] in the [`Renderer`] for drawing.
    pub fn draw_primitive(&mut self, primitive: Primitive<B::Primitive>) {
        self.primitives.push(primitive);
    }

    /// Runs the given closure with the [`Backend`] and the recorded primitives
    /// of the [`Renderer`].
    pub fn with_primitives<O>(
        &mut self,
        f: impl FnOnce(&mut B, &[Primitive<B::Primitive>]) -> O,
    ) -> O {
        f(&mut self.backend, &self.primitives)
    }

    /// Starts recording a new layer.
    pub fn start_layer(&mut self) -> Vec<Primitive<B::Primitive>> {
        std::mem::take(&mut self.primitives)
    }

    /// Ends the recording of a layer.
    pub fn end_layer(
        &mut self,
        primitives: Vec<Primitive<B::Primitive>>,
        bounds: Rectangle,
    ) {
        let layer = std::mem::replace(&mut self.primitives, primitives);

        self.primitives.push(Primitive::group(layer).clip(bounds));
    }

    /// Starts recording a translation.
    pub fn start_transformation(&mut self) -> Vec<Primitive<B::Primitive>> {
        std::mem::take(&mut self.primitives)
    }

    /// Ends the recording of a translation.
    pub fn end_transformation(
        &mut self,
        primitives: Vec<Primitive<B::Primitive>>,
        transformation: Transformation,
    ) {
        let layer = std::mem::replace(&mut self.primitives, primitives);

        self.primitives
            .push(Primitive::group(layer).transform(transformation));
    }
}

impl<B: Backend> iced_core::Renderer for Renderer<B> {
    fn with_layer(&mut self, bounds: Rectangle, f: impl FnOnce(&mut Self)) {
        let current = self.start_layer();

        f(self);

        self.end_layer(current, bounds);
    }

    fn with_transformation(
        &mut self,
        transformation: Transformation,
        f: impl FnOnce(&mut Self),
    ) {
        let current = self.start_transformation();

        f(self);

        self.end_transformation(current, transformation);
    }

    fn fill_quad(
        &mut self,
        quad: renderer::Quad,
        background: impl Into<Background>,
    ) {
        self.primitives.push(Primitive::Quad {
            bounds: quad.bounds,
            background: background.into(),
            border: quad.border,
            shadow: quad.shadow,
        });
    }

    fn clear(&mut self) {
        self.primitives.clear();
    }
}

impl<B> core::text::Renderer for Renderer<B>
where
    B: Backend + backend::Text,
{
    type Font = Font;
    type Paragraph = text::Paragraph;
    type Editor = text::Editor;

    const ICON_FONT: Font = Font::with_name("Iced-Icons");
    const CHECKMARK_ICON: char = '\u{f00c}';
    const ARROW_DOWN_ICON: char = '\u{e800}';

    fn default_font(&self) -> Self::Font {
        self.default_font
    }

    fn default_size(&self) -> Pixels {
        self.default_text_size
    }

    fn load_font(&mut self, bytes: Cow<'static, [u8]>) {
        self.backend.load_font(bytes);
    }

    fn fill_paragraph(
        &mut self,
        paragraph: &Self::Paragraph,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        self.primitives.push(Primitive::Paragraph {
            paragraph: paragraph.downgrade(),
            position,
            color,
            clip_bounds,
        });
    }

    fn fill_editor(
        &mut self,
        editor: &Self::Editor,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        self.primitives.push(Primitive::Editor {
            editor: editor.downgrade(),
            position,
            color,
            clip_bounds,
        });
    }

    fn fill_text(
        &mut self,
        text: Text<'_, Self::Font>,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        self.primitives.push(Primitive::Text {
            content: text.content.to_string(),
            bounds: Rectangle::new(position, text.bounds),
            size: text.size,
            line_height: text.line_height,
            color,
            font: text.font,
            horizontal_alignment: text.horizontal_alignment,
            vertical_alignment: text.vertical_alignment,
            shaping: text.shaping,
            clip_bounds,
        });
    }
}

impl<B> image::Renderer for Renderer<B>
where
    B: Backend + backend::Image,
{
    type Handle = image::Handle;

    fn dimensions(&self, handle: &image::Handle) -> Size<u32> {
        self.backend().dimensions(handle)
    }

    fn draw(
        &mut self,
        handle: image::Handle,
        filter_method: image::FilterMethod,
        bounds: Rectangle,
    ) {
        self.primitives.push(Primitive::Image {
            handle,
            filter_method,
            bounds,
        });
    }
}

impl<B> svg::Renderer for Renderer<B>
where
    B: Backend + backend::Svg,
{
    fn dimensions(&self, handle: &svg::Handle) -> Size<u32> {
        self.backend().viewport_dimensions(handle)
    }

    fn draw(
        &mut self,
        handle: svg::Handle,
        color: Option<Color>,
        bounds: Rectangle,
    ) {
        self.primitives.push(Primitive::Svg {
            handle,
            color,
            bounds,
        });
    }
}
