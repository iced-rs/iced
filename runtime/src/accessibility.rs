//! Accessibility tree construction and management.

use crate::core::Rectangle;
use crate::core::widget::{Id, Operation, operation};
use crate::user_interface::UserInterface;

use accesskit::{
    ActionRequest, Node, NodeId, Role, Tree as AccessKitTree, TreeUpdate,
};

use std::collections::HashMap;

/// An accessibility action to be performed by the runtime.
#[derive(Debug, Clone)]
pub enum Action {
    /// An action was requested by an assistive technology.
    ActionRequested(ActionRequest),
    /// Accessibility was deactivated.
    Deactivated,
}

/// Builds an accessibility tree from a UserInterface.
///
/// This traverses the widget tree and collects accessibility information.
pub fn build_tree_from_ui<Message, Theme, Renderer>(
    ui: &mut UserInterface<'_, Message, Theme, Renderer>,
    renderer: &Renderer,
) -> TreeUpdate
where
    Renderer: crate::core::Renderer,
{
    let mut builder = TreeBuilder::new();
    ui.operate(renderer, &mut builder);
    builder.build()
}

/// Helper struct for building the accessibility tree via Operation pattern.
pub struct TreeBuilder {
    nodes: HashMap<NodeId, Node>,
    node_id_counter: u64,
}

impl TreeBuilder {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            node_id_counter: 1, // Start at 1, 0 is reserved for root
        }
    }

    fn next_id(&mut self) -> NodeId {
        let id = NodeId(self.node_id_counter);
        self.node_id_counter += 1;
        id
    }

    fn build(mut self) -> TreeUpdate {
        // Ensure we have at least a root node
        if self.nodes.is_empty() {
            let mut root = Node::new(Role::Window);
            root.set_label("Application".to_string());
            let _ = self.nodes.insert(NodeId(0), root);
        }

        TreeUpdate {
            nodes: self.nodes.into_iter().collect(),
            tree: Some(AccessKitTree::new(NodeId(0))),
            focus: NodeId(0),
        }
    }
}

impl Operation for TreeBuilder {
    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        let node_id = self.next_id();

        let mut node = Node::new(Role::GenericContainer);
        node.set_bounds(accesskit::Rect {
            x0: bounds.x as f64,
            y0: bounds.y as f64,
            x1: (bounds.x + bounds.width) as f64,
            y1: (bounds.y + bounds.height) as f64,
        });

        if let Some(id) = id {
            node.set_label(format!("Container {:?}", id));
        }

        let _ = self.nodes.insert(node_id, node);
    }

    fn focusable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        _state: &mut dyn operation::Focusable,
    ) {
        let node_id = self.next_id();

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

        let _ = self.nodes.insert(node_id, node);
    }

    fn text_input(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        _state: &mut dyn operation::TextInput,
    ) {
        let node_id = self.next_id();

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
        let node_id = self.next_id();

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
        let node_id = self.next_id();

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

    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }
}
