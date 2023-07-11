//! Create a renderer from a [`Backend`].
use crate::backend::{self, Backend};
use crate::Primitive;

use iced_core::image;
use iced_core::layout;
use iced_core::renderer::{self, Effect};
use iced_core::svg;
use iced_core::text::{self, Text};
use iced_core::{
    Background, Color, Element, Font, Point, Rectangle, Size, Vector,
};

use std::borrow::Cow;
use std::marker::PhantomData;

/// A backend-agnostic renderer that supports all the built-in widgets.
#[derive(Debug)]
pub struct Renderer<B: Backend, Theme> {
    backend: B,
    primitives: Vec<Primitive<B::Primitive>>,
    theme: PhantomData<Theme>,
}

impl<B: Backend, T> Renderer<B, T> {
    /// Creates a new [`Renderer`] from the given [`Backend`].
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            primitives: Vec::new(),
            theme: PhantomData,
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
        effect: Option<Effect>,
        bounds: Rectangle,
    ) {
        let layer = std::mem::replace(&mut self.primitives, primitives);

        if let Some(effect) = effect {
            self.primitives
                .push(Primitive::group(layer).effect(bounds, effect));
        } else {
            self.primitives.push(Primitive::group(layer).clip(bounds));
        }
    }

    /// Starts recording a translation.
    pub fn start_translation(&mut self) -> Vec<Primitive<B::Primitive>> {
        std::mem::take(&mut self.primitives)
    }

    /// Ends the recording of a translation.
    pub fn end_translation(
        &mut self,
        primitives: Vec<Primitive<B::Primitive>>,
        translation: Vector,
    ) {
        let layer = std::mem::replace(&mut self.primitives, primitives);

        self.primitives
            .push(Primitive::group(layer).translate(translation));
    }
}

impl<B: Backend, T> iced_core::Renderer for Renderer<B, T> {
    type Theme = T;

    fn layout<Message>(
        &mut self,
        element: &Element<'_, Message, Self>,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.backend.trim_measurements();

        element.as_widget().layout(self, limits)
    }

    fn with_layer(
        &mut self,
        bounds: Rectangle,
        effect: Option<Effect>,
        f: impl FnOnce(&mut Self),
    ) {
        let current = self.start_layer();

        f(self);

        self.end_layer(current, effect, bounds);
    }

    fn with_translation(
        &mut self,
        translation: Vector,
        f: impl FnOnce(&mut Self),
    ) {
        let current = self.start_translation();

        f(self);

        self.end_translation(current, translation);
    }

    fn fill_quad(
        &mut self,
        quad: renderer::Quad,
        background: impl Into<Background>,
    ) {
        self.primitives.push(Primitive::Quad {
            bounds: quad.bounds,
            background: background.into(),
            border_radius: quad.border_radius.into(),
            border_width: quad.border_width,
            border_color: quad.border_color,
        });
    }

    fn clear(&mut self) {
        self.primitives.clear();
    }
}

impl<B, T> text::Renderer for Renderer<B, T>
where
    B: Backend + backend::Text,
{
    type Font = Font;

    const ICON_FONT: Font = B::ICON_FONT;
    const CHECKMARK_ICON: char = B::CHECKMARK_ICON;
    const ARROW_DOWN_ICON: char = B::ARROW_DOWN_ICON;

    fn default_font(&self) -> Self::Font {
        self.backend().default_font()
    }

    fn default_size(&self) -> f32 {
        self.backend().default_size()
    }

    fn measure(
        &self,
        content: &str,
        size: f32,
        line_height: text::LineHeight,
        font: Font,
        bounds: Size,
        shaping: text::Shaping,
    ) -> Size {
        self.backend().measure(
            content,
            size,
            line_height,
            font,
            bounds,
            shaping,
        )
    }

    fn hit_test(
        &self,
        content: &str,
        size: f32,
        line_height: text::LineHeight,
        font: Font,
        bounds: Size,
        shaping: text::Shaping,
        point: Point,
        nearest_only: bool,
    ) -> Option<text::Hit> {
        self.backend().hit_test(
            content,
            size,
            line_height,
            font,
            bounds,
            shaping,
            point,
            nearest_only,
        )
    }

    fn load_font(&mut self, bytes: Cow<'static, [u8]>) {
        self.backend.load_font(bytes);
    }

    fn fill_text(&mut self, text: Text<'_, Self::Font>) {
        self.primitives.push(Primitive::Text {
            content: text.content.to_string(),
            bounds: text.bounds,
            size: text.size,
            line_height: text.line_height,
            color: text.color,
            font: text.font,
            horizontal_alignment: text.horizontal_alignment,
            vertical_alignment: text.vertical_alignment,
            shaping: text.shaping,
        });
    }
}

impl<B, T> image::Renderer for Renderer<B, T>
where
    B: Backend + backend::Image,
{
    type Handle = image::Handle;

    fn dimensions(&self, handle: &image::Handle) -> Size<u32> {
        self.backend().dimensions(handle)
    }

    fn draw(&mut self, handle: image::Handle, bounds: Rectangle) {
        self.primitives.push(Primitive::Image { handle, bounds })
    }
}

impl<B, T> svg::Renderer for Renderer<B, T>
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
        })
    }
}
