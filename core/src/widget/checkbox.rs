//! Show toggle controls using checkboxes.
use crate::Color;

/// A box that can be checked.
///
/// # Example
///
/// ```
/// use iced_core::Checkbox;
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
pub struct Checkbox<Message> {
    /// Whether the checkbox is checked or not
    pub is_checked: bool,

    /// Function to call when checkbox is toggled to produce a __message__.
    ///
    /// The function should be provided `true` when the checkbox is checked
    /// and `false` otherwise.
    pub on_toggle: Box<dyn Fn(bool) -> Message>,

    /// The label of the checkbox
    pub label: String,

    /// The color of the label
    pub label_color: Option<Color>,
}

impl<Message> std::fmt::Debug for Checkbox<Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Checkbox")
            .field("is_checked", &self.is_checked)
            .field("label", &self.label)
            .field("label_color", &self.label_color)
            .finish()
    }
}

impl<Message> Checkbox<Message> {
    /// Creates a new [`Checkbox`].
    ///
    /// It expects:
    ///   * a boolean describing whether the [`Checkbox`] is checked or not
    ///   * the label of the [`Checkbox`]
    ///   * a function that will be called when the [`Checkbox`] is toggled.
    ///     It will receive the new state of the [`Checkbox`] and must produce
    ///     a `Message`.
    ///
    /// [`Checkbox`]: struct.Checkbox.html
    pub fn new<F>(is_checked: bool, label: &str, f: F) -> Self
    where
        F: 'static + Fn(bool) -> Message,
    {
        Checkbox {
            is_checked,
            on_toggle: Box::new(f),
            label: String::from(label),
            label_color: None,
        }
    }

    /// Sets the color of the label of the [`Checkbox`].
    ///
    /// [`Checkbox`]: struct.Checkbox.html
    pub fn label_color<C: Into<Color>>(mut self, color: C) -> Self {
        self.label_color = Some(color.into());
        self
    }
}
