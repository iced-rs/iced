//! Show toggle controls using checkboxes.
use std::hash::Hash;

use crate::input::{mouse, ButtonState};
use crate::{layout, Element, Event, Hasher, Layout, Point, Widget};

pub use iced_core::Checkbox;

impl<Message, Renderer> Widget<Message, Renderer> for Checkbox<Message>
where
    Renderer: self::Renderer,
{
    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> Layout {
        renderer.layout(&self, limits)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: &Layout,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        _renderer: &Renderer,
    ) {
        match event {
            Event::Mouse(mouse::Event::Input {
                button: mouse::Button::Left,
                state: ButtonState::Pressed,
            }) => {
                let mouse_over = layout.bounds().contains(cursor_position);

                if mouse_over {
                    messages.push((self.on_toggle)(!self.is_checked));
                }
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: &Layout,
        cursor_position: Point,
    ) -> Renderer::Output {
        renderer.draw(&self, layout, cursor_position)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.label.hash(state);
    }
}

/// The renderer of a [`Checkbox`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`Checkbox`] in your user interface.
///
/// [`Checkbox`]: struct.Checkbox.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: crate::Renderer {
    /// Creates a [`Node`] for the provided [`Checkbox`].
    ///
    /// [`Node`]: ../../struct.Node.html
    /// [`Checkbox`]: struct.Checkbox.html
    fn layout<Message>(
        &self,
        checkbox: &Checkbox<Message>,
        limits: &layout::Limits,
    ) -> Layout;

    /// Draws a [`Checkbox`].
    ///
    /// It receives:
    ///   * the current cursor position
    ///   * the bounds of the [`Checkbox`]
    ///   * the bounds of the label of the [`Checkbox`]
    ///   * whether the [`Checkbox`] is checked or not
    ///
    /// [`Checkbox`]: struct.Checkbox.html
    fn draw<Message>(
        &mut self,
        checkbox: &Checkbox<Message>,
        layout: &Layout,
        cursor_position: Point,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<Checkbox<Message>>
    for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer,
    Message: 'static,
{
    fn from(checkbox: Checkbox<Message>) -> Element<'a, Message, Renderer> {
        Element::new(checkbox)
    }
}
