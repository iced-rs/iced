use crate::event;
use crate::layout;
use crate::mouse;
use crate::overlay;
use crate::renderer;
use crate::widget;
use crate::{Event, Layout, Shell, Size};

/// A container of nested overlays.
pub struct Nested<'a, Message, Theme, Renderer> {
    children: Vec<overlay::Element<'a, Message, Theme, Renderer>>,
}

fn sort_overlays<'a, Message, Theme, Renderer>(
    children: &mut [overlay::Element<'a, Message, Theme, Renderer>],
) where
    Renderer: renderer::Renderer,
{
    use std::cmp;

    children.sort_by(|a, b| {
        a.as_overlay()
            .index()
            .partial_cmp(&b.as_overlay().index())
            .unwrap_or(cmp::Ordering::Equal)
    });
}

impl<'a, Message, Theme, Renderer> Nested<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    /// Creates a [`Nested`] container for the given overlays.
    ///
    /// The overlays are sorted by their
    /// [`index`](crate::Overlay::index).
    pub fn new(mut children: Vec<overlay::Element<'a, Message, Theme, Renderer>>) -> Self {
        sort_overlays(&mut children);

        Self { children }
    }

    /// Returns the layout [`Node`] of the [`Nested`] overlay.
    ///
    /// [`Node`]: layout::Node
    pub fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        fn recurse<Message, Theme, Renderer>(
            children: &mut [overlay::Element<'_, Message, Theme, Renderer>],
            renderer: &Renderer,
            bounds: Size,
        ) -> layout::Node
        where
            Renderer: renderer::Renderer,
        {
            let children = children
                .iter_mut()
                .map(|element| {
                    let overlay = element.as_overlay_mut();
                    let node = overlay.layout(renderer, bounds);

                    let mut nested = overlay.overlay(Layout::new(&node), renderer);

                    if nested.is_empty() {
                        drop(nested);

                        layout::Node::with_children(node.size(), vec![node])
                    } else {
                        sort_overlays(&mut nested);

                        let nested_node = recurse(&mut nested, renderer, bounds);
                        drop(nested);

                        layout::Node::with_children(node.size(), vec![node, nested_node])
                    }
                })
                .collect();

            layout::Node::with_children(bounds, children)
        }

        recurse(&mut self.children, renderer, bounds)
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
            children: &mut [overlay::Element<'_, Message, Theme, Renderer>],
            layout: Layout<'_>,
            renderer: &mut Renderer,
            theme: &Theme,
            style: &renderer::Style,
            cursor: mouse::Cursor,
        ) where
            Renderer: renderer::Renderer,
        {
            for (element, wrap_layout) in children.iter_mut().zip(layout.children()) {
                let mut layouts = wrap_layout.children();

                let Some(layout) = layouts.next() else {
                    continue;
                };
                let nested_layout = layouts.next();

                // TODO: Get rid of cursor argument in `draw`
                let is_over = cursor.position().zip(nested_layout).is_some_and(
                    |(cursor_position, nested_layout)| {
                        let nested = element.as_overlay_mut().overlay(layout, renderer);

                        !nested.is_empty()
                            && Nested::new(nested).mouse_interaction(
                                nested_layout,
                                mouse::Cursor::Available(cursor_position),
                                renderer,
                            ) != mouse::Interaction::None
                    },
                );

                renderer.with_layer(layout.bounds(), |renderer| {
                    element.as_overlay().draw(
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

                if let Some(nested_layout) = nested_layout {
                    let mut nested = element.as_overlay_mut().overlay(layout, renderer);

                    if !nested.is_empty() {
                        sort_overlays(&mut nested);

                        recurse(&mut nested, nested_layout, renderer, theme, style, cursor);
                    }
                }
            }
        }

        recurse(&mut self.children, layout, renderer, theme, style, cursor);
    }

    /// Applies a [`widget::Operation`] to the [`Nested`] overlay.
    pub fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        fn recurse<Message, Theme, Renderer>(
            children: &mut [overlay::Element<'_, Message, Theme, Renderer>],
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn widget::Operation,
        ) where
            Renderer: renderer::Renderer,
        {
            for (element, wrap_layout) in children.iter_mut().zip(layout.children()) {
                let mut layouts = wrap_layout.children();

                let Some(layout) = layouts.next() else {
                    continue;
                };
                let nested_layout = layouts.next();

                let overlay = element.as_overlay_mut();

                overlay.operate(layout, renderer, operation);

                if let Some(nested_layout) = nested_layout {
                    let mut nested = overlay.overlay(layout, renderer);

                    if !nested.is_empty() {
                        sort_overlays(&mut nested);

                        recurse(&mut nested, nested_layout, renderer, operation);
                    }
                }
            }
        }

        recurse(&mut self.children, layout, renderer, operation);
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
            children: &mut [overlay::Element<'_, Message, Theme, Renderer>],
            layout: Layout<'_>,
            event: &Event,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            shell: &mut Shell<'_, Message>,
        ) -> bool
        where
            Renderer: renderer::Renderer,
        {
            let mut is_over = false;

            for (element, wrap_layout) in children.iter_mut().zip(layout.children()) {
                if shell.event_status() != event::Status::Ignored {
                    return is_over;
                }

                let mut layouts = wrap_layout.children();

                let Some(layout) = layouts.next() else {
                    continue;
                };
                let nested_layout = layouts.next();

                let overlay = element.as_overlay_mut();

                let nested = overlay.overlay(layout, renderer);
                let nested_is_over = (!nested.is_empty())
                    .then_some(nested)
                    .zip(nested_layout)
                    .map(|(mut nested, nested_layout)| {
                        sort_overlays(&mut nested);

                        recurse(&mut nested, nested_layout, event, cursor, renderer, shell)
                    })
                    .unwrap_or_default();

                if shell.event_status() != event::Status::Ignored {
                    return nested_is_over || is_over;
                }

                let child_is_over = nested_is_over
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

                is_over |= child_is_over;
            }

            is_over
        }

        let _ = recurse(&mut self.children, layout, event, cursor, renderer, shell);
    }

    /// Returns the current [`mouse::Interaction`] of the [`Nested`] overlay.
    pub fn mouse_interaction(
        &mut self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        fn recurse<Message, Theme, Renderer>(
            children: &mut [overlay::Element<'_, Message, Theme, Renderer>],
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &Renderer,
        ) -> mouse::Interaction
        where
            Renderer: renderer::Renderer,
        {
            children
                .iter_mut()
                .zip(layout.children())
                .map(|(element, wrap_layout)| {
                    let mut layouts = wrap_layout.children();

                    let Some(layout) = layouts.next() else {
                        return mouse::Interaction::None;
                    };
                    let nested_layout = layouts.next();

                    let overlay = element.as_overlay_mut();
                    let interaction = overlay.mouse_interaction(layout, cursor, renderer);

                    let nested = overlay.overlay(layout, renderer);
                    let nested_interaction = (!nested.is_empty())
                        .then_some(nested)
                        .zip(nested_layout)
                        .map(|(mut nested, nested_layout)| {
                            sort_overlays(&mut nested);

                            recurse(&mut nested, nested_layout, cursor, renderer)
                        })
                        .unwrap_or_default();

                    nested_interaction.max(interaction)
                })
                .max()
                .unwrap_or_default()
        }

        recurse(&mut self.children, layout, cursor, renderer)
    }
}
