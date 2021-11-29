//! Allow your users to perform actions by pressing a button.
//!
//! A [`Button`] has some local [`State`].
use crate::event::{self, Event};
use crate::layout;
use crate::mouse;
use crate::overlay;
use crate::renderer;
use crate::touch;
use crate::{
    Background, Clipboard, Color, Element, Hasher, Layout, Length, Padding,
    Point, Rectangle, Shell, Vector, Widget,
};

use std::hash::Hash;

pub use iced_style::button::{Style, StyleSheet};

/// A generic widget that produces a message when pressed.
///
/// ```
/// # use iced_native::widget::{button, Text};
/// #
/// # type Button<'a, Message> =
/// #     iced_native::widget::Button<'a, Message, iced_native::renderer::Null>;
/// #
/// #[derive(Clone)]
/// enum Message {
///     ButtonPressed,
/// }
///
/// let mut state = button::State::new();
/// let button = Button::new(&mut state, Text::new("Press me!"))
///     .on_press(Message::ButtonPressed);
/// ```
///
/// If a [`Button::on_press`] handler is not set, the resulting [`Button`] will
/// be disabled:
///
/// ```
/// # use iced_native::widget::{button, Text};
/// #
/// # type Button<'a, Message> =
/// #     iced_native::widget::Button<'a, Message, iced_native::renderer::Null>;
/// #
/// #[derive(Clone)]
/// enum Message {
///     ButtonPressed,
/// }
///
/// fn disabled_button(state: &mut button::State) -> Button<'_, Message> {
///     Button::new(state, Text::new("I'm disabled!"))
/// }
///
/// fn enabled_button(state: &mut button::State) -> Button<'_, Message> {
///     disabled_button(state).on_press(Message::ButtonPressed)
/// }
/// ```
#[allow(missing_debug_implementations)]
pub struct Button<'a, Message, Renderer> {
    state: &'a mut State,
    content: Element<'a, Message, Renderer>,
    on_press: Option<Message>,
    width: Length,
    height: Length,
    min_width: u32,
    min_height: u32,
    padding: Padding,
    style_sheet: Box<dyn StyleSheet + 'a>,
}

impl<'a, Message, Renderer> Button<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: crate::Renderer,
{
    /// Creates a new [`Button`] with some local [`State`] and the given
    /// content.
    pub fn new<E>(state: &'a mut State, content: E) -> Self
    where
        E: Into<Element<'a, Message, Renderer>>,
    {
        Button {
            state,
            content: content.into(),
            on_press: None,
            width: Length::Shrink,
            height: Length::Shrink,
            min_width: 0,
            min_height: 0,
            padding: Padding::new(5),
            style_sheet: Default::default(),
        }
    }

    /// Sets the width of the [`Button`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Button`].
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the minimum width of the [`Button`].
    pub fn min_width(mut self, min_width: u32) -> Self {
        self.min_width = min_width;
        self
    }

    /// Sets the minimum height of the [`Button`].
    pub fn min_height(mut self, min_height: u32) -> Self {
        self.min_height = min_height;
        self
    }

    /// Sets the [`Padding`] of the [`Button`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed.
    /// If on_press isn't set, button will be disabled.
    pub fn on_press(mut self, msg: Message) -> Self {
        self.on_press = Some(msg);
        self
    }

    /// Sets the style of the [`Button`].
    pub fn style(
        mut self,
        style_sheet: impl Into<Box<dyn StyleSheet + 'a>>,
    ) -> Self {
        self.style_sheet = style_sheet.into();
        self
    }
}

/// The local state of a [`Button`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    is_pressed: bool,
}

impl State {
    /// Creates a new [`State`].
    pub fn new() -> State {
        State::default()
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Button<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: crate::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits
            .min_width(self.min_width)
            .min_height(self.min_height)
            .width(self.width)
            .height(self.height)
            .pad(self.padding);

        let mut content = self.content.layout(renderer, &limits);
        content.move_to(Point::new(
            self.padding.left.into(),
            self.padding.top.into(),
        ));

        let size = limits.resolve(content.size()).pad(self.padding);

        layout::Node::with_children(size, vec![content])
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        if let event::Status::Captured = self.content.on_event(
            event.clone(),
            layout.children().next().unwrap(),
            cursor_position,
            renderer,
            clipboard,
            shell,
        ) {
            return event::Status::Captured;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if self.on_press.is_some() {
                    let bounds = layout.bounds();

                    if bounds.contains(cursor_position) {
                        self.state.is_pressed = true;

                        return event::Status::Captured;
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if let Some(on_press) = self.on_press.clone() {
                    let bounds = layout.bounds();

                    if self.state.is_pressed {
                        self.state.is_pressed = false;

                        if bounds.contains(cursor_position) {
                            shell.publish(on_press);
                        }

                        return event::Status::Captured;
                    }
                }
            }
            Event::Touch(touch::Event::FingerLost { .. }) => {
                self.state.is_pressed = false;
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
        let is_mouse_over = layout.bounds().contains(cursor_position);
        let is_disabled = self.on_press.is_none();

        if is_mouse_over && !is_disabled {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();

        let is_mouse_over = bounds.contains(cursor_position);
        let is_disabled = self.on_press.is_none();

        let styling = if is_disabled {
            self.style_sheet.disabled()
        } else if is_mouse_over {
            if self.state.is_pressed {
                self.style_sheet.pressed()
            } else {
                self.style_sheet.hovered()
            }
        } else {
            self.style_sheet.active()
        };

        if styling.background.is_some() || styling.border_width > 0.0 {
            if styling.shadow_offset != Vector::default() {
                // TODO: Implement proper shadow support
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x + styling.shadow_offset.x,
                            y: bounds.y + styling.shadow_offset.y,
                            ..bounds
                        },
                        border_radius: styling.border_radius,
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                    Background::Color([0.0, 0.0, 0.0, 0.5].into()),
                );
            }

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border_radius: styling.border_radius,
                    border_width: styling.border_width,
                    border_color: styling.border_color,
                },
                styling
                    .background
                    .unwrap_or(Background::Color(Color::TRANSPARENT)),
            );
        }

        self.content.draw(
            renderer,
            &renderer::Style {
                text_color: styling.text_color,
            },
            content_layout,
            cursor_position,
            &bounds,
        );
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
        self.content.hash_layout(state);
    }

    fn overlay(
        &mut self,
        layout: Layout<'_>,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        self.content.overlay(layout.children().next().unwrap())
    }
}

impl<'a, Message, Renderer> From<Button<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + crate::Renderer,
{
    fn from(
        button: Button<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(button)
    }
}
