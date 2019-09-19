//! Show toggle controls using checkboxes.
use std::hash::Hash;

use crate::input::{mouse, ButtonState};
use crate::widget::{text, Column, Row, Text};
use crate::{
    Align, Color, Element, Event, Hasher, Layout, MouseCursor, Node, Point,
    Rectangle, Widget,
};

/// A box that can be checked.
///
/// It implements [`Widget`] when the associated `Renderer` implements the
/// [`checkbox::Renderer`] trait.
///
/// [`Widget`]: ../trait.Widget.html
/// [`checkbox::Renderer`]: trait.Renderer.html
///
/// # Example
///
/// ```
/// use iced::{Checkbox, Color};
///
/// pub enum Message {
///     CheckboxToggled(bool),
/// }
///
/// let is_checked = true;
///
/// Checkbox::new(is_checked, "Toggle me!", Message::CheckboxToggled);
/// ```
///
/// ![Checkbox drawn by Coffee's renderer](https://github.com/hecrj/coffee/blob/bda9818f823dfcb8a7ad0ff4940b4d4b387b5208/images/ui/checkbox.png?raw=true)
pub struct Checkbox<Message> {
    /// Whether the checkbox is checked or not
    pub is_checked: bool,

    /// Function to call when checkbox is toggled to produce a __message__.
    ///
    /// The function should be provided `true` when the checkbox is checked
    /// and `false` otherwise.
    pub on_toggle: Box<dyn Fn(bool) -> Message>,

    /// The label of the checkbox
    pub label: String,

    /// The color of the label
    pub label_color: Option<Color>,
}

impl<Message> std::fmt::Debug for Checkbox<Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Checkbox")
            .field("is_checked", &self.is_checked)
            .field("label", &self.label)
            .field("label_color", &self.label_color)
            .finish()
    }
}

impl<Message> Checkbox<Message> {
    /// Creates a new [`Checkbox`].
    ///
    /// It expects:
    ///   * a boolean describing whether the [`Checkbox`] is checked or not
    ///   * the label of the [`Checkbox`]
    ///   * a function that will be called when the [`Checkbox`] is toggled.
    ///     It will receive the new state of the [`Checkbox`] and must produce
    ///     a `Message`.
    ///
    /// [`Checkbox`]: struct.Checkbox.html
    pub fn new<F>(is_checked: bool, label: &str, f: F) -> Self
    where
        F: 'static + Fn(bool) -> Message,
    {
        Checkbox {
            is_checked,
            on_toggle: Box::new(f),
            label: String::from(label),
            label_color: None,
        }
    }

    /// Sets the color of the label of the [`Checkbox`].
    ///
    /// [`Checkbox`]: struct.Checkbox.html
    pub fn label_color<C: Into<Color>>(mut self, color: C) -> Self {
        self.label_color = Some(color.into());
        self
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for Checkbox<Message>
where
    Renderer: self::Renderer + text::Renderer,
{
    fn node(&self, renderer: &mut Renderer) -> Node {
        Row::<(), Renderer>::new()
            .spacing(15)
            .align_items(Align::Center)
            .push(Column::new().width(28).height(28))
            .push(Text::new(&self.label))
            .node(renderer)
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
                let mouse_over = layout
                    .children()
                    .any(|child| child.bounds().contains(cursor_position));

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
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> MouseCursor {
        let children: Vec<_> = layout.children().collect();

        let text_bounds = children[1].bounds();

        text::Renderer::draw(
            renderer,
            text_bounds,
            &self.label,
            None,
            self.label_color,
            text::HorizontalAlignment::Left,
            text::VerticalAlignment::Top,
        );

        self::Renderer::draw(
            renderer,
            cursor_position,
            children[0].bounds(),
            text_bounds,
            self.is_checked,
        )
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
pub trait Renderer {
    /// Draws a [`Checkbox`].
    ///
    /// It receives:
    ///   * the current cursor position
    ///   * the bounds of the [`Checkbox`]
    ///   * the bounds of the label of the [`Checkbox`]
    ///   * whether the [`Checkbox`] is checked or not
    ///
    /// [`Checkbox`]: struct.Checkbox.html
    fn draw(
        &mut self,
        cursor_position: Point,
        bounds: Rectangle,
        label_bounds: Rectangle,
        is_checked: bool,
    ) -> MouseCursor;
}

impl<'a, Message, Renderer> From<Checkbox<Message>>
    for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer + text::Renderer,
    Message: 'static,
{
    fn from(checkbox: Checkbox<Message>) -> Element<'a, Message, Renderer> {
        Element::new(checkbox)
    }
}
