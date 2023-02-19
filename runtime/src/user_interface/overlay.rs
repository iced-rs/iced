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
        ) -> layout::Node
        where
            Renderer: renderer::Renderer,
        {
            let translation = position - Point::ORIGIN;

            let node = element.layout(renderer, bounds, translation);

            if let Some(mut nested) =
                element.overlay(Layout::new(&node), renderer)
            {
                layout::Node::with_children(
                    node.size(),
                    vec![
                        node,
                        recurse(&mut nested, renderer, bounds, position),
                    ],
                )
            } else {
                layout::Node::with_children(node.size(), vec![node])
            }
        }

        self.overlay.with_element_mut(|element| {
            recurse(element, renderer, bounds, position)
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
        fn recurse<Message, Renderer>(
            element: &mut overlay::Element<'_, Message, Renderer>,
            layout: Layout<'_>,
            renderer: &mut Renderer,
            theme: &<Renderer as renderer::Renderer>::Theme,
            style: &renderer::Style,
            cursor: mouse::Cursor,
        ) where
            Renderer: renderer::Renderer,
        {
            let mut layouts = layout.children();

            if let Some(layout) = layouts.next() {
                let nested_layout = layouts.next();

                let is_over = cursor
                    .position()
                    .zip(nested_layout)
                    .and_then(|(cursor_position, nested_layout)| {
                        element.overlay(layout, renderer).map(|nested| {
                            nested.is_over(
                                nested_layout,
                                renderer,
                                cursor_position,
                            )
                        })
                    })
                    .unwrap_or_default();

                renderer.with_layer(layout.bounds(), |renderer| {
                    element.draw(
                        renderer,
                        theme,
                        style,
                        layout,
                        if is_over {
                            mouse::Cursor::Unavailable
                        } else {
                            cursor
                        },
                    );
                });

                if let Some((mut nested, nested_layout)) =
                    element.overlay(layout, renderer).zip(nested_layout)
                {
                    recurse(
                        &mut nested,
                        nested_layout,
                        renderer,
                        theme,
                        style,
                        cursor,
                    );
                }
            }
        }

        self.overlay.with_element_mut(|element| {
            recurse(element, layout, renderer, theme, style, cursor);
        })
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        fn recurse<Message, Renderer>(
            element: &mut overlay::Element<'_, Message, Renderer>,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn widget::Operation<Message>,
        ) where
            Renderer: renderer::Renderer,
        {
            let mut layouts = layout.children();

            if let Some(layout) = layouts.next() {
                element.operate(layout, renderer, operation);

                if let Some((mut nested, nested_layout)) =
                    element.overlay(layout, renderer).zip(layouts.next())
                {
                    recurse(&mut nested, nested_layout, renderer, operation);
                }
            }
        }

        recurse(self.overlay.0.get_mut(), layout, renderer, operation)
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
        fn recurse<Message, Renderer>(
            element: &mut overlay::Element<'_, Message, Renderer>,
            layout: Layout<'_>,
            event: Event,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
        ) -> event::Status
        where
            Renderer: renderer::Renderer,
        {
            let mut layouts = layout.children();

            if let Some(layout) = layouts.next() {
                let status = if let Some((mut nested, nested_layout)) =
                    element.overlay(layout, renderer).zip(layouts.next())
                {
                    recurse(
                        &mut nested,
                        nested_layout,
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
                    element.on_event(
                        event, layout, cursor, renderer, clipboard, shell,
                    )
                } else {
                    status
                }
            } else {
                event::Status::Ignored
            }
        }

        recurse(
            self.overlay.0.get_mut(),
            layout,
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
        fn recurse<Message, Renderer>(
            element: &mut overlay::Element<'_, Message, Renderer>,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> Option<mouse::Interaction>
        where
            Renderer: renderer::Renderer,
        {
            let mut layouts = layout.children();

            let layout = layouts.next()?;
            let cursor_position = cursor.position()?;

            if !element.is_over(layout, renderer, cursor_position) {
                return None;
            }

            Some(
                element
                    .overlay(layout, renderer)
                    .zip(layouts.next())
                    .and_then(|(mut overlay, layout)| {
                        recurse(
                            &mut overlay,
                            layout,
                            cursor,
                            viewport,
                            renderer,
                        )
                    })
                    .unwrap_or_else(|| {
                        element.mouse_interaction(
                            layout, cursor, viewport, renderer,
                        )
                    }),
            )
        }

        self.overlay
            .with_element_mut(|element| {
                recurse(element, layout, cursor, viewport, renderer)
            })
            .unwrap_or_default()
    }

    fn is_over(
        &self,
        layout: Layout<'_>,
        renderer: &Renderer,
        cursor_position: Point,
    ) -> bool {
        fn recurse<Message, Renderer>(
            element: &mut overlay::Element<'_, Message, Renderer>,
            layout: Layout<'_>,
            renderer: &Renderer,
            cursor_position: Point,
        ) -> bool
        where
            Renderer: renderer::Renderer,
        {
            let mut layouts = layout.children();

            if let Some(layout) = layouts.next() {
                if element.is_over(layout, renderer, cursor_position) {
                    return true;
                }

                if let Some((mut nested, nested_layout)) =
                    element.overlay(layout, renderer).zip(layouts.next())
                {
                    recurse(
                        &mut nested,
                        nested_layout,
                        renderer,
                        cursor_position,
                    )
                } else {
                    false
                }
            } else {
                false
            }
        }

        self.overlay.with_element_mut(|element| {
            recurse(element, layout, renderer, cursor_position)
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
