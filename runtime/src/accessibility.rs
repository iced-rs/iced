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
/// Returns the TreeUpdate, bounds mapping, and action callbacks (as closures) for routing.
pub fn build_tree_from_ui<Message, Theme, Renderer>(
    ui: &mut UserInterface<'_, Message, Theme, Renderer>,
    renderer: &Renderer,
) -> (TreeUpdate, HashMap<NodeId, Rectangle>, HashMap<NodeId, Box<dyn Fn() -> Message + Send>>)
where
    Message: Send + 'static,
    Renderer: crate::core::Renderer,
{
    let mut builder = TreeBuilder::new();
    ui.operate(renderer, &mut builder);
    builder.build()
}

/// Helper struct for building the accessibility tree via Operation pattern.
pub struct TreeBuilder<Message = ()> {
    nodes: HashMap<NodeId, Node>,
    children: Vec<NodeId>,
    /// Mapping of NodeId to bounds for action routing
    node_bounds: HashMap<NodeId, Rectangle>,
    /// Mapping of NodeId to action callbacks (closures that produce messages)
    action_callbacks: HashMap<NodeId, Box<dyn Fn() -> Message + Send>>,
    /// Path stack for generating stable IDs based on widget tree position
    path_stack: Vec<String>,
    /// Counter for each widget type at current level
    type_counters: HashMap<String, usize>,
    /// Whether we're inside a leaf node (children should not create accessibility nodes)
    inside_leaf_node: bool,
}

impl<Message> TreeBuilder<Message> {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            children: Vec::new(),
            node_bounds: HashMap::new(),
            action_callbacks: HashMap::new(),
            path_stack: vec!["window".to_string()],
            type_counters: HashMap::new(),
            inside_leaf_node: false,
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
            Some(id) => {
                // Convert widget::Id to NodeId via deterministic hashing
                NodeId::from(id)
            }

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

    fn build(mut self) -> (TreeUpdate, HashMap<NodeId, Rectangle>, HashMap<NodeId, Box<dyn Fn() -> Message + Send>>) {
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

        (tree_update, self.node_bounds, self.action_callbacks)
    }
}

impl<Message> TreeBuilder<Message> {
    /// Register an action callback for a node.
    /// This allows widgets to specify what closure should be called when an accessibility action occurs.
    pub fn register_action(&mut self, node_id: NodeId, callback: Box<dyn Fn() -> Message + Send>) {
        let _ = self.action_callbacks.insert(node_id, callback);
    }
}

impl<Message: Send + 'static> Operation for TreeBuilder<Message> {
    fn accessibility(
        &mut self,
        accessibility_node: Option<
            crate::core::accessibility::AccessibilityNode,
        >,
    ) {
        // If we're inside a leaf node, don't add child nodes to the tree
        if self.inside_leaf_node {
            return;
        }

        if let Some(a11y_node) = accessibility_node {
            // Store whether this widget is a leaf node
            let is_leaf = a11y_node.is_leaf_node;

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

                // Extract and store action callback if present
                if let Some(action_closure) = a11y_node.on_action {
                    // The closure produces Box<dyn Any + Send>, we need to downcast and create a new closure
                    let callback = Box::new(move || {
                        let any_box = action_closure();
                        // Downcast the Any box to the concrete Message type
                        *any_box.downcast::<Message>().expect("Message type mismatch")
                    });
                    let _ = self.action_callbacks.insert(node_id, callback);
                }

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

            // If this is a leaf node, set the flag so children won't be added
            // This will affect the subsequent traverse() call
            if is_leaf {
                self.inside_leaf_node = true;
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
        // Perform the traversal
        // If inside_leaf_node is true, child nodes won't be added to the tree
        operate(self);

        // Reset the flag after traversing
        // This ensures the flag only affects immediate children of the leaf node,
        // not siblings or other widgets at the same level
        self.inside_leaf_node = false;
    }
}
