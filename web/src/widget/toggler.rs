//! Show toggle controls using togglers.
use crate::{css, Bus, Css, Element, Length, Widget};

pub use iced_style::toggler::{Style, StyleSheet};

use dodrio::bumpalo;
use std::rc::Rc;

/// A toggler that can be toggled.
///
/// # Example
///
/// ```
/// # use iced_web::Toggler;
///
/// pub enum Message {
///     TogglerToggled(bool),
/// }
///
/// let is_active = true;
///
/// Toggler::new(is_active, String::from("Toggle me!"), Message::TogglerToggled);
/// ```
///
#[allow(missing_debug_implementations)]
pub struct Toggler<Message> {
    is_active: bool,
    on_toggle: Rc<dyn Fn(bool) -> Message>,
    label: Option<String>,
    id: Option<String>,
    width: Length,
    style: Box<dyn StyleSheet>,
}

impl<Message> Toggler<Message> {
    /// Creates a new [`Toggler`].
    ///
    /// It expects:
    ///   * a boolean describing whether the [`Toggler`] is active or not
    ///   * An optional label for the [`Toggler`]
    ///   * a function that will be called when the [`Toggler`] is toggled. It
    ///     will receive the new state of the [`Toggler`] and must produce a
    ///     `Message`.
    ///
    /// [`Toggler`]: struct.Toggler.html
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
            on_toggle: Rc::new(f),
            label: label.into(),
            id: None,
            width: Length::Shrink,
            style: Default::default(),
        }
    }

    /// Sets the width of the [`Toggler`].
    ///
    /// [`Toggler`]: struct.Toggler.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the style of the [`Toggler`].
    ///
    /// [`Toggler`]: struct.Toggler.html
    pub fn style(mut self, style: impl Into<Box<dyn StyleSheet>>) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the id of the [`Toggler`].
    ///
    /// [`Toggler`]: struct.Toggler.html
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

impl<Message> Widget<Message> for Toggler<Message>
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

        let toggler_label = &self
            .label
            .as_ref()
            .map(|label| String::from_str_in(&label, bump).into_bump_str());

        let event_bus = bus.clone();
        let on_toggle = self.on_toggle.clone();
        let is_active = self.is_active;

        let row_class = style_sheet.insert(bump, css::Rule::Row);
        let toggler_class = style_sheet.insert(bump, css::Rule::Toggler(16));

        let (label, input) = if let Some(id) = &self.id {
            let id = String::from_str_in(id, bump).into_bump_str();

            (label(bump).attr("for", id), input(bump).attr("id", id))
        } else {
            (label(bump), input(bump))
        };

        let checkbox = input
            .attr("type", "checkbox")
            .bool_attr("checked", self.is_active)
            .on("click", move |_root, vdom, _event| {
                let msg = on_toggle(!is_active);
                event_bus.publish(msg);

                vdom.schedule_render();
            })
            .finish();

        let toggler = span(bump).children(vec![span(bump).finish()]).finish();

        label
            .attr(
                "class",
                bumpalo::format!(in bump, "{} {}", row_class, toggler_class)
                    .into_bump_str(),
            )
            .attr(
                "style",
                bumpalo::format!(in bump, "width: {}; align-items: center", css::length(self.width))
                .into_bump_str()
            )
            .children(
                if let Some(label) = toggler_label {
                    vec![
                        text(label),
                        checkbox,
                        toggler,
                    ]
                } else {
                    vec![
                        checkbox,
                        toggler,
                    ]
                }
            )
            .finish()
    }
}

impl<'a, Message> From<Toggler<Message>> for Element<'a, Message>
where
    Message: 'static,
{
    fn from(toggler: Toggler<Message>) -> Element<'a, Message> {
        Element::new(toggler)
    }
}
