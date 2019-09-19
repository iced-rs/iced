//! Display an interactive selector of a single value from a range of values.
//!
//! A [`Slider`] has some local [`State`].
//!
//! [`Slider`]: struct.Slider.html
//! [`State`]: struct.State.html
use std::hash::Hash;
use std::ops::RangeInclusive;
use std::rc::Rc;

use crate::input::{mouse, ButtonState};
use crate::{
    Element, Event, Hasher, Layout, MouseCursor, Node, Point, Rectangle, Style,
    Widget,
};

/// An horizontal bar and a handle that selects a single value from a range of
/// values.
///
/// A [`Slider`] will try to fill the horizontal space of its container.
///
/// It implements [`Widget`] when the associated `Renderer` implements the
/// [`slider::Renderer`] trait.
///
/// [`Slider`]: struct.Slider.html
/// [`Widget`]: ../trait.Widget.html
/// [`slider::Renderer`]: trait.Renderer.html
///
/// # Example
/// ```
/// use iced::{slider, Slider};
///
/// pub enum Message {
///     SliderChanged(f32),
/// }
///
/// let state = &mut slider::State::new();
/// let value = 50.0;
///
/// Slider::new(state, 0.0..=100.0, value, Message::SliderChanged);
/// ```
///
/// ![Slider drawn by Coffee's renderer](https://github.com/hecrj/coffee/blob/bda9818f823dfcb8a7ad0ff4940b4d4b387b5208/images/ui/slider.png?raw=true)
pub struct Slider<'a, Message> {
    state: &'a mut State,
    /// The range of the slider
    pub range: RangeInclusive<f32>,
    /// The current value of the slider
    pub value: f32,
    /// The function to produce messages on change
    pub on_change: Rc<Box<dyn Fn(f32) -> Message>>,
    style: Style,
}

impl<'a, Message> std::fmt::Debug for Slider<'a, Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Slider")
            .field("state", &self.state)
            .field("range", &self.range)
            .field("value", &self.value)
            .field("style", &self.style)
            .finish()
    }
}

impl<'a, Message> Slider<'a, Message> {
    /// Creates a new [`Slider`].
    ///
    /// It expects:
    ///   * the local [`State`] of the [`Slider`]
    ///   * an inclusive range of possible values
    ///   * the current value of the [`Slider`]
    ///   * a function that will be called when the [`Slider`] is dragged.
    ///   It receives the new value of the [`Slider`] and must produce a
    ///   `Message`.
    ///
    /// [`Slider`]: struct.Slider.html
    /// [`State`]: struct.State.html
    pub fn new<F>(
        state: &'a mut State,
        range: RangeInclusive<f32>,
        value: f32,
        on_change: F,
    ) -> Self
    where
        F: 'static + Fn(f32) -> Message,
    {
        Slider {
            state,
            value: value.max(*range.start()).min(*range.end()),
            range,
            on_change: Rc::new(Box::new(on_change)),
            style: Style::default().min_width(100).fill_width(),
        }
    }

    /// Sets the width of the [`Slider`] in pixels.
    ///
    /// [`Slider`]: struct.Slider.html
    pub fn width(mut self, width: u16) -> Self {
        self.style = self.style.width(width);
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for Slider<'a, Message>
where
    Renderer: self::Renderer,
{
    fn node(&self, _renderer: &mut Renderer) -> Node {
        Node::new(self.style.height(25))
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
        renderer.draw(
            cursor_position,
            layout.bounds(),
            self.state,
            self.range.clone(),
            self.value,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.style.hash(state);
    }
}

/// The local state of a [`Slider`].
///
/// [`Slider`]: struct.Slider.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    is_dragging: bool,
}

impl State {
    /// Creates a new [`State`].
    ///
    /// [`State`]: struct.State.html
    pub fn new() -> State {
        State::default()
    }

    /// Returns whether the associated [`Slider`] is currently being dragged or
    /// not.
    ///
    /// [`Slider`]: struct.Slider.html
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
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
    fn draw(
        &mut self,
        cursor_position: Point,
        bounds: Rectangle,
        state: &State,
        range: RangeInclusive<f32>,
        value: f32,
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
