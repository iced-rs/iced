//! Display a horizontal or vertical rule for dividing content.

use std::hash::Hash;

use crate::{
    layout, Element, Hasher, Layout, Length, Point, Rectangle, Size, Widget,
};

/// Display a horizontal or vertical rule for dividing content.
#[derive(Debug, Copy, Clone)]
pub struct Rule<Renderer: self::Renderer> {
    width: Length,
    height: Length,
    style: Renderer::Style,
    is_horizontal: bool,
}

impl<Renderer: self::Renderer> Rule<Renderer> {
    /// Creates a horizontal [`Rule`] for dividing content by the given vertical spacing.
    ///
    /// [`Rule`]: struct.Rule.html
    pub fn horizontal(spacing: u16) -> Self {
        Rule {
            width: Length::Fill,
            height: Length::from(Length::Units(spacing)),
            style: Renderer::Style::default(),
            is_horizontal: true,
        }
    }

    /// Creates a vertical [`Rule`] for dividing content by the given horizontal spacing.
    ///
    /// [`Rule`]: struct.Rule.html
    pub fn vertical(spacing: u16) -> Self {
        Rule {
            width: Length::from(Length::Units(spacing)),
            height: Length::Fill,
            style: Renderer::Style::default(),
            is_horizontal: false,
        }
    }

    /// Sets the style of the [`Rule`].
    ///
    /// [`Rule`]: struct.Rule.html
    pub fn style(mut self, style: impl Into<Renderer::Style>) -> Self {
        self.style = style.into();
        self
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for Rule<Renderer>
where
    Renderer: self::Renderer,
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
        _defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) -> Renderer::Output {
        renderer.draw(layout.bounds(), &self.style, self.is_horizontal)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
        self.height.hash(state);
    }
}

/// The renderer of a [`Rule`].
///
/// [`Rule`]: struct.Rule.html
pub trait Renderer: crate::Renderer {
    /// The style supported by this renderer.
    type Style: Default;

    /// Draws a [`Rule`].
    ///
    /// It receives:
    ///   * the bounds of the [`Rule`]
    ///   * the style of the [`Rule`]
    ///   * whether the [`Rule`] is horizontal (true) or vertical (false)
    ///
    /// [`Rule`]: struct.Rule.html
    fn draw(
        &mut self,
        bounds: Rectangle,
        style: &Self::Style,
        is_horizontal: bool,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<Rule<Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer,
    Message: 'a,
{
    fn from(rule: Rule<Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(rule)
    }
}
