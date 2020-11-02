//! Position your widgets properly.
mod debugger;
mod limits;
mod node;

pub mod flex;

pub use debugger::Debugger;
pub use limits::Limits;
pub use node::Node;

use crate::{Point, Rectangle, Vector};

/// The bounds of a [`Node`] and its children, using absolute coordinates.
///
/// [`Node`]: struct.Node.html
#[derive(Debug, Clone, Copy)]
pub struct Layout<'a> {
    position: Point,
    node: &'a Node,
}

impl<'a> Layout<'a> {
    pub(crate) fn new(node: &'a Node) -> Self {
        Self::with_offset(Vector::new(0.0, 0.0), node)
    }

    pub(crate) fn with_offset(offset: Vector, node: &'a Node) -> Self {
        let bounds = node.bounds();

        Self {
            position: Point::new(bounds.x, bounds.y) + offset,
            node,
        }
    }

    /// Returns the position of the [`Layout`].
    ///
    /// [`Layout`]: struct.Layout.html
    pub fn position(&self) -> Point {
        self.position
    }

    /// Returns the bounds of the [`Layout`].
    ///
    /// The returned [`Rectangle`] describes the position and size of a
    /// [`Node`].
    ///
    /// [`Layout`]: struct.Layout.html
    /// [`Rectangle`]: struct.Rectangle.html
    /// [`Node`]: struct.Node.html
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
    ///
    /// [`Layout`]: struct.Layout.html
    /// [`Node`]: struct.Node.html
    pub fn children(self) -> impl Iterator<Item = Layout<'a>> {
        self.node.children().iter().map(move |node| {
            Layout::with_offset(
                Vector::new(self.position.x, self.position.y),
                node,
            )
        })
    }
}
