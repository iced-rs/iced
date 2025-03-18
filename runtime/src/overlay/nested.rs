use crate::core::event;
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget;
use crate::core::{
    Clipboard, Event, Layout, Mouse, Point, Rectangle, Shell, Size,
};

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
            let node = element.layout(renderer, bounds);

            if let Some(mut nested) =
                element.overlay(Layout::new(&node), renderer)
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
        mouse: Mouse,
    ) {
        fn recurse<Message, Theme, Renderer>(
            element: &mut overlay::Element<'_, Message, Theme, Renderer>,
            layout: Layout<'_>,
            renderer: &mut Renderer,
            theme: &Theme,
            style: &renderer::Style,
            mouse: Mouse,
        ) where
            Renderer: renderer::Renderer,
        {
            let mut layouts = layout.children();

            if let Some(layout) = layouts.next() {
                let nested_layout = layouts.next();

                let is_over = mouse
                    .position()
                    .zip(nested_layout)
                    .and_then(|(mouse_position, nested_layout)| {
                        element.overlay(layout, renderer).map(|nested| {
                            nested.is_over(
                                nested_layout.children().next().unwrap(),
                                renderer,
                                mouse_position,
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
                        if is_over { Mouse::Unavailable } else { mouse },
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
                        mouse,
                    );
                }
            }
        }

        recurse(&mut self.overlay, layout, renderer, theme, style, mouse);
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
                element.operate(layout, renderer, operation);

                if let Some((mut nested, nested_layout)) =
                    element.overlay(layout, renderer).zip(layouts.next())
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
        mouse: Mouse,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        fn recurse<Message, Theme, Renderer>(
            element: &mut overlay::Element<'_, Message, Theme, Renderer>,
            layout: Layout<'_>,
            event: &Event,
            mouse: Mouse,
            renderer: &Renderer,
            clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
        ) -> bool
        where
            Renderer: renderer::Renderer,
        {
            let mut layouts = layout.children();

            if let Some(layout) = layouts.next() {
                let nested_is_over = if let Some((mut nested, nested_layout)) =
                    element.overlay(layout, renderer).zip(layouts.next())
                {
                    recurse(
                        &mut nested,
                        nested_layout,
                        event,
                        mouse,
                        renderer,
                        clipboard,
                        shell,
                    )
                } else {
                    false
                };

                if shell.event_status() == event::Status::Ignored {
                    let is_over = nested_is_over
                        || mouse
                            .position()
                            .map(|mouse_position| {
                                element.is_over(
                                    layout,
                                    renderer,
                                    mouse_position,
                                )
                            })
                            .unwrap_or_default();

                    element.update(
                        event,
                        layout,
                        if nested_is_over {
                            Mouse::Unavailable
                        } else {
                            mouse
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
            mouse,
            renderer,
            clipboard,
            shell,
        );
    }

    /// Returns the current [`mouse::Interaction`] of the [`Nested`] overlay.
    pub fn mouse_interaction(
        &mut self,
        layout: Layout<'_>,
        mouse: Mouse,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        fn recurse<Message, Theme, Renderer>(
            element: &mut overlay::Element<'_, Message, Theme, Renderer>,
            layout: Layout<'_>,
            mouse: Mouse,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> Option<mouse::Interaction>
        where
            Renderer: renderer::Renderer,
        {
            let mut layouts = layout.children();

            let layout = layouts.next()?;
            let mouse_position = mouse.position()?;

            if !element.is_over(layout, renderer, mouse_position) {
                return None;
            }

            Some(
                element
                    .overlay(layout, renderer)
                    .zip(layouts.next())
                    .and_then(|(mut overlay, layout)| {
                        recurse(&mut overlay, layout, mouse, viewport, renderer)
                    })
                    .unwrap_or_else(|| {
                        element.mouse_interaction(
                            layout, mouse, viewport, renderer,
                        )
                    }),
            )
        }

        recurse(&mut self.overlay, layout, mouse, viewport, renderer)
            .unwrap_or_default()
    }

    /// Returns true if the cursor is over the [`Nested`] overlay.
    pub fn is_over(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        mouse_position: Point,
    ) -> bool {
        fn recurse<Message, Theme, Renderer>(
            element: &mut overlay::Element<'_, Message, Theme, Renderer>,
            layout: Layout<'_>,
            renderer: &Renderer,
            mouse_position: Point,
        ) -> bool
        where
            Renderer: renderer::Renderer,
        {
            let mut layouts = layout.children();

            if let Some(layout) = layouts.next() {
                if element.is_over(layout, renderer, mouse_position) {
                    return true;
                }

                if let Some((mut nested, nested_layout)) =
                    element.overlay(layout, renderer).zip(layouts.next())
                {
                    recurse(
                        &mut nested,
                        nested_layout,
                        renderer,
                        mouse_position,
                    )
                } else {
                    false
                }
            } else {
                false
            }
        }

        recurse(&mut self.overlay, layout, renderer, mouse_position)
    }
}
