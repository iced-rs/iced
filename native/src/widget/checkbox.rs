//! Show toggle controls using checkboxes.
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

pub use iced_style::checkbox::{Style, StyleSheet};

/// A box that can be checked.
///
/// # Example
///
/// ```
/// # type Checkbox<'a, Message> = iced_native::widget::Checkbox<'a, Message, iced_native::renderer::Null>;
/// #
/// pub enum Message {
///     CheckboxToggled(bool),
/// }
///
/// let is_checked = true;
///
/// Checkbox::new(is_checked, "Toggle me!", Message::CheckboxToggled);
/// ```
///
/// ![Checkbox drawn by `iced_wgpu`](https://github.com/hecrj/iced/blob/7760618fb112074bc40b148944521f312152012a/docs/images/checkbox.png?raw=true)
#[allow(missing_debug_implementations)]
pub struct Checkbox<'a, Message, Renderer: text::Renderer> {
    is_checked: bool,
    on_toggle: Box<dyn Fn(bool) -> Message>,
    label: String,
    width: Length,
    size: u16,
    spacing: u16,
    text_size: Option<u16>,
    font: Renderer::Font,
    text_color: Option<Color>,
    style_sheet: Box<dyn StyleSheet + 'a>,
}

impl<'a, Message, Renderer: text::Renderer> Checkbox<'a, Message, Renderer> {
    /// The default size of a [`Checkbox`].
    const DEFAULT_SIZE: u16 = 20;

    /// The default spacing of a [`Checkbox`].
    const DEFAULT_SPACING: u16 = 15;

    /// Creates a new [`Checkbox`].
    ///
    /// It expects:
    ///   * a boolean describing whether the [`Checkbox`] is checked or not
    ///   * the label of the [`Checkbox`]
    ///   * a function that will be called when the [`Checkbox`] is toggled. It
    ///     will receive the new state of the [`Checkbox`] and must produce a
    ///     `Message`.
    pub fn new<F>(is_checked: bool, label: impl Into<String>, f: F) -> Self
    where
        F: 'static + Fn(bool) -> Message,
    {
        Checkbox {
            is_checked,
            on_toggle: Box::new(f),
            label: label.into(),
            width: Length::Shrink,
            size: Self::DEFAULT_SIZE,
            spacing: Self::DEFAULT_SPACING,
            text_size: None,
            font: Renderer::Font::default(),
            text_color: None,
            style_sheet: Default::default(),
        }
    }

    /// Sets the size of the [`Checkbox`].
    pub fn size(mut self, size: u16) -> Self {
        self.size = size;
        self
    }

    /// Sets the width of the [`Checkbox`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the spacing between the [`Checkbox`] and the text.
    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the text size of the [`Checkbox`].
    pub fn text_size(mut self, text_size: u16) -> Self {
        self.text_size = Some(text_size);
        self
    }

    /// Sets the [`Font`] of the text of the [`Checkbox`].
    ///
    /// [`Font`]: crate::widget::text::Renderer::Font
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = font;
        self
    }

    /// Sets the text color of the [`Checkbox`] button.
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = Some(color);
        self
    }

    /// Sets the style of the [`Checkbox`].
    pub fn style(
        mut self,
        style_sheet: impl Into<Box<dyn StyleSheet + 'a>>,
    ) -> Self {
        self.style_sheet = style_sheet.into();
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Checkbox<'a, Message, Renderer>
where
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
                    .font(self.font)
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
                let mouse_over = layout.bounds().contains(cursor_position);

                if mouse_over {
                    shell.publish((self.on_toggle)(!self.is_checked));

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

        let custom_style = if is_mouse_over {
            self.style_sheet.hovered(self.is_checked)
        } else {
            self.style_sheet.active(self.is_checked)
        };

        {
            let layout = children.next().unwrap();
            let bounds = layout.bounds();

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border_radius: custom_style.border_radius,
                    border_width: custom_style.border_width,
                    border_color: custom_style.border_color,
                },
                custom_style.background,
            );

            if self.is_checked {
                renderer.fill_text(text::Text {
                    content: &Renderer::CHECKMARK_ICON.to_string(),
                    font: Renderer::ICON_FONT,
                    size: bounds.height * 0.7,
                    bounds: Rectangle {
                        x: bounds.center_x(),
                        y: bounds.center_y(),
                        ..bounds
                    },
                    color: custom_style.checkmark_color,
                    horizontal_alignment: alignment::Horizontal::Center,
                    vertical_alignment: alignment::Vertical::Center,
                });
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
                self.text_color.or(Some(custom_style.text_color)),
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

impl<'a, Message, Renderer> From<Checkbox<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + text::Renderer,
    Message: 'a,
{
    fn from(
        checkbox: Checkbox<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(checkbox)
    }
}
