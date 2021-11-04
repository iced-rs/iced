//! Display a horizontal or vertical rule for dividing content.
use crate::layout;
use crate::renderer;
use crate::{
    Color, Element, Hasher, Layout, Length, Point, Rectangle, Size, Widget,
};

use std::hash::Hash;

pub use iced_style::rule::{FillMode, Style, StyleSheet};

/// Display a horizontal or vertical rule for dividing content.
#[allow(missing_debug_implementations)]
pub struct Rule<'a> {
    width: Length,
    height: Length,
    is_horizontal: bool,
    style_sheet: Box<dyn StyleSheet + 'a>,
}

impl<'a> Rule<'a> {
    /// Creates a horizontal [`Rule`] for dividing content by the given vertical spacing.
    pub fn horizontal(spacing: u16) -> Self {
        Rule {
            width: Length::Fill,
            height: Length::from(Length::Units(spacing)),
            is_horizontal: true,
            style_sheet: Default::default(),
        }
    }

    /// Creates a vertical [`Rule`] for dividing content by the given horizontal spacing.
    pub fn vertical(spacing: u16) -> Self {
        Rule {
            width: Length::from(Length::Units(spacing)),
            height: Length::Fill,
            is_horizontal: false,
            style_sheet: Default::default(),
        }
    }

    /// Sets the style of the [`Rule`].
    pub fn style(
        mut self,
        style_sheet: impl Into<Box<dyn StyleSheet + 'a>>,
    ) -> Self {
        self.style_sheet = style_sheet.into();
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for Rule<'a>
where
    Renderer: crate::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);

        layout::Node::new(limits.resolve(Size::ZERO))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let style = self.style_sheet.style();

        let bounds = if self.is_horizontal {
            let line_y = (bounds.y + (bounds.height / 2.0)
                - (style.width as f32 / 2.0))
                .round();

            let (offset, line_width) = style.fill_mode.fill(bounds.width);
            let line_x = bounds.x + offset;

            Rectangle {
                x: line_x,
                y: line_y,
                width: line_width,
                height: style.width as f32,
            }
        } else {
            let line_x = (bounds.x + (bounds.width / 2.0)
                - (style.width as f32 / 2.0))
                .round();

            let (offset, line_height) = style.fill_mode.fill(bounds.height);
            let line_y = bounds.y + offset;

            Rectangle {
                x: line_x,
                y: line_y,
                width: style.width as f32,
                height: line_height,
            }
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border_radius: style.radius,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            style.color,
        );
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
        self.height.hash(state);
    }
}

impl<'a, Message, Renderer> From<Rule<'a>> for Element<'a, Message, Renderer>
where
    Renderer: 'a + crate::Renderer,
    Message: 'a,
{
    fn from(rule: Rule<'a>) -> Element<'a, Message, Renderer> {
        Element::new(rule)
    }
}
