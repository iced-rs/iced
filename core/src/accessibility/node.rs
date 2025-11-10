//! Accessibility node for describing widget semantics.

use crate::Rectangle;
use crate::widget::Id;
use std::any::Any;

pub use accesskit::Role;

/// An accessibility node describing a widget's semantics.
///
/// This is a simplified wrapper around AccessKit's node types,
/// providing a builder API for common accessibility properties.
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

    /// Optional widget ID for maximum accessibility tree stability.
    ///
    /// When provided, this ID is used to generate stable AccessKit NodeIds
    /// that remain consistent even when the widget tree structure changes.
    /// This is particularly useful for dynamic lists or frequently changing UIs.
    ///
    /// If not provided, a path-based ID is generated automatically.
    pub widget_id: Option<Id>,

    /// Whether this widget is a leaf node in the accessibility tree.
    ///
    /// When `true`, child widgets will not have their accessibility nodes
    /// included in the tree. This is useful for widgets like buttons that
    /// want to present themselves as a single accessible element with their
    /// content embedded in the label, rather than as a container with children.
    ///
    /// Default: `false`
    pub is_leaf_node: bool,

    /// Optional action callback that will be invoked when an accessibility action occurs.
    ///
    /// This is a closure that produces a type-erased Message when called.
    /// It will be invoked when an accessibility action (like Click) is performed on this node.
    /// TreeBuilder will extract this and store it in the action callback map.
    ///
    /// Using a closure avoids requiring Clone on the Message type, following iced's pattern.
    pub on_action: Option<Box<dyn Fn() -> Box<dyn Any + Send> + Send>>,
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
            widget_id: None,
            is_leaf_node: false,
            on_action: None,
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

    /// Sets an optional widget ID for maximum accessibility tree stability.
    ///
    /// When a widget ID is provided, it's used to generate a stable AccessKit
    /// NodeId that remains consistent across UI updates, even when the widget
    /// tree structure changes. This is particularly important for dynamic lists
    /// and frequently changing UIs.
    ///
    /// If no widget ID is provided, a path-based ID is generated automatically
    /// based on the widget's position in the tree.
    ///
    /// # Example
    /// ```
    /// use iced_core::accessibility::AccessibilityNode;
    /// use iced_core::widget::Id;
    /// use iced_core::Rectangle;
    ///
    /// let node = AccessibilityNode::new(Rectangle::default())
    ///     .widget_id(Some(Id::new("my-button")));
    /// ```
    pub fn widget_id(mut self, id: Option<Id>) -> Self {
        self.widget_id = id;
        self
    }

    /// Sets whether this widget is a leaf node in the accessibility tree.
    ///
    /// When set to `true`, child widgets will not have their accessibility
    /// nodes included in the tree. This is useful for widgets like buttons
    /// that want to present themselves as a single accessible element rather
    /// than as a container with children.
    ///
    /// # Example
    /// ```
    /// use iced_core::accessibility::{AccessibilityNode, Role};
    /// use iced_core::Rectangle;
    ///
    /// let node = AccessibilityNode::new(Rectangle::default())
    ///     .role(Role::Button)
    ///     .label("Click me")
    ///     .is_leaf_node(true);
    /// ```
    pub fn is_leaf_node(mut self, is_leaf: bool) -> Self {
        self.is_leaf_node = is_leaf;
        self
    }

    /// Sets the action callback for this widget.
    ///
    /// When an accessibility action (like Click) is performed on this node,
    /// the provided message will be published to the application.
    ///
    /// The message is captured in a closure to avoid requiring Clone on the Message type.
    ///
    /// # Example
    /// ```ignore
    /// use iced_core::accessibility::{AccessibilityNode, Role};
    /// use iced_core::Rectangle;
    ///
    /// #[derive(Clone)]
    /// enum Message {
    ///     ButtonPressed,
    /// }
    ///
    /// let node = AccessibilityNode::new(Rectangle::default())
    ///     .role(Role::Button)
    ///     .on_action(Message::ButtonPressed);
    /// ```
    pub fn on_action<M>(mut self, message: M) -> Self
    where
        M: 'static + Clone + Send,
    {
        self.on_action = Some(Box::new(move || Box::new(message.clone())));
        self
    }
}

