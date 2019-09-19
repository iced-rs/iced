//! Create choices using radio buttons.
use crate::input::{mouse, ButtonState};
use crate::widget::{text, Column, Row, Text};
use crate::{
    Align, Color, Element, Event, Hasher, Layout, MouseCursor, Node, Point,
    Rectangle, Widget,
};

use std::hash::Hash;

/// A circular button representing a choice.
///
/// It implements [`Widget`] when the associated `Renderer` implements the
/// [`radio::Renderer`] trait.
///
/// [`Widget`]: ../trait.Widget.html
/// [`radio::Renderer`]: trait.Renderer.html
///
/// # Example
/// ```
/// use iced::Radio;
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// pub enum Choice {
///     A,
///     B,
/// }
///
/// #[derive(Debug, Clone, Copy)]
/// pub enum Message {
///     RadioSelected(Choice),
/// }
///
/// let selected_choice = Some(Choice::A);
///
/// Radio::new(Choice::A, "This is A", selected_choice, Message::RadioSelected);
///
/// Radio::new(Choice::B, "This is B", selected_choice, Message::RadioSelected);
/// ```
///
/// ![Radio buttons drawn by Coffee's renderer](https://github.com/hecrj/coffee/blob/bda9818f823dfcb8a7ad0ff4940b4d4b387b5208/images/ui/radio.png?raw=true)
pub struct Radio<Message> {
    /// Whether the radio button is selected or not
    pub is_selected: bool,

    /// The message to produce when the radio button is clicked
    pub on_click: Message,

    /// The label of the radio button
    pub label: String,

    /// The color of the label
    pub label_color: Option<Color>,
}

impl<Message> std::fmt::Debug for Radio<Message>
where
    Message: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Radio")
            .field("is_selected", &self.is_selected)
            .field("on_click", &self.on_click)
            .field("label", &self.label)
            .field("label_color", &self.label_color)
            .finish()
    }
}

impl<Message> Radio<Message> {
    /// Creates a new [`Radio`] button.
    ///
    /// It expects:
    ///   * the value related to the [`Radio`] button
    ///   * the label of the [`Radio`] button
    ///   * the current selected value
    ///   * a function that will be called when the [`Radio`] is selected. It
    ///   receives the value of the radio and must produce a `Message`.
    ///
    /// [`Radio`]: struct.Radio.html
    pub fn new<F, V>(value: V, label: &str, selected: Option<V>, f: F) -> Self
    where
        V: Eq + Copy,
        F: 'static + Fn(V) -> Message,
    {
        Radio {
            is_selected: Some(value) == selected,
            on_click: f(value),
            label: String::from(label),
            label_color: None,
        }
    }

    /// Sets the `Color` of the label of the [`Radio`].
    ///
    /// [`Radio`]: struct.Radio.html
    pub fn label_color<C: Into<Color>>(mut self, color: C) -> Self {
        self.label_color = Some(color.into());
        self
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for Radio<Message>
where
    Renderer: self::Renderer + text::Renderer,
    Message: Copy + std::fmt::Debug,
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
    ) -> MouseCursor {
        let children: Vec<_> = layout.children().collect();

        let mut text_bounds = children[1].bounds();
        text_bounds.y -= 2.0;

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
            layout.bounds(),
            self.is_selected,
        )
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
pub trait Renderer {
    /// Draws a [`Radio`] button.
    ///
    /// It receives:
    ///   * the current cursor position
    ///   * the bounds of the [`Radio`]
    ///   * the bounds of the label of the [`Radio`]
    ///   * whether the [`Radio`] is selected or not
    ///
    /// [`Radio`]: struct.Radio.html
    fn draw(
        &mut self,
        cursor_position: Point,
        bounds: Rectangle,
        label_bounds: Rectangle,
        is_selected: bool,
    ) -> MouseCursor;
}

impl<'a, Message, Renderer> From<Radio<Message>>
    for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer + text::Renderer,
    Message: 'static + Copy + std::fmt::Debug,
{
    fn from(checkbox: Radio<Message>) -> Element<'a, Message, Renderer> {
        Element::new(checkbox)
    }
}
