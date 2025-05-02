use crate::core::event;
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget;
use crate::core::{Clipboard, Event, Layout, Shell, Size};

/// An overlay container that displays nested overlays
#[allow(missing_debug_implementations)]
pub struct Nested<'a, Message, Theme, Renderer> {
    overlay: overlay::Element<'a, Message, Theme, Renderer>,
}

impl<'a, Message, Theme, Renderer> Nested<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    /// Creates a nested overlay from the provided [`overlay::Element`]
    pub fn new(
        element: overlay::Element<'a, Message, Theme, Renderer>,
    ) -> Self {
        Self { overlay: element }
    }

    /// Returns the layout [`Node`] of the [`Nested`] overlay.
    ///
    /// [`Node`]: layout::Node
    pub fn layout(
        &mut self,
        renderer: &Renderer,
        bounds: Size,
    ) -> layout::Node {
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

            if let Some(mut nested) =
                overlay.overlay(Layout::new(&node), renderer)
            {
                layout::Node::with_children(
                    node.size(),
                    vec![node, recurse(&mut nested, renderer, bounds)],
                )
            } else {
                layout::Node::with_children(node.size(), vec![node])
            }
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

            if let Some(layout) = layouts.next() {
                let nested_layout = layouts.next();
                let overlay = element.as_overlay_mut();

                let is_over = cursor
                    .position()
                    .zip(nested_layout)
                    .and_then(|(cursor_position, nested_layout)| {
                        overlay.overlay(layout, renderer).map(|nested| {
                            nested.as_overlay().mouse_interaction(
                                nested_layout.children().next().unwrap(),
                                mouse::Cursor::Available(cursor_position),
                                renderer,
                            ) != mouse::Interaction::None
                        })
                    })
                    .unwrap_or_default();

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

            if let Some(layout) = layouts.next() {
                let overlay = element.as_overlay_mut();

                overlay.operate(layout, renderer, operation);

                if let Some((mut nested, nested_layout)) =
                    overlay.overlay(layout, renderer).zip(layouts.next())
                {
                    recurse(&mut nested, nested_layout, renderer, operation);
                }
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
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        fn recurse<Message, Theme, Renderer>(
            element: &mut overlay::Element<'_, Message, Theme, Renderer>,
            layout: Layout<'_>,
            event: &Event,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
        ) -> bool
        where
            Renderer: renderer::Renderer,
        {
            let mut layouts = layout.children();

            if let Some(layout) = layouts.next() {
                let overlay = element.as_overlay_mut();

                let nested_is_over = if let Some((mut nested, nested_layout)) =
                    overlay.overlay(layout, renderer).zip(layouts.next())
                {
                    recurse(
                        &mut nested,
                        nested_layout,
                        event,
                        cursor,
                        renderer,
                        clipboard,
                        shell,
                    )
                } else {
                    false
                };

                if shell.event_status() == event::Status::Ignored {
                    let is_over = nested_is_over
                        || cursor
                            .position()
                            .map(|cursor_position| {
                                overlay.mouse_interaction(
                                    layout,
                                    mouse::Cursor::Available(cursor_position),
                                    renderer,
                                ) != mouse::Interaction::None
                            })
                            .unwrap_or_default();

                    overlay.update(
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
                    );

                    is_over
                } else {
                    nested_is_over
                }
            } else {
                false
            }
        }

        let _ = recurse(
            &mut self.overlay,
            layout,
            event,
            cursor,
            renderer,
            clipboard,
            shell,
        );
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
        ) -> Option<mouse::Interaction>
        where
            Renderer: renderer::Renderer,
        {
            let mut layouts = layout.children();

            let layout = layouts.next()?;
            let overlay = element.as_overlay_mut();

            Some(
                overlay
                    .overlay(layout, renderer)
                    .zip(layouts.next())
                    .and_then(|(mut overlay, layout)| {
                        recurse(&mut overlay, layout, cursor, renderer)
                    })
                    .unwrap_or_else(|| {
                        overlay.mouse_interaction(layout, cursor, renderer)
                    }),
            )
        }

        recurse(&mut self.overlay, layout, cursor, renderer).unwrap_or_default()
    }
}
