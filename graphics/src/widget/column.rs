use crate::{Backend, Primitive, Renderer};
use iced_native::column;
use iced_native::mouse;
use iced_native::{Element, Layout, Point, Rectangle};

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
        viewport: &Rectangle,
    ) -> Self::Output {
        let mut mouse_interaction = mouse::Interaction::default();

        (
            Primitive::Group {
                primitives: content
                    .iter()
                    .zip(layout.children())
                    .map(|(child, layout)| {
                        let (primitive, new_mouse_interaction) = child.draw(
                            self,
                            defaults,
                            layout,
                            cursor_position,
                            viewport,
                        );

                        if new_mouse_interaction > mouse_interaction {
                            mouse_interaction = new_mouse_interaction;
                        }

                        primitive
                    })
                    .collect(),
            },
            mouse_interaction,
        )
    }
}
