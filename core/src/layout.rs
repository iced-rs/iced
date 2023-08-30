//! Position your widgets properly.
mod limits;
mod node;

pub mod flex;

pub use limits::Limits;
pub use node::Node;

use crate::{Point, Rectangle, Size, Vector};

/// The bounds of a [`Node`] and its children, using absolute coordinates.
#[derive(Debug, Clone, Copy)]
pub struct Layout<'a> {
    position: Point,
    node: &'a Node,
}

impl<'a> Layout<'a> {
    /// Creates a new [`Layout`] for the given [`Node`] at the origin.
    pub fn new(node: &'a Node) -> Self {
        Self::with_offset(Vector::new(0.0, 0.0), node)
    }

    /// Creates a new [`Layout`] for the given [`Node`] with the provided offset
    /// from the origin.
    pub fn with_offset(offset: Vector, node: &'a Node) -> Self {
        let bounds = node.bounds();

        Self {
            position: Point::new(bounds.x, bounds.y) + offset,
            node,
        }
    }

    /// Returns the position of the [`Layout`].
    pub fn position(&self) -> Point {
        self.position
    }

    /// Returns the bounds of the [`Layout`].
    ///
    /// The returned [`Rectangle`] describes the position and size of a
    /// [`Node`].
    pub fn bounds(&self) -> Rectangle {
        let bounds = self.node.bounds();

        Rectangle {
            x: self.position.x,
            y: self.position.y,
            width: bounds.width,
            height: bounds.height,
        }
    }

    /// Returns an iterator over the [`Layout`] of the children of a [`Node`].
    pub fn children(self) -> impl Iterator<Item = Layout<'a>> {
        self.node.children().iter().map(move |node| {
            Layout::with_offset(
                Vector::new(self.position.x, self.position.y),
                node,
            )
        })
    }
}

/// Produces a [`Node`] with two children nodes one right next to each other.
pub fn next_to_each_other(
    limits: &Limits,
    spacing: f32,
    left: impl FnOnce(&Limits) -> Node,
    right: impl FnOnce(&Limits) -> Node,
) -> Node {
    let mut left_node = left(limits);
    let left_size = left_node.size();

    let right_limits = limits.shrink(Size::new(left_size.width + spacing, 0.0));

    let mut right_node = right(&right_limits);
    let right_size = right_node.size();

    let (left_y, right_y) = if left_size.height > right_size.height {
        (0.0, (left_size.height - right_size.height) / 2.0)
    } else {
        ((right_size.height - left_size.height) / 2.0, 0.0)
    };

    left_node.move_to(Point::new(0.0, left_y));
    right_node.move_to(Point::new(left_size.width + spacing, right_y));

    Node::with_children(
        Size::new(
            left_size.width + spacing + right_size.width,
            left_size.height.max(right_size.height),
        ),
        vec![left_node, right_node],
    )
}
