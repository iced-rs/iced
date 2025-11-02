//! Accessibility node for describing widget semantics.

use crate::Rectangle;

pub use accesskit::Role;

/// An accessibility node describing a widget's semantics.
///
/// This is a simplified wrapper around AccessKit's node types,
/// providing a builder API for common accessibility properties.
#[derive(Debug, Clone)]
pub struct AccessibilityNode {
    /// The bounding box of the widget in screen coordinates
    pub bounds: Rectangle,

    /// The semantic role of this widget
    pub role: Option<Role>,

    /// The accessible label/name for this widget
    pub label: Option<String>,

    /// The current value (for inputs, sliders, etc.)
    pub value: Option<String>,

    /// Whether the widget is enabled for interaction
    pub enabled: bool,

    /// Whether the widget can receive keyboard focus
    pub focusable: bool,
}

impl AccessibilityNode {
    /// Creates a new [`AccessibilityNode`] with the given bounds.
    ///
    /// By default, the node has no role, label, or value, and is enabled
    /// but not focusable.
    ///
    /// # Example
    /// ```
    /// use iced_core::accessibility::AccessibilityNode;
    /// use iced_core::Rectangle;
    ///
    /// let node = AccessibilityNode::new(Rectangle {
    ///     x: 0.0,
    ///     y: 0.0,
    ///     width: 100.0,
    ///     height: 50.0,
    /// });
    /// ```
    pub fn new(bounds: Rectangle) -> Self {
        Self {
            bounds,
            role: None,
            label: None,
            value: None,
            enabled: true,
            focusable: false,
        }
    }

    /// Sets the semantic role of this widget.
    ///
    /// # Example
    /// ```
    /// use iced_core::accessibility::{AccessibilityNode, Role};
    /// use iced_core::Rectangle;
    ///
    /// let node = AccessibilityNode::new(Rectangle::default())
    ///     .role(Role::Button);
    /// ```
    pub fn role(mut self, role: Role) -> Self {
        self.role = Some(role);
        self
    }

    /// Sets the accessible label for this widget.
    ///
    /// # Example
    /// ```
    /// use iced_core::accessibility::AccessibilityNode;
    /// use iced_core::Rectangle;
    ///
    /// let node = AccessibilityNode::new(Rectangle::default())
    ///     .label("Click me");
    /// ```
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the current value for this widget.
    ///
    /// Typically used for text inputs, sliders, or other widgets with state.
    ///
    /// # Example
    /// ```
    /// use iced_core::accessibility::AccessibilityNode;
    /// use iced_core::Rectangle;
    ///
    /// let node = AccessibilityNode::new(Rectangle::default())
    ///     .value("Hello, world!");
    /// ```
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Sets whether this widget is enabled for interaction.
    ///
    /// Disabled widgets are typically grayed out and don't respond to input.
    ///
    /// # Example
    /// ```
    /// use iced_core::accessibility::AccessibilityNode;
    /// use iced_core::Rectangle;
    ///
    /// let node = AccessibilityNode::new(Rectangle::default())
    ///     .enabled(false);
    /// ```
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets whether this widget can receive keyboard focus.
    ///
    /// # Example
    /// ```
    /// use iced_core::accessibility::AccessibilityNode;
    /// use iced_core::Rectangle;
    ///
    /// let node = AccessibilityNode::new(Rectangle::default())
    ///     .focusable(true);
    /// ```
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }
}
