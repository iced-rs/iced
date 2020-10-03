use crate::{Backend, Primitive, Renderer};
use iced_native::column;
use iced_native::mouse;
use iced_native::{Element, Layout, Point};

/// A container that distributes its contents vertically.
pub type Column<'a, Message, Backend> =
    iced_native::Column<'a, Message, Renderer<Backend>>;

impl<B> column::Renderer for Renderer<B>
where
    B: Backend,
{
    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        content: &[Element<'_, Message, Self>],
        layout: Layout<'_>,
        cursor_position: Point,
        draw_at: &mut Option<std::time::Instant>,
    ) -> Self::Output {
        let mut mouse_interaction = mouse::Interaction::default();

        (
            Primitive::Group {
                primitives: content
                    .iter()
                    .zip(layout.children())
                    .map(|(child, layout)| {
                        let mut draw_at_elem = None;
                        let (primitive, new_mouse_interaction) = child.draw(
                            self,
                            defaults,
                            layout,
                            cursor_position,
                            &mut draw_at_elem,
                        );

                        if new_mouse_interaction > mouse_interaction {
                            mouse_interaction = new_mouse_interaction;
                        }

                        // Set draw_at to lower of the child or current values.
                        *draw_at = match (draw_at_elem, draw_at.is_some()) {
                            (Some(dae), false) => Some(dae),
                            (Some(dae), true) => {
                                Some(std::cmp::min(dae, draw_at.unwrap()))
                            }
                            (None, _) => *draw_at,
                        };

                        primitive
                    })
                    .collect(),
            },
            mouse_interaction,
        )
    }
}
