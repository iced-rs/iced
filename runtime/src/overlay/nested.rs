use crate::core::event;
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget;
use crate::core::{Clipboard, Event, Layout, Point, Rectangle, Shell, Size};

/// An [`Overlay`] container that displays nested overlays
#[allow(missing_debug_implementations)]
pub struct Nested<'a, Message, Renderer> {
    overlay: overlay::Element<'a, Message, Renderer>,
}

impl<'a, Message, Renderer> Nested<'a, Message, Renderer>
where
    Renderer: renderer::Renderer,
{
    /// Creates a nested overlay from the provided [`overlay::Element`]
    pub fn new(element: overlay::Element<'a, Message, Renderer>) -> Self {
        Self { overlay: element }
    }

    /// Returns the position of the [`Nested`] overlay.
    pub fn position(&self) -> Point {
        self.overlay.position()
    }

    /// Returns the layout [`Node`] of the [`Nested`] overlay.
    pub fn layout(
        &mut self,
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

        recurse(&mut self.overlay, renderer, bounds, position)
    }

    /// Draws the [`Nested`] overlay using the associated `Renderer`.
    pub fn draw(
        &mut self,
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
                                nested_layout.children().next().unwrap(),
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

        recurse(&mut self.overlay, layout, renderer, theme, style, cursor);
    }

    /// Applies a [`widget::Operation`] to the [`Nested`] overlay.
    pub fn operate(
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

        recurse(&mut self.overlay, layout, renderer, operation)
    }

    /// Processes a runtime [`Event`].
    pub fn on_event(
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
        ) -> (event::Status, bool)
        where
            Renderer: renderer::Renderer,
        {
            let mut layouts = layout.children();

            if let Some(layout) = layouts.next() {
                let (nested_status, nested_is_over) =
                    if let Some((mut nested, nested_layout)) =
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
                        (event::Status::Ignored, false)
                    };

                if matches!(nested_status, event::Status::Ignored) {
                    let is_over = nested_is_over
                        || cursor
                            .position()
                            .map(|cursor_position| {
                                element.is_over(
                                    layout,
                                    renderer,
                                    cursor_position,
                                )
                            })
                            .unwrap_or_default();

                    (
                        element.on_event(
                            event,
                            layout,
                            if nested_is_over {
                                mouse::Cursor::Unavailable
                            } else {
                                cursor
                            },
                            renderer,
                            clipboard,
                            shell,
                        ),
                        is_over,
                    )
                } else {
                    (nested_status, nested_is_over)
                }
            } else {
                (event::Status::Ignored, false)
            }
        }

        let (status, _) = recurse(
            &mut self.overlay,
            layout,
            event,
            cursor,
            renderer,
            clipboard,
            shell,
        );

        status
    }

    /// Returns the current [`mouse::Interaction`] of the [`Nested`] overlay.
    pub fn mouse_interaction(
        &mut self,
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

        recurse(&mut self.overlay, layout, cursor, viewport, renderer)
            .unwrap_or_default()
    }

    /// Returns true if the cursor is over the [`Nested`] overlay.
    pub fn is_over(
        &mut self,
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

        recurse(&mut self.overlay, layout, renderer, cursor_position)
    }
}
