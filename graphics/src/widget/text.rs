//! Write some text for your users to read.
use crate::backend::{self, Backend};
use crate::Renderer;
use iced_native::text;
use iced_native::{Font, Point, Size};

/// A paragraph of text.
///
/// This is an alias of an `iced_native` text with an `iced_wgpu::Renderer`.
pub type Text<Backend> = iced_native::Text<Renderer<Backend>>;

use std::f32;

impl<B> text::Renderer for Renderer<B>
where
    B: Backend + backend::Text,
{
    type Font = Font;

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
}
