//! Show toggle controls using checkboxes.
use crate::{css, Bus, Css, Element, Length, Widget};

pub use iced_style::checkbox::{Style, StyleSheet};

use dodrio::bumpalo;
use std::rc::Rc;

/// A box that can be checked.
///
/// # Example
///
/// ```
/// # use iced_web::Checkbox;
///
/// pub enum Message {
///     CheckboxToggled(bool),
/// }
///
/// let is_checked = true;
///
/// Checkbox::new(is_checked, "Toggle me!", Message::CheckboxToggled);
/// ```
///
/// ![Checkbox drawn by Coffee's renderer](https://github.com/hecrj/coffee/blob/bda9818f823dfcb8a7ad0ff4940b4d4b387b5208/images/ui/checkbox.png?raw=true)
#[allow(missing_debug_implementations)]
pub struct Checkbox<'a, Message> {
    is_checked: bool,
    on_toggle: Rc<dyn Fn(bool) -> Message>,
    label: String,
    id: Option<String>,
    width: Length,
    #[allow(dead_code)]
    style_sheet: Box<dyn StyleSheet + 'a>,
}

impl<'a, Message> Checkbox<'a, Message> {
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
            on_toggle: Rc::new(f),
            label: label.into(),
            id: None,
            width: Length::Shrink,
            style_sheet: Default::default(),
        }
    }

    /// Sets the width of the [`Checkbox`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
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

    /// Sets the id of the [`Checkbox`].
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

impl<'a, Message> Widget<Message> for Checkbox<'a, Message>
where
    Message: 'static,
{
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
        style_sheet: &mut Css<'b>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::*;
        use dodrio::bumpalo::collections::String;

        let checkbox_label =
            String::from_str_in(&self.label, bump).into_bump_str();

        let event_bus = bus.clone();
        let on_toggle = self.on_toggle.clone();
        let is_checked = self.is_checked;

        let row_class = style_sheet.insert(bump, css::Rule::Row);

        let spacing_class = style_sheet.insert(bump, css::Rule::Spacing(5));

        let (label, input) = if let Some(id) = &self.id {
            let id = String::from_str_in(id, bump).into_bump_str();

            (label(bump).attr("for", id), input(bump).attr("id", id))
        } else {
            (label(bump), input(bump))
        };

        label
            .attr(
                "class",
                bumpalo::format!(in bump, "{} {}", row_class, spacing_class)
                    .into_bump_str(),
            )
            .attr(
                "style",
                bumpalo::format!(in bump, "width: {}; align-items: center", css::length(self.width))
                    .into_bump_str(),
            )
            .children(vec![
                // TODO: Checkbox styling
                 input
                    .attr("type", "checkbox")
                    .bool_attr("checked", self.is_checked)
                    .on("click", move |_root, vdom, _event| {
                        let msg = on_toggle(!is_checked);
                        event_bus.publish(msg);

                        vdom.schedule_render();
                    })
                    .finish(),
                text(checkbox_label),
            ])
            .finish()
    }
}

impl<'a, Message> From<Checkbox<'a, Message>> for Element<'a, Message>
where
    Message: 'static,
{
    fn from(checkbox: Checkbox<'a, Message>) -> Element<'a, Message> {
        Element::new(checkbox)
    }
}
