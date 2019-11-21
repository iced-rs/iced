use crate::{Bus, Color, Element, Widget};

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
pub struct Radio<Message> {
    is_selected: bool,
    on_click: Message,
    label: String,
    label_color: Option<Color>,
}

impl<Message> Radio<Message> {
    /// Creates a new [`Radio`] button.
    ///
    /// It expects:
    ///   * the value related to the [`Radio`] button
    ///   * the label of the [`Radio`] button
    ///   * the current selected value
    ///   * a function that will be called when the [`Radio`] is selected. It
    ///   receives the value of the radio and must produce a `Message`.
    ///
    /// [`Radio`]: struct.Radio.html
    pub fn new<F, V>(value: V, label: &str, selected: Option<V>, f: F) -> Self
    where
        V: Eq + Copy,
        F: 'static + Fn(V) -> Message,
    {
        Radio {
            is_selected: Some(value) == selected,
            on_click: f(value),
            label: String::from(label),
            label_color: None,
        }
    }

    /// Sets the `Color` of the label of the [`Radio`].
    ///
    /// [`Radio`]: struct.Radio.html
    pub fn label_color<C: Into<Color>>(mut self, color: C) -> Self {
        self.label_color = Some(color.into());
        self
    }
}

impl<Message> Widget<Message> for Radio<Message>
where
    Message: 'static + Copy,
{
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::*;

        let radio_label = bumpalo::format!(in bump, "{}", self.label);

        let event_bus = bus.clone();
        let on_click = self.on_click;

        // TODO: Complete styling
        label(bump)
            .attr("style", "display: block")
            .children(vec![
                input(bump)
                    .attr("type", "radio")
                    .bool_attr("checked", self.is_selected)
                    .on("click", move |root, vdom, _event| {
                        event_bus.publish(on_click, root);

                        vdom.schedule_render();
                    })
                    .finish(),
                text(radio_label.into_bump_str()),
            ])
            .finish()
    }
}

impl<'a, Message> From<Radio<Message>> for Element<'a, Message>
where
    Message: 'static + Copy,
{
    fn from(radio: Radio<Message>) -> Element<'a, Message> {
        Element::new(radio)
    }
}
