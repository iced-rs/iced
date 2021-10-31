//! Display fields that can be filled with text.
//!
//! A [`TextInput`] has some local [`State`].
use crate::{bumpalo, css, Bus, Css, Element, Length, Padding, Widget};

pub use iced_style::text_input::{Style, StyleSheet};

use std::{rc::Rc, u32};

/// A field that can be filled with text.
///
/// # Example
/// ```
/// # use iced_web::{text_input, TextInput};
/// #
/// enum Message {
///     TextInputChanged(String),
/// }
///
/// let mut state = text_input::State::new();
/// let value = "Some text";
///
/// let input = TextInput::new(
///     &mut state,
///     "This is the placeholder...",
///     value,
///     Message::TextInputChanged,
/// );
/// ```
#[allow(missing_debug_implementations)]
pub struct TextInput<'a, Message> {
    _state: &'a mut State,
    placeholder: String,
    value: String,
    is_secure: bool,
    width: Length,
    max_width: u32,
    padding: Padding,
    size: Option<u16>,
    on_change: Rc<Box<dyn Fn(String) -> Message>>,
    on_submit: Option<Message>,
    style_sheet: Box<dyn StyleSheet + 'a>,
}

impl<'a, Message> TextInput<'a, Message> {
    /// Creates a new [`TextInput`].
    ///
    /// It expects:
    /// - some [`State`]
    /// - a placeholder
    /// - the current value
    /// - a function that produces a message when the [`TextInput`] changes
    pub fn new<F>(
        state: &'a mut State,
        placeholder: &str,
        value: &str,
        on_change: F,
    ) -> Self
    where
        F: 'static + Fn(String) -> Message,
    {
        Self {
            _state: state,
            placeholder: String::from(placeholder),
            value: String::from(value),
            is_secure: false,
            width: Length::Fill,
            max_width: u32::MAX,
            padding: Padding::ZERO,
            size: None,
            on_change: Rc::new(Box::new(on_change)),
            on_submit: None,
            style_sheet: Default::default(),
        }
    }

    /// Converts the [`TextInput`] into a secure password input.
    pub fn password(mut self) -> Self {
        self.is_secure = true;
        self
    }

    /// Sets the width of the [`TextInput`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the maximum width of the [`TextInput`].
    pub fn max_width(mut self, max_width: u32) -> Self {
        self.max_width = max_width;
        self
    }

    /// Sets the [`Padding`] of the [`TextInput`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the text size of the [`TextInput`].
    pub fn size(mut self, size: u16) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets the message that should be produced when the [`TextInput`] is
    /// focused and the enter key is pressed.
    pub fn on_submit(mut self, message: Message) -> Self {
        self.on_submit = Some(message);
        self
    }

    /// Sets the style of the [`TextInput`].
    pub fn style(
        mut self,
        style_sheet: impl Into<Box<dyn StyleSheet + 'a>>,
    ) -> Self {
        self.style_sheet = style_sheet.into();
        self
    }
}

impl<'a, Message> Widget<Message> for TextInput<'a, Message>
where
    Message: 'static + Clone,
{
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
        _style_sheet: &mut Css<'b>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::*;
        use wasm_bindgen::JsCast;

        let placeholder = {
            use dodrio::bumpalo::collections::String;

            String::from_str_in(&self.placeholder, bump).into_bump_str()
        };

        let value = {
            use dodrio::bumpalo::collections::String;

            String::from_str_in(&self.value, bump).into_bump_str()
        };

        let on_change = self.on_change.clone();
        let on_submit = self.on_submit.clone();
        let input_event_bus = bus.clone();
        let submit_event_bus = bus.clone();
        let style = self.style_sheet.active();

        input(bump)
            .attr(
                "style",
                bumpalo::format!(
                    in bump,
                    "width: {}; max-width: {}; padding: {}; font-size: {}px; \
                    background: {}; border-width: {}px; border-color: {}; \
                    border-radius: {}px; color: {}",
                    css::length(self.width),
                    css::max_length(self.max_width),
                    css::padding(self.padding),
                    self.size.unwrap_or(20),
                    css::background(style.background),
                    style.border_width,
                    css::color(style.border_color),
                    style.border_radius,
                    css::color(self.style_sheet.value_color())
                )
                .into_bump_str(),
            )
            .attr("placeholder", placeholder)
            .attr("value", value)
            .attr("type", if self.is_secure { "password" } else { "text" })
            .on("input", move |_root, _vdom, event| {
                let text_input = match event.target().and_then(|t| {
                    t.dyn_into::<web_sys::HtmlInputElement>().ok()
                }) {
                    None => return,
                    Some(text_input) => text_input,
                };

                input_event_bus.publish(on_change(text_input.value()));
            })
            .on("keypress", move |_root, _vdom, event| {
                if let Some(on_submit) = on_submit.clone() {
                    let event =
                        event.unchecked_into::<web_sys::KeyboardEvent>();

                    match event.key_code() {
                        13 => {
                            submit_event_bus.publish(on_submit);
                        }
                        _ => {}
                    }
                }
            })
            .finish()
    }
}

impl<'a, Message> From<TextInput<'a, Message>> for Element<'a, Message>
where
    Message: 'static + Clone,
{
    fn from(text_input: TextInput<'a, Message>) -> Element<'a, Message> {
        Element::new(text_input)
    }
}

/// The state of a [`TextInput`].
#[derive(Debug, Clone, Copy, Default)]
pub struct State;

impl State {
    /// Creates a new [`State`], representing an unfocused [`TextInput`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`State`], representing a focused [`TextInput`].
    pub fn focused() -> Self {
        // TODO
        Self::default()
    }

    /// Selects all the content of the [`TextInput`].
    pub fn select_all(&mut self) {
        // TODO
    }
}
