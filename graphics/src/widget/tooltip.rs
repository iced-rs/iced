//! Decorate content and apply alignment.
use crate::defaults::Defaults;
use crate::{Backend, Renderer};
use iced_native::{Element, Layout, Point, Rectangle};

/// An element decorating some content.
///
/// This is an alias of an `iced_native` tooltip with a default
/// `Renderer`.
pub type Tooltip<'a, Message, Backend> =
    iced_native::Tooltip<'a, Message, Renderer<Backend>>;

impl<B> iced_native::tooltip::Renderer for Renderer<B>
where
    B: Backend,
{
    type Style = ();

    fn draw<Message>(
        &mut self,
        defaults: &Defaults,
        cursor_position: Point,
        content: &Element<'_, Message, Self>,
        content_layout: Layout<'_>,
        viewport: &Rectangle,
    ) -> Self::Output {
        let (content, mouse_interaction) = content.draw(
            self,
            &defaults,
            content_layout,
            cursor_position,
            viewport,
        );

        (content, mouse_interaction)
    }
}
