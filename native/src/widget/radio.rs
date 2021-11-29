//! Create choices using radio buttons.
use std::hash::Hash;

use crate::alignment;
use crate::event::{self, Event};
use crate::layout;
use crate::mouse;
use crate::renderer;
use crate::text;
use crate::touch;
use crate::widget::{self, Row, Text};
use crate::{
    Alignment, Clipboard, Color, Element, Hasher, Layout, Length, Point,
    Rectangle, Shell, Widget,
};

pub use iced_style::radio::{Style, StyleSheet};

/// A circular button representing a choice.
///
/// # Example
/// ```
/// # type Radio<'a, Message> =
/// #     iced_native::widget::Radio<'a, Message, iced_native::renderer::Null>;
/// #
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
/// ![Radio buttons drawn by `iced_wgpu`](https://github.com/hecrj/iced/blob/7760618fb112074bc40b148944521f312152012a/docs/images/radio.png?raw=true)
#[allow(missing_debug_implementations)]
pub struct Radio<'a, Message, Renderer: text::Renderer> {
    is_selected: bool,
    on_click: Message,
    label: String,
    width: Length,
    size: u16,
    spacing: u16,
    text_size: Option<u16>,
    text_color: Option<Color>,
    font: Renderer::Font,
    style_sheet: Box<dyn StyleSheet + 'a>,
}

impl<'a, Message, Renderer: text::Renderer> Radio<'a, Message, Renderer>
where
    Message: Clone,
{
    /// The default size of a [`Radio`] button.
    pub const DEFAULT_SIZE: u16 = 28;

    /// The default spacing of a [`Radio`] button.
    pub const DEFAULT_SPACING: u16 = 15;

    /// Creates a new [`Radio`] button.
    ///
    /// It expects:
    ///   * the value related to the [`Radio`] button
    ///   * the label of the [`Radio`] button
    ///   * the current selected value
    ///   * a function that will be called when the [`Radio`] is selected. It
    ///   receives the value of the radio and must produce a `Message`.
    pub fn new<F, V>(
        value: V,
        label: impl Into<String>,
        selected: Option<V>,
        f: F,
    ) -> Self
    where
        V: Eq + Copy,
        F: 'static + Fn(V) -> Message,
    {
        Radio {
            is_selected: Some(value) == selected,
            on_click: f(value),
            label: label.into(),
            width: Length::Shrink,
            size: Self::DEFAULT_SIZE,
            spacing: Self::DEFAULT_SPACING, //15
            text_size: None,
            text_color: None,
            font: Default::default(),
            style_sheet: Default::default(),
        }
    }

    /// Sets the size of the [`Radio`] button.
    pub fn size(mut self, size: u16) -> Self {
        self.size = size;
        self
    }

    /// Sets the width of the [`Radio`] button.
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the spacing between the [`Radio`] button and the text.
    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the text size of the [`Radio`] button.
    pub fn text_size(mut self, text_size: u16) -> Self {
        self.text_size = Some(text_size);
        self
    }

    /// Sets the text color of the [`Radio`] button.
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = Some(color);
        self
    }

    /// Sets the text font of the [`Radio`] button.
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = font;
        self
    }

    /// Sets the style of the [`Radio`] button.
    pub fn style(
        mut self,
        style_sheet: impl Into<Box<dyn StyleSheet + 'a>>,
    ) -> Self {
        self.style_sheet = style_sheet.into();
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Radio<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: text::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        Row::<(), Renderer>::new()
            .width(self.width)
            .spacing(self.spacing)
            .align_items(Alignment::Center)
            .push(
                Row::new()
                    .width(Length::Units(self.size))
                    .height(Length::Units(self.size)),
            )
            .push(
                Text::new(&self.label)
                    .width(self.width)
                    .size(self.text_size.unwrap_or(renderer.default_size())),
            )
            .layout(renderer, limits)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if layout.bounds().contains(cursor_position) {
                    shell.publish(self.on_click.clone());

                    return event::Status::Captured;
                }
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) -> mouse::Interaction {
        if layout.bounds().contains(cursor_position) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let is_mouse_over = bounds.contains(cursor_position);

        let mut children = layout.children();

        {
            let layout = children.next().unwrap();
            let bounds = layout.bounds();

            let size = bounds.width;
            let dot_size = size / 2.0;

            let style = if is_mouse_over {
                self.style_sheet.hovered()
            } else {
                self.style_sheet.active()
            };

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border_radius: size / 2.0,
                    border_width: style.border_width,
                    border_color: style.border_color,
                },
                style.background,
            );

            if self.is_selected {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x + dot_size / 2.0,
                            y: bounds.y + dot_size / 2.0,
                            width: bounds.width - dot_size,
                            height: bounds.height - dot_size,
                        },
                        border_radius: dot_size / 2.0,
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                    style.dot_color,
                );
            }
        }

        {
            let label_layout = children.next().unwrap();

            widget::text::draw(
                renderer,
                style,
                label_layout,
                &self.label,
                self.font,
                self.text_size,
                self.text_color,
                alignment::Horizontal::Left,
                alignment::Vertical::Center,
            );
        }
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.label.hash(state);
    }
}

impl<'a, Message, Renderer> From<Radio<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + text::Renderer,
{
    fn from(
        radio: Radio<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(radio)
    }
}
