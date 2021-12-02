//! Show toggle controls using togglers.
use std::hash::Hash;

use crate::alignment;
use crate::event;
use crate::layout;
use crate::mouse;
use crate::renderer;
use crate::text;
use crate::widget::{Row, Text};
use crate::{
    Alignment, Clipboard, Element, Event, Hasher, Layout, Length, Point,
    Rectangle, Shell, Widget,
};

pub use iced_style::toggler::{Style, StyleSheet};

/// A toggler widget
///
/// # Example
///
/// ```
/// # type Toggler<'a, Message> = iced_native::widget::Toggler<'a, Message, iced_native::renderer::Null>;
/// #
/// pub enum Message {
///     TogglerToggled(bool),
/// }
///
/// let is_active = true;
///
/// Toggler::new(is_active, String::from("Toggle me!"), |b| Message::TogglerToggled(b));
/// ```
#[allow(missing_debug_implementations)]
pub struct Toggler<'a, Message, Renderer: text::Renderer> {
    is_active: bool,
    on_toggle: Box<dyn Fn(bool) -> Message>,
    label: Option<String>,
    width: Length,
    size: u16,
    text_size: Option<u16>,
    text_alignment: alignment::Horizontal,
    spacing: u16,
    font: Renderer::Font,
    style_sheet: Box<dyn StyleSheet + 'a>,
}

impl<'a, Message, Renderer: text::Renderer> Toggler<'a, Message, Renderer> {
    /// The default size of a [`Toggler`].
    pub const DEFAULT_SIZE: u16 = 20;

    /// Creates a new [`Toggler`].
    ///
    /// It expects:
    ///   * a boolean describing whether the [`Toggler`] is checked or not
    ///   * An optional label for the [`Toggler`]
    ///   * a function that will be called when the [`Toggler`] is toggled. It
    ///     will receive the new state of the [`Toggler`] and must produce a
    ///     `Message`.
    pub fn new<F>(
        is_active: bool,
        label: impl Into<Option<String>>,
        f: F,
    ) -> Self
    where
        F: 'static + Fn(bool) -> Message,
    {
        Toggler {
            is_active,
            on_toggle: Box::new(f),
            label: label.into(),
            width: Length::Fill,
            size: Self::DEFAULT_SIZE,
            text_size: None,
            text_alignment: alignment::Horizontal::Left,
            spacing: 0,
            font: Renderer::Font::default(),
            style_sheet: Default::default(),
        }
    }

    /// Sets the size of the [`Toggler`].
    pub fn size(mut self, size: u16) -> Self {
        self.size = size;
        self
    }

    /// Sets the width of the [`Toggler`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the text size o the [`Toggler`].
    pub fn text_size(mut self, text_size: u16) -> Self {
        self.text_size = Some(text_size);
        self
    }

    /// Sets the horizontal alignment of the text of the [`Toggler`]
    pub fn text_alignment(mut self, alignment: alignment::Horizontal) -> Self {
        self.text_alignment = alignment;
        self
    }

    /// Sets the spacing between the [`Toggler`] and the text.
    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the [`Font`] of the text of the [`Toggler`]
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = font;
        self
    }

    /// Sets the style of the [`Toggler`].
    pub fn style(
        mut self,
        style_sheet: impl Into<Box<dyn StyleSheet + 'a>>,
    ) -> Self {
        self.style_sheet = style_sheet.into();
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Toggler<'a, Message, Renderer>
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
        let mut row = Row::<(), Renderer>::new()
            .width(self.width)
            .spacing(self.spacing)
            .align_items(Alignment::Center);

        if let Some(label) = &self.label {
            row = row.push(
                Text::new(label)
                    .horizontal_alignment(self.text_alignment)
                    .font(self.font)
                    .width(self.width)
                    .size(self.text_size.unwrap_or(renderer.default_size())),
            );
        }

        row = row.push(
            Row::new()
                .width(Length::Units(2 * self.size))
                .height(Length::Units(self.size)),
        );

        row.layout(renderer, limits)
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
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let mouse_over = layout.bounds().contains(cursor_position);

                if mouse_over {
                    shell.publish((self.on_toggle)(!self.is_active));

                    event::Status::Captured
                } else {
                    event::Status::Ignored
                }
            }
            _ => event::Status::Ignored,
        }
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
        /// Makes sure that the border radius of the toggler looks good at every size.
        const BORDER_RADIUS_RATIO: f32 = 32.0 / 13.0;

        /// The space ratio between the background Quad and the Toggler bounds, and
        /// between the background Quad and foreground Quad.
        const SPACE_RATIO: f32 = 0.05;

        let mut children = layout.children();

        if let Some(label) = &self.label {
            let label_layout = children.next().unwrap();

            crate::widget::text::draw(
                renderer,
                style,
                label_layout,
                &label,
                self.font,
                self.text_size,
                None,
                self.text_alignment,
                alignment::Vertical::Center,
            );
        }

        let toggler_layout = children.next().unwrap();
        let bounds = toggler_layout.bounds();

        let is_mouse_over = bounds.contains(cursor_position);

        let style = if is_mouse_over {
            self.style_sheet.hovered(self.is_active)
        } else {
            self.style_sheet.active(self.is_active)
        };

        let border_radius = bounds.height as f32 / BORDER_RADIUS_RATIO;
        let space = SPACE_RATIO * bounds.height as f32;

        let toggler_background_bounds = Rectangle {
            x: bounds.x + space,
            y: bounds.y + space,
            width: bounds.width - (2.0 * space),
            height: bounds.height - (2.0 * space),
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: toggler_background_bounds,
                border_radius,
                border_width: 1.0,
                border_color: style
                    .background_border
                    .unwrap_or(style.background),
            },
            style.background,
        );

        let toggler_foreground_bounds = Rectangle {
            x: bounds.x
                + if self.is_active {
                    bounds.width - 2.0 * space - (bounds.height - (4.0 * space))
                } else {
                    2.0 * space
                },
            y: bounds.y + (2.0 * space),
            width: bounds.height - (4.0 * space),
            height: bounds.height - (4.0 * space),
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: toggler_foreground_bounds,
                border_radius,
                border_width: 1.0,
                border_color: style
                    .foreground_border
                    .unwrap_or(style.foreground),
            },
            style.foreground,
        );
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.label.hash(state)
    }
}

impl<'a, Message, Renderer> From<Toggler<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + text::Renderer,
    Message: 'a,
{
    fn from(
        toggler: Toggler<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(toggler)
    }
}
