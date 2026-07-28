//! Add semantic metadata to widgets.

use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::metadata::{Metadata, Role};
use crate::core::widget::tree::Tree;
use crate::core::{Element, Event, Layout, Length, Rectangle, Shell, Size, Vector, Widget};

/// A widget that adds semantic metadata to its contents.
pub struct Semantics<'a, Message, Theme, Renderer = crate::Renderer>
where
    Renderer: crate::core::Renderer,
{
    content: Element<'a, Message, Theme, Renderer>,
    metadata: Metadata,
}

impl<'a, Message, Theme, Renderer> Semantics<'a, Message, Theme, Renderer>
where
    Renderer: crate::core::Renderer,
{
    /// Creates new [`Semantics`] with the given [`Role`] and `content`.
    pub fn new(role: Role, content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            metadata: Metadata::new(role),
        }
    }

    /// Sets the user-facing label.
    pub fn label(mut self, label: impl Into<crate::core::SmolStr>) -> Self {
        self.metadata.label = Some(label.into());
        self
    }

    /// Sets the test identifier.
    pub fn test_id(mut self, test_id: impl Into<crate::core::SmolStr>) -> Self {
        self.metadata.test_id = Some(test_id.into());
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Semantics<'_, Message, Theme, Renderer>
where
    Renderer: crate::core::Renderer,
{
    fn tag(&self) -> crate::core::widget::tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> crate::core::widget::tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content.as_widget_mut().layout(tree, renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn crate::core::widget::Operation,
    ) {
        operation.metadata(None, layout.bounds(), &self.metadata);

        self.content
            .as_widget_mut()
            .operate(tree, layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content
            .as_widget_mut()
            .update(tree, event, layout, cursor, renderer, shell, viewport);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(tree, layout, renderer, viewport, translation)
    }
}

impl<'a, Message, Theme, Renderer> From<Semantics<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: 'a + crate::core::Renderer,
{
    fn from(semantics: Semantics<'a, Message, Theme, Renderer>) -> Self {
        Element::new(semantics)
    }
}
