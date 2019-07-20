//! Show toggle controls using checkboxes.
use std::hash::Hash;

use crate::input::{mouse, ButtonState};
use crate::widget::{text, Column, Row, Text};
use crate::{
    Align, Element, Event, Hasher, Layout, MouseCursor, Node, Point, Rectangle,
    Widget,
};

/// A box that can be checked.
///
/// It implements [`Widget`] when the [`core::Renderer`] implements the
/// [`checkbox::Renderer`] trait.
///
/// [`Widget`]: ../../core/trait.Widget.html
/// [`core::Renderer`]: ../../core/trait.Renderer.html
/// [`checkbox::Renderer`]: trait.Renderer.html
///
/// # Example
///
/// ```
/// use iced::Checkbox;
///
/// #[derive(Debug, Clone, Copy)]
/// pub enum Color {
///     Black,
///     White,
/// }
///
/// impl Default for Color {
///     fn default() -> Color {
///         Color::Black
///     }
/// }
///
/// pub enum Message {
///     CheckboxToggled(bool),
/// }
///
/// fn some_checkbox(is_checked: bool) -> Checkbox<Color, Message> {
///     Checkbox::new(is_checked, "Toggle me!", Message::CheckboxToggled)
///         .label_color(Color::White)
/// }
/// ```
pub struct Checkbox<Color, Message> {
    is_checked: bool,
    on_toggle: Box<dyn Fn(bool) -> Message>,
    label: String,
    label_color: Color,
}

impl<Color, Message> std::fmt::Debug for Checkbox<Color, Message>
where
    Color: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Checkbox")
            .field("is_checked", &self.is_checked)
            .field("label", &self.label)
            .field("label_color", &self.label_color)
            .finish()
    }
}

impl<Color, Message> Checkbox<Color, Message>
where
    Color: Default,
{
    /// Creates a new [`Checkbox`].
    ///
    /// It expects:
    ///   * a boolean describing whether the [`Checkbox`] is checked or not
    ///   * the label of the [`Checkbox`]
    ///   * a function that will be called when the [`Checkbox`] is toggled.
    ///   It receives the new state of the [`Checkbox`] and must produce a
    ///   `Message`.
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
            label_color: Color::default(),
        }
    }

    /// Sets the [`Color`] of the label of the [`Checkbox`].
    ///
    /// [`Color`]: ../../../../graphics/struct.Color.html
    /// [`Checkbox`]: struct.Checkbox.html
    pub fn label_color(mut self, color: Color) -> Self {
        self.label_color = color;
        self
    }
}

impl<Color, Message, Renderer> Widget<Message, Renderer>
    for Checkbox<Color, Message>
where
    Color: 'static + Copy + Default + std::fmt::Debug,
    Renderer: self::Renderer + text::Renderer<Color>,
{
    fn node(&self, renderer: &Renderer) -> Node {
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
            20.0,
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

    fn hash(&self, state: &mut Hasher) {
        self.label.hash(state);
    }
}

/// The renderer of a [`Checkbox`].
///
/// Your [`core::Renderer`] will need to implement this trait before being
/// able to use a [`Checkbox`] in your user interface.
///
/// [`Checkbox`]: struct.Checkbox.html
/// [`core::Renderer`]: ../../core/trait.Renderer.html
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
        bounds: Rectangle<f32>,
        label_bounds: Rectangle<f32>,
        is_checked: bool,
    ) -> MouseCursor;
}

impl<'a, Color, Message, Renderer> From<Checkbox<Color, Message>>
    for Element<'a, Message, Renderer>
where
    Color: 'static + Copy + Default + std::fmt::Debug,
    Renderer: self::Renderer + text::Renderer<Color>,
    Message: 'static,
{
    fn from(
        checkbox: Checkbox<Color, Message>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(checkbox)
    }
}
