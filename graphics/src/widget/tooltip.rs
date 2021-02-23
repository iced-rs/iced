//! Decorate content and apply alignment.
use crate::backend::{self, Backend};
use crate::defaults::Defaults;
use crate::{Primitive, Renderer, Vector};

use iced_native::layout::{self, Layout};
use iced_native::{Element, Point, Rectangle, Size, Text};

/// An element decorating some content.
///
/// This is an alias of an `iced_native` tooltip with a default
/// `Renderer`.
pub type Tooltip<'a, Message, Backend> =
    iced_native::Tooltip<'a, Message, Renderer<Backend>>;

pub use iced_native::tooltip::Position;

impl<B> iced_native::tooltip::Renderer for Renderer<B>
where
    B: Backend + backend::Text,
{
    type Style = ();

    fn draw<Message>(
        &mut self,
        defaults: &Defaults,
        cursor_position: Point,
        content_layout: Layout<'_>,
        viewport: &Rectangle,
        content: &Element<'_, Message, Self>,
        tooltip: &Text<Self>,
        position: Position,
    ) -> Self::Output {
        let bounds = content_layout.bounds();

        let (content, mouse_interaction) = content.draw(
            self,
            &defaults,
            content_layout,
            cursor_position,
            viewport,
        );

        if bounds.contains(cursor_position) {
            use iced_native::Widget;

            let tooltip_layout = Widget::<(), Self>::layout(
                tooltip,
                self,
                &layout::Limits::new(Size::ZERO, viewport.size()),
            );

            let tooltip_bounds = tooltip_layout.bounds();

            let x_center =
                bounds.x + (bounds.width - tooltip_bounds.width) / 2.0;

            let y_center =
                bounds.y + (bounds.height - tooltip_bounds.height) / 2.0;

            let offset = match position {
                Position::Top => {
                    Vector::new(x_center, bounds.y - tooltip_bounds.height)
                }
                Position::Bottom => {
                    Vector::new(x_center, bounds.y + bounds.height)
                }
                Position::Left => {
                    Vector::new(bounds.x - tooltip_bounds.width, y_center)
                }
                Position::Right => {
                    Vector::new(bounds.x + bounds.width, y_center)
                }
                Position::FollowCursor => Vector::new(
                    cursor_position.x,
                    cursor_position.y - tooltip_bounds.height,
                ),
            };

            let (tooltip, _) = Widget::<(), Self>::draw(
                tooltip,
                self,
                defaults,
                Layout::with_offset(offset, &tooltip_layout),
                cursor_position,
                viewport,
            );

            (
                Primitive::Clip {
                    bounds: *viewport,
                    offset: Vector::new(0, 0),
                    content: Box::new(Primitive::Group {
                        primitives: vec![content, tooltip],
                    }),
                },
                mouse_interaction,
            )
        } else {
            (content, mouse_interaction)
        }
    }
}
