//! Accessibility tree construction and management.

use crate::core::Rectangle;
use crate::core::widget::{Id, Operation, operation};

use accesskit::{Node, NodeId, Role, Tree as AccessKitTree, TreeUpdate};

use std::collections::HashMap;

/// Produces an [`Operation`] that builds an accessibility tree.
///
/// This operation traverses the widget tree and collects accessibility
/// information from each widget to build an AccessKit TreeUpdate.
pub fn build_tree() -> impl Operation<TreeUpdate> {
    struct BuildTree {
        nodes: HashMap<NodeId, Node>,
        node_id_stack: Vec<NodeId>,
        current_index: usize,
    }

    impl BuildTree {
        fn new() -> Self {
            Self {
                nodes: HashMap::new(),
                node_id_stack: vec![NodeId(0)], // Start with root
                current_index: 0,
            }
        }

        /// Generate a stable NodeId from the current path.
        fn generate_node_id(&self) -> NodeId {
            if self.node_id_stack.len() == 1 {
                return NodeId(0); // Root
            }

            // Hash the path to create a stable ID
            let mut hash: u64 = 1;
            for &NodeId(id) in &self.node_id_stack {
                hash = hash.wrapping_mul(31).wrapping_add(id);
            }
            hash = hash
                .wrapping_mul(31)
                .wrapping_add(self.current_index as u64);

            NodeId(if hash == 0 { 1 } else { hash })
        }
    }

    impl Operation<TreeUpdate> for BuildTree {
        fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
            // Containers can optionally provide accessibility info
            // For now, we'll create a generic container node
            let node_id = self.generate_node_id();

            let mut node = Node::new(Role::GenericContainer);
            node.set_bounds(accesskit::Rect {
                x0: bounds.x as f64,
                y0: bounds.y as f64,
                x1: (bounds.x + bounds.width) as f64,
                y1: (bounds.y + bounds.height) as f64,
            });

            // Use widget ID as label if available
            if let Some(id) = id {
                node.set_label(format!("Container {:?}", id));
            }

            let _ = self.nodes.insert(node_id, node);
        }

        fn focusable(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            state: &mut dyn operation::Focusable,
        ) {
            let node_id = self.generate_node_id();

            // Focusable widgets are likely buttons or inputs
            let mut node = Node::new(Role::Button);
            node.set_bounds(accesskit::Rect {
                x0: bounds.x as f64,
                y0: bounds.y as f64,
                x1: (bounds.x + bounds.width) as f64,
                y1: (bounds.y + bounds.height) as f64,
            });

            if let Some(id) = id {
                node.set_label(format!("Focusable {:?}", id));
            }

            if state.is_focused() {
                // This node is currently focused
                // We'll handle this when building the final tree
            }

            let _ = self.nodes.insert(node_id, node);
        }

        fn text_input(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            _state: &mut dyn operation::TextInput,
        ) {
            let node_id = self.generate_node_id();

            let mut node = Node::new(Role::TextInput);
            node.set_bounds(accesskit::Rect {
                x0: bounds.x as f64,
                y0: bounds.y as f64,
                x1: (bounds.x + bounds.width) as f64,
                y1: (bounds.y + bounds.height) as f64,
            });

            if let Some(id) = id {
                node.set_label(format!("TextInput {:?}", id));
            }

            let _ = self.nodes.insert(node_id, node);
        }

        fn text(&mut self, _id: Option<&Id>, bounds: Rectangle, text: &str) {
            let node_id = self.generate_node_id();

            let mut node = Node::new(Role::TextRun);
            node.set_bounds(accesskit::Rect {
                x0: bounds.x as f64,
                y0: bounds.y as f64,
                x1: (bounds.x + bounds.width) as f64,
                y1: (bounds.y + bounds.height) as f64,
            });
            node.set_label(text.to_string());

            let _ = self.nodes.insert(node_id, node);
        }

        fn scrollable(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            _content_bounds: Rectangle,
            _translation: crate::core::Vector,
            _state: &mut dyn operation::Scrollable,
        ) {
            let node_id = self.generate_node_id();

            let mut node = Node::new(Role::ScrollView);
            node.set_bounds(accesskit::Rect {
                x0: bounds.x as f64,
                y0: bounds.y as f64,
                x1: (bounds.x + bounds.width) as f64,
                y1: (bounds.y + bounds.height) as f64,
            });

            if let Some(id) = id {
                node.set_label(format!("Scrollable {:?}", id));
            }

            let _ = self.nodes.insert(node_id, node);
        }

        fn traverse(
            &mut self,
            operate: &mut dyn FnMut(&mut dyn Operation<TreeUpdate>),
        ) {
            // Continue traversal
            operate(self);
        }

        fn finish(&self) -> operation::Outcome<TreeUpdate> {
            if self.nodes.is_empty() {
                // Create a minimal root node
                let mut root = Node::new(Role::Window);
                root.set_label("Application".to_string());

                let nodes = vec![(NodeId(0), root)];

                operation::Outcome::Some(TreeUpdate {
                    nodes,
                    tree: Some(AccessKitTree::new(NodeId(0))),
                    focus: NodeId(0),
                })
            } else {
                operation::Outcome::Some(TreeUpdate {
                    nodes: self.nodes.clone().into_iter().collect(),
                    tree: Some(AccessKitTree::new(NodeId(0))),
                    focus: NodeId(0),
                })
            }
        }
    }

    BuildTree::new()
}
