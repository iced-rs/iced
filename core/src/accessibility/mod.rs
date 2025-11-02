//! Build and display accessibility information for widgets.
//!
//! This module provides types for describing widget semantics to
//! assistive technologies like screen readers.

mod node;

pub use node::AccessibilityNode;

// Re-export commonly used AccessKit types
pub use accesskit::{Action, NodeId, Role};
