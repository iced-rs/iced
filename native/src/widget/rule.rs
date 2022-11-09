//! Display a horizontal or vertical rule for dividing content.
use crate::layout;
use crate::renderer;
use crate::widget::Tree;
use crate::{Color, Element, Layout, Length, Point, Rectangle, Size, Widget};

pub use iced_style::rule::{Appearance, FillMode, StyleSheet};

/// Display a horizontal or vertical rule for dividing content.
#[allow(missing_debug_implementations)]
pub struct Rule<Renderer>
where
    Renderer: crate::Renderer,
    Renderer::Theme: StyleSheet,
{
    width: Length,
    height: Length,
    is_horizontal: bool,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<Renderer> Rule<Renderer>
where
    Renderer: crate::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Creates a horizontal [`Rule`] with the given height.
    pub fn horizontal(height: u16) -> Self {
        Rule {
            width: Length::Fill,
            height: Length::Units(height),
            is_horizontal: true,
            style: Default::default(),
        }
    }

    /// Creates a vertical [`Rule`] with the given width.
    pub fn vertical(width: u16) -> Self {
        Rule {
            width: Length::Units(width),
            height: Length::Fill,
            is_horizontal: false,
            style: Default::default(),
        }
    }

    /// Sets the style of the [`Rule`].
    pub fn style(
        mut self,
        style: impl Into<<Renderer::Theme as StyleSheet>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for Rule<Renderer>
where
    Renderer: crate::Renderer,
    Renderer::Theme: StyleSheet,
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
        _state: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor_position: Point,
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
                border_radius: style.radius,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            style.color,
        );
    }
}

impl<'a, Message, Renderer> From<Rule<Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + crate::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn from(rule: Rule<Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(rule)
    }
}
