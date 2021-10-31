//! Allow your users to perform actions by pressing a button.
//!
//! A [`Button`] has some local [`State`].
use crate::{css, Background, Bus, Css, Element, Length, Padding, Widget};

pub use iced_style::button::{Style, StyleSheet};

use dodrio::bumpalo;

/// A generic widget that produces a message when pressed.
///
/// ```
/// # use iced_web::{button, Button, Text};
/// #
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
/// # use iced_web::{button, Button, Text};
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
pub struct Button<'a, Message> {
    content: Element<'a, Message>,
    on_press: Option<Message>,
    width: Length,
    #[allow(dead_code)]
    height: Length,
    min_width: u32,
    #[allow(dead_code)]
    min_height: u32,
    padding: Padding,
    style: Box<dyn StyleSheet + 'a>,
}

impl<'a, Message> Button<'a, Message> {
    /// Creates a new [`Button`] with some local [`State`] and the given
    /// content.
    pub fn new<E>(_state: &'a mut State, content: E) -> Self
    where
        E: Into<Element<'a, Message>>,
    {
        Button {
            content: content.into(),
            on_press: None,
            width: Length::Shrink,
            height: Length::Shrink,
            min_width: 0,
            min_height: 0,
            padding: Padding::new(5),
            style: Default::default(),
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

    /// Sets the style of the [`Button`].
    pub fn style(mut self, style: impl Into<Box<dyn StyleSheet + 'a>>) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed.
    /// If on_press isn't set, button will be disabled.
    pub fn on_press(mut self, msg: Message) -> Self {
        self.on_press = Some(msg);
        self
    }
}

/// The local state of a [`Button`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State;

impl State {
    /// Creates a new [`State`].
    pub fn new() -> State {
        State::default()
    }
}

impl<'a, Message> Widget<Message> for Button<'a, Message>
where
    Message: 'static + Clone,
{
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
        style_sheet: &mut Css<'b>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::*;

        // TODO: State-based styling
        let style = self.style.active();

        let background = match style.background {
            None => String::from("none"),
            Some(background) => match background {
                Background::Color(color) => css::color(color),
            },
        };

        let mut node = button(bump)
            .attr(
                "style",
                bumpalo::format!(
                    in bump,
                    "background: {}; border-radius: {}px; width:{}; \
                    min-width: {}; color: {}; padding: {}",
                    background,
                    style.border_radius,
                    css::length(self.width),
                    css::min_length(self.min_width),
                    css::color(style.text_color),
                    css::padding(self.padding)
                )
                .into_bump_str(),
            )
            .children(vec![self.content.node(bump, bus, style_sheet)]);

        if let Some(on_press) = self.on_press.clone() {
            let event_bus = bus.clone();

            node = node.on("click", move |_root, _vdom, _event| {
                event_bus.publish(on_press.clone());
            });
        } else {
            node = node.attr("disabled", "");
        }

        node.finish()
    }
}

impl<'a, Message> From<Button<'a, Message>> for Element<'a, Message>
where
    Message: 'static + Clone,
{
    fn from(button: Button<'a, Message>) -> Element<'a, Message> {
        Element::new(button)
    }
}
