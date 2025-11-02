//! Accessibility tree construction and management.

use crate::core::Element;
use crate::core::layout::Layout;
use crate::core::widget::Tree;

use accesskit::{Node, NodeId, Role, Tree as AccessKitTree, TreeUpdate};

use std::collections::HashMap;

/// Builds an AccessKit tree from a widget tree.
///
/// This traverses the widget tree, collecting accessibility information
/// from each widget and building a tree structure that AccessKit can consume.
pub fn build_accessibility_tree<Message, Theme, Renderer>(
    root: &Element<'_, Message, Theme, Renderer>,
    state: &Tree,
    layout: Layout<'_>,
) -> TreeUpdate
where
    Renderer: crate::core::Renderer,
{
    let mut builder = TreeBuilder::new();

    // Traverse the widget tree and collect accessibility nodes
    builder.traverse(root, state, layout, &[]);

    builder.build()
}

/// Helper struct for building the accessibility tree.
struct TreeBuilder {
    /// Map from stable IDs to AccessKit nodes
    nodes: HashMap<NodeId, Node>,

    /// The root node ID
    root_id: NodeId,
}

impl TreeBuilder {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root_id: NodeId(0),
        }
    }

    /// Traverse a widget and its children, collecting accessibility information.
    fn traverse<Message, Theme, Renderer>(
        &mut self,
        element: &Element<'_, Message, Theme, Renderer>,
        state: &Tree,
        layout: Layout<'_>,
        path: &[usize],
    ) where
        Renderer: crate::core::Renderer,
    {
        // Generate stable ID from path
        let node_id = self.path_to_node_id(path);

        // Get accessibility information from the widget
        if let Some(accessibility_node) =
            element.as_widget().accessibility(state, layout)
        {
            // Build AccessKit node from our AccessibilityNode
            let mut node = Node::new(
                accessibility_node.role.unwrap_or(Role::GenericContainer),
            );

            // Set bounds (AccessKit uses screen coordinates)
            let bounds = accessibility_node.bounds;
            node.set_bounds(accesskit::Rect {
                x0: bounds.x as f64,
                y0: bounds.y as f64,
                x1: (bounds.x + bounds.width) as f64,
                y1: (bounds.y + bounds.height) as f64,
            });

            // Set label if present
            if let Some(label) = accessibility_node.label {
                node.set_label(label);
            }

            // Set value if present
            if let Some(value) = accessibility_node.value {
                node.set_value(value);
            }

            // Set enabled state
            if !accessibility_node.enabled {
                node.set_disabled();
            }

            // Note: AccessKit doesn't have a direct "focusable" property.
            // Focusability is determined by the role and supported actions.
            // The focusable field in AccessibilityNode is informational only.

            // Collect children IDs
            let mut child_ids = Vec::new();
            for (index, _child_layout) in layout.children().enumerate() {
                let mut child_path = path.to_vec();
                child_path.push(index);

                // Recursively traverse children
                // Note: We need children states, but we don't have direct access yet
                // This is a limitation we'll need to address

                let child_id = self.path_to_node_id(&child_path);
                child_ids.push(child_id);
            }

            if !child_ids.is_empty() {
                node.set_children(child_ids);
            }

            let _ = self.nodes.insert(node_id, node);
        }

        // If widget returned None for accessibility, it's transparent to the tree
        // We still need to traverse its children, but they become direct children
        // of the parent instead
    }

    /// Convert a path to a stable NodeId.
    ///
    /// This uses a simple hash of the path to generate a stable ID.
    /// For the root (empty path), we use NodeId(0).
    fn path_to_node_id(&self, path: &[usize]) -> NodeId {
        if path.is_empty() {
            return self.root_id;
        }

        // Simple hash: combine indices with prime multipliers
        let mut hash: u64 = 1;
        for &index in path {
            hash = hash.wrapping_mul(31).wrapping_add(index as u64);
        }

        // Ensure we don't use 0 (reserved for root)
        NodeId(if hash == 0 { 1 } else { hash })
    }

    /// Build the final AccessKit TreeUpdate.
    fn build(self) -> TreeUpdate {
        TreeUpdate {
            nodes: self.nodes.into_iter().collect(),
            tree: Some(AccessKitTree::new(self.root_id)),
            focus: self.root_id,
        }
    }
}
