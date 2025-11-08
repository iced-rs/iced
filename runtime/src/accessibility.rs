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
/// Returns the TreeUpdate and a mapping of NodeId to bounds for action routing.
pub fn build_tree_from_ui<Message, Theme, Renderer>(
    ui: &mut UserInterface<'_, Message, Theme, Renderer>,
    renderer: &Renderer,
) -> (TreeUpdate, HashMap<NodeId, Rectangle>)
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
    children: Vec<NodeId>,
    /// Mapping of NodeId to bounds for action routing
    node_bounds: HashMap<NodeId, Rectangle>,
    /// Path stack for generating stable IDs based on widget tree position
    path_stack: Vec<String>,
    /// Counter for each widget type at current level
    type_counters: HashMap<String, usize>,
    /// Cache mapping widget::Id to stable AccessKit NodeId
    id_cache: HashMap<Id, NodeId>,
}

impl TreeBuilder {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            children: Vec::new(),
            node_bounds: HashMap::new(),
            path_stack: vec!["window".to_string()],
            type_counters: HashMap::new(),
            id_cache: HashMap::new(),
        }
    }

    /// Generate a stable NodeId based on widget::Id (preferred) or tree position (fallback)
    ///
    /// Priority:
    /// 1. If widget_id is provided, use it for maximum stability
    /// 2. Otherwise, fall back to path-based hashing
    fn generate_stable_id(
        &mut self,
        widget_type: &str,
        widget_id: Option<&Id>,
    ) -> NodeId {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        match widget_id {
            Some(id) => self
                .id_cache
                .entry(id.clone())
                .or_insert_with(|| NodeId::from(id))
                .clone(),

            None => {
                let counter = self
                    .type_counters
                    .entry(widget_type.to_string())
                    .or_insert(0);
                let index = *counter;
                *counter += 1;

                // Build path string like "window/column/button[0]"
                let mut path = self.path_stack.join("/");
                path.push_str(&format!("/{}[{}]", widget_type, index));

                // Hash the path to get stable u64 ID
                let mut hasher = DefaultHasher::new();
                path.hash(&mut hasher);
                NodeId(hasher.finish())
            }
        }
    }

    fn build(mut self) -> (TreeUpdate, HashMap<NodeId, Rectangle>) {
        // Create root node and add all collected nodes as children
        let mut root = Node::new(Role::Window);
        root.set_label("Iced Application".to_string());
        root.set_children(self.children);

        let _ = self.nodes.insert(NodeId(0), root);

        let tree_update = TreeUpdate {
            nodes: self.nodes.into_iter().collect(),
            tree: Some(AccessKitTree::new(NodeId(0))),
            focus: NodeId(0),
        };

        (tree_update, self.node_bounds)
    }
}

impl Operation for TreeBuilder {
    fn accessibility(
        &mut self,
        accessibility_node: Option<
            crate::core::accessibility::AccessibilityNode,
        >,
    ) {
        if let Some(a11y_node) = accessibility_node {
            // Role is required for AccessKit nodes
            if let Some(role) = a11y_node.role {
                // Generate stable ID based on role type and position
                let widget_type = match role {
                    Role::Button => "button",
                    Role::Label => "label",
                    Role::TextInput => "textinput",
                    Role::CheckBox => "checkbox",
                    Role::Slider => "slider",
                    Role::Image => "image",
                    Role::Link => "link",
                    _ => "widget",
                };
                // NEW: Pass widget_id to generate_stable_id for hybrid approach
                let node_id = self.generate_stable_id(
                    widget_type,
                    a11y_node.widget_id.as_ref(),
                );

                // Convert iced AccessibilityNode to AccessKit Node
                let mut node = Node::new(role);

                // Set bounds
                node.set_bounds(accesskit::Rect {
                    x0: a11y_node.bounds.x as f64,
                    y0: a11y_node.bounds.y as f64,
                    x1: (a11y_node.bounds.x + a11y_node.bounds.width) as f64,
                    y1: (a11y_node.bounds.y + a11y_node.bounds.height) as f64,
                });

                // Set label if present
                if let Some(label) = a11y_node.label {
                    node.set_label(label);
                }

                // Set value if present
                if let Some(value) = a11y_node.value {
                    node.set_value(value);
                }

                // Set other properties
                if a11y_node.focusable {
                    node.add_action(accesskit::Action::Focus);
                }

                // Add Click action for buttons
                if role == Role::Button && a11y_node.enabled {
                    node.add_action(accesskit::Action::Click);
                }

                if a11y_node.enabled {
                    // Enabled state is implicit; we mark disabled state
                } else {
                    node.set_disabled();
                }

                self.children.push(node_id);
                let _ = self.nodes.insert(node_id, node);

                // Store bounds for action routing
                let _ = self.node_bounds.insert(node_id, a11y_node.bounds);
            }
        }
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        // NEW: Pass widget ID through to generate_stable_id
        let node_id = self.generate_stable_id("container", id);

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

        self.children.push(node_id);
        let _ = self.nodes.insert(node_id, node);
    }

    fn focusable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        _state: &mut dyn operation::Focusable,
    ) {
        // NEW: Pass widget ID through to generate_stable_id
        let node_id = self.generate_stable_id("focusable", id);

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

        self.children.push(node_id);
        let _ = self.nodes.insert(node_id, node);
    }

    fn text_input(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        _state: &mut dyn operation::TextInput,
    ) {
        // NEW: Pass widget ID through to generate_stable_id
        let node_id = self.generate_stable_id("textinput", id);

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

        self.children.push(node_id);
        let _ = self.nodes.insert(node_id, node);
    }

    fn text(&mut self, id: Option<&Id>, bounds: Rectangle, text: &str) {
        // NEW: Pass widget ID through to generate_stable_id (though text rarely has IDs)
        let node_id = self.generate_stable_id("text", id);

        let mut node = Node::new(Role::TextRun);
        node.set_bounds(accesskit::Rect {
            x0: bounds.x as f64,
            y0: bounds.y as f64,
            x1: (bounds.x + bounds.width) as f64,
            y1: (bounds.y + bounds.height) as f64,
        });
        node.set_label(text.to_string());

        self.children.push(node_id);
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
        // NEW: Pass widget ID through to generate_stable_id
        let node_id = self.generate_stable_id("scrollable", id);

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

        self.children.push(node_id);
        let _ = self.nodes.insert(node_id, node);
    }

    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }
}
