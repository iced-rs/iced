use crate::event;
use crate::layout;
use crate::mouse;
use crate::overlay;
use crate::renderer;
use crate::widget;
use crate::{Event, Layout, Shell, Size};

/// An overlay container that displays nested overlays
pub struct Nested<'a, Message, Theme, Renderer> {
    overlay: overlay::Element<'a, Message, Theme, Renderer>,
}

impl<'a, Message, Theme, Renderer> Nested<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    /// Creates a nested overlay from the provided [`overlay::Element`]
    pub fn new(element: overlay::Element<'a, Message, Theme, Renderer>) -> Self {
        Self { overlay: element }
    }

    /// Returns the layout [`Node`] of the [`Nested`] overlay.
    ///
    /// [`Node`]: layout::Node
    pub fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        fn recurse<Message, Theme, Renderer>(
            element: &mut overlay::Element<'_, Message, Theme, Renderer>,
            renderer: &Renderer,
            bounds: Size,
        ) -> layout::Node
        where
            Renderer: renderer::Renderer,
        {
            let overlay = element.as_overlay_mut();
            let node = overlay.layout(renderer, bounds);

            let nested_node = overlay
                .overlay(Layout::new(&node), renderer)
                .as_mut()
                .map(|nested| recurse(nested, renderer, bounds));

            layout::Node::with_children(
                node.size(),
                if let Some(nested_node) = nested_node {
                    vec![node, nested_node]
                } else {
                    vec![node]
                },
            )
        }

        recurse(&mut self.overlay, renderer, bounds)
    }

    /// Draws the [`Nested`] overlay using the associated `Renderer`.
    pub fn draw(
        &mut self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        fn recurse<Message, Theme, Renderer>(
            element: &mut overlay::Element<'_, Message, Theme, Renderer>,
            layout: Layout<'_>,
            renderer: &mut Renderer,
            theme: &Theme,
            style: &renderer::Style,
            cursor: mouse::Cursor,
        ) where
            Renderer: renderer::Renderer,
        {
            let mut layouts = layout.children();
            let Some(layout) = layouts.next() else {
                return;
            };
            let nested_layout = layouts.next();
            let overlay = element.as_overlay_mut();

            let is_over = cursor.position().zip(nested_layout).is_some_and(
                |(cursor_position, nested_layout)| {
                    overlay.overlay(layout, renderer).is_some_and(|nested| {
                        nested.as_overlay().mouse_interaction(
                            nested_layout.children().next().unwrap(),
                            mouse::Cursor::Available(cursor_position),
                            renderer,
                        ) != mouse::Interaction::None
                    })
                },
            );

            renderer.with_layer(layout.bounds(), |renderer| {
                overlay.draw(
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
                overlay.overlay(layout, renderer).zip(nested_layout)
            {
                recurse(&mut nested, nested_layout, renderer, theme, style, cursor);
            }
        }

        recurse(&mut self.overlay, layout, renderer, theme, style, cursor);
    }

    /// Applies a [`widget::Operation`] to the [`Nested`] overlay.
    pub fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        fn recurse<Message, Theme, Renderer>(
            element: &mut overlay::Element<'_, Message, Theme, Renderer>,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn widget::Operation,
        ) where
            Renderer: renderer::Renderer,
        {
            let mut layouts = layout.children();

            let Some(layout) = layouts.next() else {
                return;
            };

            let overlay = element.as_overlay_mut();

            overlay.operate(layout, renderer, operation);

            if let Some((mut nested, nested_layout)) =
                overlay.overlay(layout, renderer).zip(layouts.next())
            {
                recurse(&mut nested, nested_layout, renderer, operation);
            }
        }

        recurse(&mut self.overlay, layout, renderer, operation);
    }

    /// Processes a runtime [`Event`].
    pub fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
    ) {
        fn recurse<Message, Theme, Renderer>(
            element: &mut overlay::Element<'_, Message, Theme, Renderer>,
            layout: Layout<'_>,
            event: &Event,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            shell: &mut Shell<'_, Message>,
        ) -> bool
        where
            Renderer: renderer::Renderer,
        {
            let mut layouts = layout.children();

            let Some(layout) = layouts.next() else {
                return false;
            };

            let overlay = element.as_overlay_mut();

            let nested_is_over = overlay
                .overlay(layout, renderer)
                .zip(layouts.next())
                .map(|(mut nested, nested_layout)| {
                    recurse(&mut nested, nested_layout, event, cursor, renderer, shell)
                })
                .unwrap_or_default();

            if shell.event_status() != event::Status::Ignored {
                return nested_is_over;
            }

            let is_over = nested_is_over
                || cursor.position().is_some_and(|cursor_position| {
                    overlay.mouse_interaction(
                        layout,
                        mouse::Cursor::Available(cursor_position),
                        renderer,
                    ) != mouse::Interaction::None
                });

            overlay.update(
                event,
                layout,
                if nested_is_over {
                    mouse::Cursor::Unavailable
                } else {
                    cursor
                },
                renderer,
                shell,
            );

            is_over
        }

        let _ = recurse(&mut self.overlay, layout, event, cursor, renderer, shell);
    }

    /// Returns the current [`mouse::Interaction`] of the [`Nested`] overlay.
    pub fn mouse_interaction(
        &mut self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        fn recurse<Message, Theme, Renderer>(
            element: &mut overlay::Element<'_, Message, Theme, Renderer>,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &Renderer,
        ) -> mouse::Interaction
        where
            Renderer: renderer::Renderer,
        {
            let mut layouts = layout.children();

            let Some(layout) = layouts.next() else {
                return mouse::Interaction::None;
            };

            let overlay = element.as_overlay_mut();
            let interaction = overlay.mouse_interaction(layout, cursor, renderer);

            let nested_interaction = overlay
                .overlay(layout, renderer)
                .zip(layouts.next())
                .map(|(mut nested, nested_layout)| {
                    recurse(&mut nested, nested_layout, cursor, renderer)
                })
                .unwrap_or_default();

            nested_interaction.max(interaction)
        }

        recurse(&mut self.overlay, layout, cursor, renderer)
    }
}
