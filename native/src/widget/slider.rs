//! Display an interactive selector of a single value from a range of values.
//!
//! A [`Slider`] has some local [`State`].
//!
//! [`Slider`]: struct.Slider.html
//! [`State`]: struct.State.html
use std::hash::Hash;

use crate::input::{mouse, ButtonState};
use crate::{Element, Event, Hasher, Layout, MouseCursor, Node, Point, Widget};

pub use iced_core::slider::*;

impl<'a, Message, Renderer> Widget<Message, Renderer> for Slider<'a, Message>
where
    Renderer: self::Renderer,
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
        let mut change = || {
            let bounds = layout.bounds();

            if cursor_position.x <= bounds.x {
                messages.push((self.on_change)(*self.range.start()));
            } else if cursor_position.x >= bounds.x + bounds.width {
                messages.push((self.on_change)(*self.range.end()));
            } else {
                let percent = (cursor_position.x - bounds.x) / bounds.width;
                let value = (self.range.end() - self.range.start()) * percent
                    + self.range.start();

                messages.push((self.on_change)(value));
            }
        };

        match event {
            Event::Mouse(mouse::Event::Input {
                button: mouse::Button::Left,
                state,
            }) => match state {
                ButtonState::Pressed => {
                    if layout.bounds().contains(cursor_position) {
                        change();
                        self.state.is_dragging = true;
                    }
                }
                ButtonState::Released => {
                    self.state.is_dragging = false;
                }
            },
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if self.state.is_dragging {
                    change();
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
    ) -> MouseCursor {
        renderer.draw(&self, layout, cursor_position)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.width.hash(state);
    }
}

/// The renderer of a [`Slider`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`Slider`] in your user interface.
///
/// [`Slider`]: struct.Slider.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer {
    /// Creates a [`Node`] for the provided [`Radio`].
    ///
    /// [`Node`]: ../../struct.Node.html
    /// [`Radio`]: struct.Radio.html
    fn node<Message>(&self, slider: &Slider<'_, Message>) -> Node;

    /// Draws a [`Slider`].
    ///
    /// It receives:
    ///   * the current cursor position
    ///   * the bounds of the [`Slider`]
    ///   * the local state of the [`Slider`]
    ///   * the range of values of the [`Slider`]
    ///   * the current value of the [`Slider`]
    ///
    /// [`Slider`]: struct.Slider.html
    /// [`State`]: struct.State.html
    /// [`Class`]: enum.Class.html
    fn draw<Message>(
        &mut self,
        slider: &Slider<'_, Message>,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> MouseCursor;
}

impl<'a, Message, Renderer> From<Slider<'a, Message>>
    for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer,
    Message: 'static,
{
    fn from(slider: Slider<'a, Message>) -> Element<'a, Message, Renderer> {
        Element::new(slider)
    }
}
