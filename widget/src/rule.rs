//! Display a horizontal or vertical rule for dividing content.
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::widget::Tree;
use crate::core::{
    Border, Element, Layout, Length, Pixels, Rectangle, Size, Widget,
};

pub use crate::style::rule::{Appearance, FillMode, StyleSheet};

/// Display a horizontal or vertical rule for dividing content.
#[allow(missing_debug_implementations)]
pub struct Rule<Theme = crate::Theme>
where
    Theme: StyleSheet,
{
    width: Length,
    height: Length,
    is_horizontal: bool,
    style: Theme::Style,
}

impl<Theme> Rule<Theme>
where
    Theme: StyleSheet,
{
    /// Creates a horizontal [`Rule`] with the given height.
    pub fn horizontal(height: impl Into<Pixels>) -> Self {
        Rule {
            width: Length::Fill,
            height: Length::Fixed(height.into().0),
            is_horizontal: true,
            style: Default::default(),
        }
    }

    /// Creates a vertical [`Rule`] with the given width.
    pub fn vertical(width: impl Into<Pixels>) -> Self {
        Rule {
            width: Length::Fixed(width.into().0),
            height: Length::Fill,
            is_horizontal: false,
            style: Default::default(),
        }
    }

    /// Sets the style of the [`Rule`].
    pub fn style(mut self, style: impl Into<Theme::Style>) -> Self {
        self.style = style.into();
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Rule<Theme>
where
    Renderer: crate::core::Renderer,
    Theme: StyleSheet,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn draw(
        &self,
        _state: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let style = theme.appearance(&self.style);

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
                border: Border::with_radius(style.radius),
                ..renderer::Quad::default()
            },
            style.color,
        );
    }
}

impl<'a, Message, Theme, Renderer> From<Rule<Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: StyleSheet + 'a,
    Renderer: 'a + crate::core::Renderer,
{
    fn from(rule: Rule<Theme>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(rule)
    }
}
