//! Create choices using radio buttons.
use crate::{Bus, Css, Element, Widget};

pub use iced_style::radio::{Style, StyleSheet};

use dodrio::bumpalo;

/// A circular button representing a choice.
///
/// # Example
/// ```
/// # use iced_web::Radio;
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
#[allow(missing_debug_implementations)]
pub struct Radio<'a, Message> {
    is_selected: bool,
    on_click: Message,
    label: String,
    id: Option<String>,
    name: Option<String>,
    #[allow(dead_code)]
    style_sheet: Box<dyn StyleSheet + 'a>,
}

impl<'a, Message> Radio<'a, Message> {
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
            id: None,
            name: None,
            style_sheet: Default::default(),
        }
    }

    /// Sets the style of the [`Radio`] button.
    pub fn style(
        mut self,
        style_sheet: impl Into<Box<dyn StyleSheet + 'a>>,
    ) -> Self {
        self.style_sheet = style_sheet.into();
        self
    }

    /// Sets the name attribute of the [`Radio`] button.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the id of the [`Radio`] button.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

impl<'a, Message> Widget<Message> for Radio<'a, Message>
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
        use dodrio::bumpalo::collections::String;

        let radio_label =
            String::from_str_in(&self.label, bump).into_bump_str();

        let event_bus = bus.clone();
        let on_click = self.on_click.clone();

        let (label, input) = if let Some(id) = &self.id {
            let id = String::from_str_in(id, bump).into_bump_str();

            (label(bump).attr("for", id), input(bump).attr("id", id))
        } else {
            (label(bump), input(bump))
        };

        let input = if let Some(name) = &self.name {
            let name = String::from_str_in(name, bump).into_bump_str();

            dodrio::builder::input(bump).attr("name", name)
        } else {
            input
        };

        // TODO: Complete styling
        label
            .attr("style", "display: block; font-size: 20px")
            .children(vec![
                input
                    .attr("type", "radio")
                    .attr("style", "margin-right: 10px")
                    .bool_attr("checked", self.is_selected)
                    .on("click", move |_root, _vdom, _event| {
                        event_bus.publish(on_click.clone());
                    })
                    .finish(),
                text(radio_label),
            ])
            .finish()
    }
}

impl<'a, Message> From<Radio<'a, Message>> for Element<'a, Message>
where
    Message: 'static + Clone,
{
    fn from(radio: Radio<'a, Message>) -> Element<'a, Message> {
        Element::new(radio)
    }
}
