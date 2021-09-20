//! Write some text for your users to read.
use crate::backend::{self, Backend};
use crate::{Primitive, Renderer};
use iced_native::alignment;
use iced_native::mouse;
use iced_native::text;
use iced_native::{Color, Font, Point, Rectangle, Size};

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

    fn draw(
        &mut self,
        defaults: &Self::Defaults,
        bounds: Rectangle,
        content: &str,
        size: u16,
        font: Font,
        color: Option<Color>,
        horizontal_alignment: alignment::Horizontal,
        vertical_alignment: alignment::Vertical,
    ) -> Self::Output {
        let x = match horizontal_alignment {
            alignment::Horizontal::Left => bounds.x,
            alignment::Horizontal::Center => bounds.center_x(),
            alignment::Horizontal::Right => bounds.x + bounds.width,
        };

        let y = match vertical_alignment {
            alignment::Vertical::Top => bounds.y,
            alignment::Vertical::Center => bounds.center_y(),
            alignment::Vertical::Bottom => bounds.y + bounds.height,
        };

        (
            Primitive::Text {
                content: content.to_string(),
                size: f32::from(size),
                bounds: Rectangle { x, y, ..bounds },
                color: color.unwrap_or(defaults.text.color),
                font,
                horizontal_alignment,
                vertical_alignment,
            },
            mouse::Interaction::default(),
        )
    }
}
