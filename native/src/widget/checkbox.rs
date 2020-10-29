//! Show toggle controls using checkboxes.
use std::hash::Hash;

use crate::{
    layout, mouse, row, text, Align, Clipboard, Element, Event, Hasher,
    HorizontalAlignment, Layout, Length, Point, Rectangle, Row, Text,
    VerticalAlignment, Widget,
};

/// A box that can be checked.
///
/// # Example
///
/// ```
/// # type Checkbox<Message> = iced_native::Checkbox<Message, iced_native::renderer::Null>;
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
pub struct Checkbox<Message, Renderer: self::Renderer + text::Renderer> {
    is_checked: bool,
    on_toggle: Box<dyn Fn(bool) -> Message>,
    label: String,
    width: Length,
    size: u16,
    spacing: u16,
    text_size: Option<u16>,
    font: Renderer::Font,
    style: Renderer::Style,
}

impl<Message, Renderer: self::Renderer + text::Renderer>
    Checkbox<Message, Renderer>
{
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
    pub fn new<F>(is_checked: bool, label: impl Into<String>, f: F) -> Self
    where
        F: 'static + Fn(bool) -> Message,
    {
        Checkbox {
            is_checked,
            on_toggle: Box::new(f),
            label: label.into(),
            width: Length::Shrink,
            size: <Renderer as self::Renderer>::DEFAULT_SIZE,
            spacing: Renderer::DEFAULT_SPACING,
            text_size: None,
            font: Renderer::Font::default(),
            style: Renderer::Style::default(),
        }
    }

    /// Sets the size of the [`Checkbox`].
    ///
    /// [`Checkbox`]: struct.Checkbox.html
    pub fn size(mut self, size: u16) -> Self {
        self.size = size;
        self
    }

    /// Sets the width of the [`Checkbox`].
    ///
    /// [`Checkbox`]: struct.Checkbox.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the spacing between the [`Checkbox`] and the text.
    ///
    /// [`Checkbox`]: struct.Checkbox.html
    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the text size of the [`Checkbox`].
    ///
    /// [`Checkbox`]: struct.Checkbox.html
    pub fn text_size(mut self, text_size: u16) -> Self {
        self.text_size = Some(text_size);
        self
    }

    /// Sets the [`Font`] of the text of the [`Checkbox`].
    ///
    /// [`Checkbox`]: struct.Checkbox.html
    /// [`Font`]: ../../struct.Font.html
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = font;
        self
    }

    /// Sets the style of the [`Checkbox`].
    ///
    /// [`Checkbox`]: struct.Checkbox.html
    pub fn style(mut self, style: impl Into<Renderer::Style>) -> Self {
        self.style = style.into();
        self
    }
}

impl<Message, Renderer> Widget<Message, Renderer>
    for Checkbox<Message, Renderer>
where
    Renderer: self::Renderer + text::Renderer + row::Renderer,
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
            .align_items(Align::Center)
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
        messages: &mut Vec<Message>,
        _renderer: &Renderer,
        _clipboard: Option<&dyn Clipboard>,
    ) {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
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
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) -> Renderer::Output {
        let bounds = layout.bounds();
        let mut children = layout.children();

        let checkbox_layout = children.next().unwrap();
        let label_layout = children.next().unwrap();
        let checkbox_bounds = checkbox_layout.bounds();

        let label = text::Renderer::draw(
            renderer,
            defaults,
            label_layout.bounds(),
            &self.label,
            self.text_size.unwrap_or(renderer.default_size()),
            self.font,
            None,
            HorizontalAlignment::Left,
            VerticalAlignment::Center,
        );

        let is_mouse_over = bounds.contains(cursor_position);

        self::Renderer::draw(
            renderer,
            checkbox_bounds,
            self.is_checked,
            is_mouse_over,
            label,
            &self.style,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

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
    /// The style supported by this renderer.
    type Style: Default;

    /// The default size of a [`Checkbox`].
    ///
    /// [`Checkbox`]: struct.Checkbox.html
    const DEFAULT_SIZE: u16;

    /// The default spacing of a [`Checkbox`].
    ///
    /// [`Checkbox`]: struct.Checkbox.html
    const DEFAULT_SPACING: u16;

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
        label: Self::Output,
        style: &Self::Style,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<Checkbox<Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer + text::Renderer + row::Renderer,
    Message: 'a,
{
    fn from(
        checkbox: Checkbox<Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(checkbox)
    }
}
