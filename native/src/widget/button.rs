//! Allow your users to perform actions by pressing a button.
//!
//! A [`Button`] has some local [`State`] and a [`Class`].
//!
//! [`Button`]: struct.Button.html
//! [`State`]: struct.State.html
//! [`Class`]: enum.Class.html

use crate::input::{mouse, ButtonState};
use crate::{Element, Event, Hasher, Layout, Node, Point, Widget};
use std::hash::Hash;

pub use iced_core::button::State;

pub type Button<'a, Message, Renderer> =
    iced_core::Button<'a, Message, Element<'a, Message, Renderer>>;

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Button<'a, Message, Renderer>
where
    Renderer: self::Renderer,
    Message: Copy + std::fmt::Debug,
{
    fn node(&self, renderer: &Renderer) -> Node {
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
                state,
            }) => {
                if let Some(on_press) = self.on_press {
                    let bounds = layout.bounds();

                    match state {
                        ButtonState::Pressed => {
                            self.state.is_pressed =
                                bounds.contains(cursor_position);
                        }
                        ButtonState::Released => {
                            let is_clicked = self.state.is_pressed
                                && bounds.contains(cursor_position);

                            self.state.is_pressed = false;

                            if is_clicked {
                                messages.push(on_press);
                            }
                        }
                    }
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
        self.width.hash(state);
        self.align_self.hash(state);
        self.content.hash_layout(state);
    }
}

/// The renderer of a [`Button`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`Button`] in your user interface.
///
/// [`Button`]: struct.Button.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: crate::Renderer + Sized {
    /// Creates a [`Node`] for the provided [`Button`].
    ///
    /// [`Node`]: ../../struct.Node.html
    /// [`Button`]: struct.Button.html
    fn node<Message>(&self, button: &Button<'_, Message, Self>) -> Node;

    /// Draws a [`Button`].
    ///
    /// [`Button`]: struct.Button.html
    fn draw<Message>(
        &mut self,
        button: &Button<'_, Message, Self>,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Self::Primitive;
}

impl<'a, Message, Renderer> From<Button<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'static + self::Renderer,
    Message: 'static + Copy + std::fmt::Debug,
{
    fn from(
        button: Button<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(button)
    }
}
