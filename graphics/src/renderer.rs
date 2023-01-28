//! Create a renderer from a [`Backend`].
use crate::backend::{self, Backend};
use crate::{Primitive, Vector};
use iced_native::image;
use iced_native::layout;
use iced_native::renderer;
use iced_native::svg;
use iced_native::text::{self, Text};
use iced_native::{Background, Color, Element, Font, Point, Rectangle, Size};

pub use iced_native::renderer::Style;

use std::marker::PhantomData;

/// A backend-agnostic renderer that supports all the built-in widgets.
#[derive(Debug)]
pub struct Renderer<B: Backend, Theme> {
    backend: B,
    primitives: Vec<Primitive>,
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

    /// Returns the [`Backend`] of the [`Renderer`].
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// Enqueues the given [`Primitive`] in the [`Renderer`] for drawing.
    pub fn draw_primitive(&mut self, primitive: Primitive) {
        self.primitives.push(primitive);
    }

    /// Runs the given closure with the [`Backend`] and the recorded primitives
    /// of the [`Renderer`].
    pub fn with_primitives(&mut self, f: impl FnOnce(&mut B, &[Primitive])) {
        f(&mut self.backend, &self.primitives);
    }
}

impl<B, T> iced_native::Renderer for Renderer<B, T>
where
    B: Backend,
{
    type Theme = T;

    fn layout<Message>(
        &mut self,
        element: &Element<'_, Message, Self>,
        limits: &layout::Limits,
    ) -> layout::Node {
        let layout = element.as_widget().layout(self, limits);

        self.backend.trim_measurements();

        layout
    }

    fn with_layer(&mut self, bounds: Rectangle, f: impl FnOnce(&mut Self)) {
        let current_primitives = std::mem::take(&mut self.primitives);

        f(self);

        let layer_primitives =
            std::mem::replace(&mut self.primitives, current_primitives);

        self.primitives.push(Primitive::Clip {
            bounds,
            content: Box::new(Primitive::Group {
                primitives: layer_primitives,
            }),
        });
    }

    fn with_translation(
        &mut self,
        translation: Vector,
        f: impl FnOnce(&mut Self),
    ) {
        let current_primitives = std::mem::take(&mut self.primitives);

        f(self);

        let layer_primitives =
            std::mem::replace(&mut self.primitives, current_primitives);

        self.primitives.push(Primitive::Translate {
            translation,
            content: Box::new(Primitive::Group {
                primitives: layer_primitives,
            }),
        });
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

    fn default_size(&self) -> u16 {
        self.backend().default_size()
    }

    fn measure(
        &self,
        content: &str,
        size: u16,
        font: Font,
        bounds: Size,
    ) -> (f32, f32) {
        self.backend()
            .measure(content, f32::from(size), font, bounds)
    }

    fn hit_test(
        &self,
        content: &str,
        size: f32,
        font: Font,
        bounds: Size,
        point: Point,
        nearest_only: bool,
    ) -> Option<text::Hit> {
        self.backend().hit_test(
            content,
            size,
            font,
            bounds,
            point,
            nearest_only,
        )
    }

    fn fill_text(&mut self, text: Text<'_, Self::Font>) {
        self.primitives.push(Primitive::Text {
            content: text.content.to_string(),
            bounds: text.bounds,
            size: text.size,
            color: text.color,
            font: text.font,
            horizontal_alignment: text.horizontal_alignment,
            vertical_alignment: text.vertical_alignment,
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
        self.draw_primitive(Primitive::Image { handle, bounds })
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
        self.draw_primitive(Primitive::Svg {
            handle,
            color,
            bounds,
        })
    }
}
