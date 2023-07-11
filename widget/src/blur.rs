//! Blur other widgets.
use crate::core::event::Status;
use crate::core::layout::{Limits, Node};
use crate::core::mouse::Cursor;
use crate::core::renderer::Effect;
use crate::core::widget::{Operation, Tree};
use crate::core::{
    mouse, overlay, renderer, Clipboard, Element, Event, Layout, Length,
    Rectangle, Shell, Widget,
};

/// A container which blurs all `content` by a `radius`.
#[allow(missing_debug_implementations)]
pub struct Blur<'a, Message, Renderer = crate::Renderer>
where
    Renderer: crate::core::Renderer,
{
    content: Element<'a, Message, Renderer>,
    radius: u16,
}

impl<'a, Message, Renderer> Blur<'a, Message, Renderer>
where
    Renderer: crate::core::Renderer,
{
    /// Create a new [`Blur`] widget with the specified `radius` and inner `content` to be blurred.
    pub fn new(
        radius: u16,
        content: impl Into<Element<'a, Message, Renderer>>,
    ) -> Self {
        Self {
            content: content.into(),
            radius,
        }
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Blur<'a, Message, Renderer>
where
    Renderer: crate::core::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content))
    }

    fn width(&self) -> Length {
        self.content.as_widget().width()
    }

    fn height(&self) -> Length {
        self.content.as_widget().height()
    }

    fn layout(&self, renderer: &Renderer, limits: &Limits) -> Node {
        self.content.as_widget().layout(renderer, limits)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        self.content.as_widget().operate(
            &mut tree.children[0],
            layout,
            renderer,
            operation,
        );
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> Status {
        self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        // TODO should content still be able to receive mouse events if it's blurred..?
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        if self.radius > 0 {
            renderer.with_layer(
                layout.bounds(),
                Some(Effect::Blur {
                    radius: self.radius,
                }),
                |renderer| {
                    self.content.as_widget().draw(
                        &tree.children[0],
                        renderer,
                        theme,
                        renderer_style,
                        layout,
                        cursor,
                        viewport,
                    );
                },
            );
        } else {
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                renderer_style,
                layout,
                cursor,
                viewport,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
        )
    }
}

impl<'a, Message, Renderer> From<Blur<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + renderer::Renderer,
{
    fn from(
        blur: Blur<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(blur)
    }
}
