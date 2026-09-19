use crate::event;
use crate::mouse;
use crate::overlay;
use crate::renderer;
use crate::widget;
use crate::{Event, Shell};

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

    /// Draws the [`Nested`] overlay using the associated `Renderer`.
    pub fn draw(
        &mut self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        cursor: mouse::Cursor,
    ) {
        fn recurse<Message, Theme, Renderer>(
            children: &mut [overlay::Element<'_, Message, Theme, Renderer>],
            renderer: &mut Renderer,
            theme: &Theme,
            style: &renderer::Style,
            cursor: mouse::Cursor,
        ) where
            Renderer: renderer::Renderer,
        {
            for element in children {
                // TODO: Get rid of cursor argument in `draw`
                let is_over = cursor.position().is_some_and(|cursor_position| {
                    let nested = element.as_overlay_mut().overlay(renderer);

                    !nested.is_empty()
                        && Nested::new(nested)
                            .mouse_interaction(mouse::Cursor::Available(cursor_position), renderer)
                            != mouse::Interaction::None
                });

                element.as_overlay().draw(
                    renderer,
                    theme,
                    style,
                    if is_over {
                        mouse::Cursor::Unavailable
                    } else {
                        cursor
                    },
                );

                let mut nested = element.as_overlay_mut().overlay(renderer);

                if !nested.is_empty() {
                    sort_overlays(&mut nested);

                    recurse(&mut nested, renderer, theme, style, cursor);
                }
            }
        }

        recurse(&mut self.children, renderer, theme, style, cursor);
    }

    /// Applies a [`widget::Operation`] to the [`Nested`] overlay.
    pub fn operate(&mut self, renderer: &Renderer, operation: &mut dyn widget::Operation) {
        fn recurse<Message, Theme, Renderer>(
            children: &mut [overlay::Element<'_, Message, Theme, Renderer>],
            renderer: &Renderer,
            operation: &mut dyn widget::Operation,
        ) where
            Renderer: renderer::Renderer,
        {
            for element in children {
                let overlay = element.as_overlay_mut();

                overlay.operate(renderer, operation);

                let mut nested = overlay.overlay(renderer);

                if !nested.is_empty() {
                    sort_overlays(&mut nested);

                    recurse(&mut nested, renderer, operation);
                }
            }
        }

        recurse(&mut self.children, renderer, operation);
    }

    /// Processes a runtime [`Event`].
    pub fn update(
        &mut self,
        event: &Event,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
    ) {
        fn recurse<Message, Theme, Renderer>(
            children: &mut [overlay::Element<'_, Message, Theme, Renderer>],
            event: &Event,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            shell: &mut Shell<'_, Message>,
        ) -> bool
        where
            Renderer: renderer::Renderer,
        {
            let mut is_over = false;

            for element in children {
                if shell.event_status() != event::Status::Ignored {
                    return is_over;
                }

                let overlay = element.as_overlay_mut();

                let nested = overlay.overlay(renderer);
                let nested_is_over = (!nested.is_empty())
                    .then_some(nested)
                    .map(|mut nested| {
                        sort_overlays(&mut nested);

                        recurse(&mut nested, event, cursor, renderer, shell)
                    })
                    .unwrap_or_default();

                if shell.event_status() != event::Status::Ignored {
                    return nested_is_over || is_over;
                }

                let child_is_over = nested_is_over
                    || cursor.position().is_some_and(|cursor_position| {
                        overlay
                            .mouse_interaction(mouse::Cursor::Available(cursor_position), renderer)
                            != mouse::Interaction::None
                    });

                overlay.update(
                    event,
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

        let _ = recurse(&mut self.children, event, cursor, renderer, shell);
    }

    /// Returns the current [`mouse::Interaction`] of the [`Nested`] overlay.
    pub fn mouse_interaction(
        &mut self,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        fn recurse<Message, Theme, Renderer>(
            children: &mut [overlay::Element<'_, Message, Theme, Renderer>],
            cursor: mouse::Cursor,
            renderer: &Renderer,
        ) -> mouse::Interaction
        where
            Renderer: renderer::Renderer,
        {
            children
                .iter_mut()
                .map(|element| {
                    let overlay = element.as_overlay_mut();
                    let interaction = overlay.mouse_interaction(cursor, renderer);

                    let nested = overlay.overlay(renderer);
                    let nested_interaction = (!nested.is_empty())
                        .then_some(nested)
                        .map(|mut nested| {
                            sort_overlays(&mut nested);

                            recurse(&mut nested, cursor, renderer)
                        })
                        .unwrap_or_default();

                    nested_interaction.max(interaction)
                })
                .max()
                .unwrap_or_default()
        }

        recurse(&mut self.children, cursor, renderer)
    }
}
