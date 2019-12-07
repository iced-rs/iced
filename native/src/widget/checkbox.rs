//! Show toggle controls using checkboxes.
use std::hash::Hash;

use crate::{
    input::{mouse, ButtonState},
    layout, row, text, Align, Element, Event, Font, Hasher,
    HorizontalAlignment, Layout, Length, Point, Rectangle, Row, Text,
    VerticalAlignment, Widget,
};

/// A box that can be checked.
///
/// # Example
///
/// ```
/// # use iced_native::Checkbox;
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
pub struct Checkbox<Message, Style> {
    is_checked: bool,
    on_toggle: Box<dyn Fn(bool) -> Message>,
    label: String,
    size: u16,
    font: Font,
    style: Style,
}

impl<Message, Style> Checkbox<Message, Style> {
    /// Creates a new [`Checkbox`].
    ///
    /// It expects:
    ///   * a boolean describing whether the [`Checkbox`] is checked or not
    ///   * the label of the [`Checkbox`]
    ///   * a function that will be called when the [`Checkbox`] is toggled. It
    ///     will receive the new state of the [`Checkbox`] and must produce a
    ///     `Message`.
    ///
    /// [`Checkbox`]: struct.Checkbox.html
    pub fn new<F>(is_checked: bool, label: &str, f: F) -> Self
    where
        F: 'static + Fn(bool) -> Message,
        Style: Default,
    {
        Checkbox {
            is_checked,
            on_toggle: Box::new(f),
            style: Style::default(),
            label: String::from(label),
            size: 20,
            font: Font::Default,
        }
    }

    /// Creates a new [`Checkbox`] with a custom `style`.
    ///
    /// It expects:
    ///   * a boolean describing whether the [`Checkbox`] is checked or not
    ///   * the label of the [`Checkbox`]
    ///   * a function that will be called when the [`Checkbox`] is toggled. It
    ///     will receive the new state of the [`Checkbox`] and must produce a
    ///     `Message`.
    ///
    /// [`Checkbox`]: struct.Checkbox.html
    pub fn new_with_style<F>(
        is_checked: bool,
        label: &str,
        f: F,
        style: Style,
    ) -> Self
    where
        F: 'static + Fn(bool) -> Message,
    {
        Checkbox {
            is_checked,
            on_toggle: Box::new(f),
            style,
            label: String::from(label),
            size: 20,
            font: Font::Default,
        }
    }

    /// Changes the style of the [`Checkbox`].
    ///
    /// [`Checkbox`]: struct.Checkbox.html
    pub fn change_style(mut self, f: impl FnOnce(&mut Style)) -> Self {
        f(&mut self.style);
        self
    }
}

impl<Message, Renderer, Style, LabelStyle> Widget<Message, Renderer>
    for Checkbox<Message, Style>
where
    Renderer: self::Renderer<WidgetStyle = Style>
        + text::Renderer<WidgetStyle = LabelStyle>
        + row::Renderer,
    LabelStyle: Clone + 'static,
    Style: AsRef<LabelStyle>,
{
    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = self::Renderer::default_size(renderer);

        Row::<(), Renderer>::new()
            .spacing(15)
            .align_items(Align::Center)
            .push(
                Row::new()
                    .width(Length::Units(size as u16))
                    .height(Length::Units(size as u16)),
            )
            .push(Text::new_with_style(
                &self.label,
                self.style.as_ref().clone(),
            ))
            .layout(renderer, limits)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
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
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        let bounds = layout.bounds();
        let mut children = layout.children();

        let checkbox_layout = children.next().unwrap();
        let label_layout = children.next().unwrap();
        let checkbox_bounds = checkbox_layout.bounds();

        let label = text::Renderer::draw(
            renderer,
            label_layout.bounds(),
            &self.label,
            self.size,
            self.font,
            self.style.as_ref(),
            HorizontalAlignment::Left,
            VerticalAlignment::Center,
        );

        let is_mouse_over = bounds.contains(cursor_position);

        self::Renderer::draw(
            renderer,
            checkbox_bounds,
            self.is_checked,
            is_mouse_over,
            &self.style,
            label,
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
pub trait Renderer: crate::Renderer {
    /// Struct that consists of all style options the renderer supports for
    /// [`Checkbox`].
    ///
    /// [`Checkbox`]: struct.Checkbox.html
    type WidgetStyle;

    /// Returns the default size of a [`Checkbox`].
    ///
    /// [`Checkbox`]: struct.Checkbox.html
    fn default_size(&self) -> u32;

    /// Draws a [`Checkbox`].
    ///
    /// It receives:
    ///   * the bounds of the [`Checkbox`]
    ///   * whether the [`Checkbox`] is selected or not
    ///   * whether the mouse is over the [`Checkbox`] or not
    ///   * the drawn label of the [`Checkbox`]
    ///
    /// [`Checkbox`]: struct.Checkbox.html
    fn draw(
        &mut self,
        bounds: Rectangle,
        is_checked: bool,
        is_mouse_over: bool,
        style: &Self::WidgetStyle,
        label: Self::Output,
    ) -> Self::Output;
}

impl<'a, Message, Renderer, Style, LabelStyle> From<Checkbox<Message, Style>>
    for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer<WidgetStyle = Style>
        + text::Renderer<WidgetStyle = LabelStyle>
        + row::Renderer,
    Message: 'static,
    Style: AsRef<LabelStyle> + 'static,
    LabelStyle: Clone + 'static,
{
    fn from(checkbox: Checkbox<Message, Style>) -> Element<'a, Message, Renderer> {
        Element::new(checkbox)
    }
}
