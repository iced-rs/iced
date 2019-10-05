//! Create choices using radio buttons.
use crate::input::{mouse, ButtonState};
use crate::{Element, Event, Hasher, Layout, Node, Point, Widget};

use std::hash::Hash;

pub use iced_core::Radio;

impl<Message, Renderer> Widget<Message, Renderer> for Radio<Message>
where
    Renderer: self::Renderer,
    Message: Copy + std::fmt::Debug,
{
    fn node(&self, renderer: &mut Renderer) -> Node {
        renderer.node(&self)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
    ) {
        match event {
            Event::Mouse(mouse::Event::Input {
                button: mouse::Button::Left,
                state: ButtonState::Pressed,
            }) => {
                if layout.bounds().contains(cursor_position) {
                    messages.push(self.on_click);
                }
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Primitive {
        renderer.draw(&self, layout, cursor_position)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.label.hash(state);
    }
}

/// The renderer of a [`Radio`] button.
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`Radio`] button in your user interface.
///
/// [`Radio`]: struct.Radio.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: crate::Renderer {
    /// Creates a [`Node`] for the provided [`Radio`].
    ///
    /// [`Node`]: ../../struct.Node.html
    /// [`Radio`]: struct.Radio.html
    fn node<Message>(&mut self, radio: &Radio<Message>) -> Node;

    /// Draws a [`Radio`] button.
    ///
    /// It receives:
    ///   * the current cursor position
    ///   * the bounds of the [`Radio`]
    ///   * the bounds of the label of the [`Radio`]
    ///   * whether the [`Radio`] is selected or not
    ///
    /// [`Radio`]: struct.Radio.html
    fn draw<Message>(
        &mut self,
        radio: &Radio<Message>,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Self::Primitive;
}

impl<'a, Message, Renderer> From<Radio<Message>>
    for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer,
    Message: 'static + Copy + std::fmt::Debug,
{
    fn from(checkbox: Radio<Message>) -> Element<'a, Message, Renderer> {
        Element::new(checkbox)
    }
}
