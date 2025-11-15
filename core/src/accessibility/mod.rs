//! Build and display accessibility information for widgets.
//!
//! This module provides types for describing widget semantics to
//! assistive technologies like screen readers.

use crate::widget::Id;

mod node;

pub use node::AccessibilityNode;

// Re-export commonly used AccessKit types
pub use accesskit::{Action, NodeId, Role};

/// Convert a widget::Id to a stable AccessKit NodeId.
///
/// This implementation ensures that the same widget ID always produces
/// the same NodeId, which is essential for stable accessibility trees.
///
/// # Example
/// ```
/// use iced_core::accessibility::NodeId;
/// use iced_core::widget::Id;
///
/// let widget_id = Id::new("my-button");
/// let node_id: NodeId = (&widget_id).into();
/// // The same widget_id always produces the same node_id
/// assert_eq!(node_id, NodeId::from(&widget_id));
/// ```
impl From<&Id> for NodeId {
    fn from(id: &Id) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        NodeId(hasher.finish())
    }
}
