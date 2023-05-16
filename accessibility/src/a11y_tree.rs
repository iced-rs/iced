use crate::{A11yId, A11yNode};

#[derive(Debug, Clone, Default)]
/// Accessible tree of nodes
pub struct A11yTree {
    /// The root of the current widget, children of the parent widget or the Window if there is no parent widget
    root: Vec<A11yNode>,
    /// The children of a widget and its children
    children: Vec<A11yNode>,
}

impl A11yTree {
    /// Create a new A11yTree
    /// XXX if you use this method, you will need to manually add the children of the root nodes
    pub fn new(root: Vec<A11yNode>, children: Vec<A11yNode>) -> Self {
        Self { root, children }
    }

    pub fn leaf<T: Into<A11yId>>(node: accesskit::NodeBuilder, id: T) -> Self {
        Self {
            root: vec![A11yNode::new(node, id)],
            children: vec![],
        }
    }

    /// Helper for creating an A11y tree with a single root node and some children
    pub fn node_with_child_tree(mut root: A11yNode, child_tree: Self) -> Self {
        root.add_children(
            child_tree.root.iter().map(|n| n.id()).cloned().collect(),
        );
        Self {
            root: vec![root],
            children: child_tree
                .children
                .into_iter()
                .chain(child_tree.root)
                .collect(),
        }
    }

    /// Joins multiple trees into a single tree
    pub fn join<T: Iterator<Item = Self>>(trees: T) -> Self {
        trees.fold(Self::default(), |mut acc, A11yTree { root, children }| {
            acc.root.extend(root);
            acc.children.extend(children);
            acc
        })
    }

    pub fn root(&self) -> &Vec<A11yNode> {
        &self.root
    }

    pub fn children(&self) -> &Vec<A11yNode> {
        &self.children
    }

    pub fn root_mut(&mut self) -> &mut Vec<A11yNode> {
        &mut self.root
    }

    pub fn children_mut(&mut self) -> &mut Vec<A11yNode> {
        &mut self.children
    }

    pub fn contains(&self, id: &A11yId) -> bool {
        self.root.iter().any(|n| n.id() == id)
            || self.children.iter().any(|n| n.id() == id)
    }
}

impl From<A11yTree> for Vec<(accesskit::NodeId, accesskit::Node)> {
    fn from(tree: A11yTree) -> Vec<(accesskit::NodeId, accesskit::Node)> {
        tree.root
            .into_iter()
            .map(|node| node.into())
            .chain(tree.children.into_iter().map(|node| node.into()))
            .collect()
    }
}
