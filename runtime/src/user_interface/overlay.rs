use crate::core::event;
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget;
use crate::core::{
    Clipboard, Event, Layout, Overlay, Point, Rectangle, Shell, Size,
};

use std::cell::RefCell;

/// An [`Overlay`] container that displays nested overlays
#[allow(missing_debug_implementations)]
pub struct Nested<'a, Message, Renderer> {
    overlay: Inner<'a, Message, Renderer>,
}

impl<'a, Message, Renderer> Nested<'a, Message, Renderer> {
    /// Creates a nested overlay from the provided [`overlay::Element`]
    pub fn new(element: overlay::Element<'a, Message, Renderer>) -> Self {
        Self {
            overlay: Inner(RefCell::new(element)),
        }
    }
}

struct Inner<'a, Message, Renderer>(
    RefCell<overlay::Element<'a, Message, Renderer>>,
);

impl<'a, Message, Renderer> Inner<'a, Message, Renderer> {
    fn with_element_mut<T>(
        &self,
        mut f: impl FnMut(&mut overlay::Element<'_, Message, Renderer>) -> T,
    ) -> T {
        (f)(&mut self.0.borrow_mut())
    }
}

impl<'a, Message, Renderer> Overlay<Message, Renderer>
    for Nested<'a, Message, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn layout(
        &self,
        renderer: &Renderer,
        bounds: Size,
        position: Point,
    ) -> layout::Node {
        fn recurse<Message, Renderer>(
            element: &mut overlay::Element<'_, Message, Renderer>,
            renderer: &Renderer,
            bounds: Size,
            position: Point,
        ) -> Vec<layout::Node>
        where
            Renderer: renderer::Renderer,
        {
            let translation = position - Point::ORIGIN;

            let node = element.layout(renderer, bounds, translation);

            if let Some(mut overlay) =
                element.overlay(Layout::new(&node), renderer)
            {
                vec![node]
                    .into_iter()
                    .chain(recurse(&mut overlay, renderer, bounds, position))
                    .collect()
            } else {
                vec![node]
            }
        }

        self.overlay.with_element_mut(|element| {
            layout::Node::with_children(
                bounds,
                recurse(element, renderer, bounds, position),
            )
        })
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &<Renderer as renderer::Renderer>::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        fn recurse<'a, Message, Renderer>(
            element: &mut overlay::Element<'_, Message, Renderer>,
            mut layouts: impl Iterator<Item = Layout<'a>>,
            renderer: &mut Renderer,
            theme: &<Renderer as renderer::Renderer>::Theme,
            style: &renderer::Style,
            cursor: mouse::Cursor,
        ) where
            Renderer: renderer::Renderer,
        {
            let layout = layouts.next().unwrap();

            renderer.with_layer(layout.bounds(), |renderer| {
                element.draw(renderer, theme, style, layout, cursor);
            });

            if let Some(mut overlay) = element.overlay(layout, renderer) {
                recurse(&mut overlay, layouts, renderer, theme, style, cursor);
            }
        }

        self.overlay.with_element_mut(|element| {
            let layouts = layout.children();

            recurse(element, layouts, renderer, theme, style, cursor);
        })
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        fn recurse<'a, Message, Renderer>(
            element: &mut overlay::Element<'_, Message, Renderer>,
            mut layouts: impl Iterator<Item = Layout<'a>>,
            renderer: &Renderer,
            operation: &mut dyn widget::Operation<Message>,
        ) where
            Renderer: renderer::Renderer,
        {
            let layout = layouts.next().unwrap();

            element.operate(layout, renderer, operation);

            if let Some(mut overlay) = element.overlay(layout, renderer) {
                recurse(&mut overlay, layouts, renderer, operation);
            }
        }

        let layouts = layout.children();

        recurse(self.overlay.0.get_mut(), layouts, renderer, operation)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        fn recurse<'a, Message, Renderer>(
            element: &mut overlay::Element<'_, Message, Renderer>,
            mut layouts: impl Iterator<Item = Layout<'a>>,
            event: Event,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
        ) -> event::Status
        where
            Renderer: renderer::Renderer,
        {
            let layout = layouts.next().unwrap();

            let status =
                if let Some(mut overlay) = element.overlay(layout, renderer) {
                    recurse(
                        &mut overlay,
                        layouts,
                        event.clone(),
                        cursor,
                        renderer,
                        clipboard,
                        shell,
                    )
                } else {
                    event::Status::Ignored
                };

            if matches!(status, event::Status::Ignored) {
                element
                    .on_event(event, layout, cursor, renderer, clipboard, shell)
            } else {
                status
            }
        }

        let layouts = layout.children();

        recurse(
            self.overlay.0.get_mut(),
            layouts,
            event,
            cursor,
            renderer,
            clipboard,
            shell,
        )
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        fn recurse<'a, Message, Renderer>(
            element: &mut overlay::Element<'_, Message, Renderer>,
            mut layouts: impl Iterator<Item = Layout<'a>>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> mouse::Interaction
        where
            Renderer: renderer::Renderer,
        {
            let layout = layouts.next().unwrap();

            let interaction =
                if let Some(mut overlay) = element.overlay(layout, renderer) {
                    recurse(&mut overlay, layouts, cursor, viewport, renderer)
                } else {
                    mouse::Interaction::default()
                };

            element
                .mouse_interaction(layout, cursor, viewport, renderer)
                .max(interaction)
        }

        self.overlay.with_element_mut(|element| {
            let layouts = layout.children();

            recurse(element, layouts, cursor, viewport, renderer)
        })
    }

    fn is_over(
        &self,
        layout: Layout<'_>,
        renderer: &Renderer,
        cursor_position: Point,
    ) -> bool {
        fn recurse<'a, Message, Renderer>(
            element: &mut overlay::Element<'_, Message, Renderer>,
            mut layouts: impl Iterator<Item = Layout<'a>>,
            renderer: &Renderer,
            cursor_position: Point,
        ) -> bool
        where
            Renderer: renderer::Renderer,
        {
            let layout = layouts.next().unwrap();

            let is_over = element.is_over(layout, renderer, cursor_position);

            if is_over {
                return true;
            }

            if let Some(mut overlay) = element.overlay(layout, renderer) {
                recurse(&mut overlay, layouts, renderer, cursor_position)
            } else {
                false
            }
        }

        self.overlay.with_element_mut(|element| {
            let layouts = layout.children();

            recurse(element, layouts, renderer, cursor_position)
        })
    }

    fn overlay<'b>(
        &'b mut self,
        _layout: Layout<'_>,
        _renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        None
    }
}
