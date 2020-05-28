
use crate::{Align, Point, Rectangle, Size, Vector};

/// The bounds of an element and its children.
#[derive(Debug, Clone, Default)]
pub struct Node {
    bounds: Rectangle,
    children: Vec<Node>,
}

impl Node {
    /// Creates a new [`Node`] with the given [`Size`].
    ///
    /// [`Node`]: struct.Node.html
    /// [`Size`]: ../struct.Size.html
    pub const fn new(size: Size) -> Self {
        Self::with_children(size, Vec::new())
    }

    /// Creates a new [`Node`] with the given [`Size`] and children.
    ///
    /// [`Node`]: struct.Node.html
    /// [`Size`]: ../struct.Size.html
    pub const fn with_children(size: Size, children: Vec<Node>) -> Self {
        Node {
            bounds: Rectangle {
                x: 0.0,
                y: 0.0,
                width: size.width,
                height: size.height,
            },
            children,
        }
    }

    /// Returns the [`Size`] of the [`Node`].
    ///
    /// [`Node`]: struct.Node.html
    /// [`Size`]: ../struct.Size.html
    pub fn size(&self) -> Size {
        Size::new(self.bounds.width, self.bounds.height)
    }

    /// Returns the bounds of the [`Node`].
    ///
    /// [`Node`]: struct.Node.html
    pub fn bounds(&self) -> Rectangle {
        self.bounds
    }

    /// Returns the children of the [`Node`].
    ///
    /// [`Node`]: struct.Node.html
    pub fn children(&self) -> &[Node] {
        &self.children
    }

    /// Aligns the [`Node`] in the given space.
    ///
    /// [`Node`]: struct.Node.html
    pub fn align(
        &mut self,
        horizontal_alignment: Align,
        vertical_alignment: Align,
        space: Size,
    ) {
        match horizontal_alignment {
            Align::Start => {}
            Align::Center => {
                self.bounds.x += (space.width - self.bounds.width) / 2.0;
            }
            Align::End => {
                self.bounds.x += space.width - self.bounds.width;
            }
        }

        match vertical_alignment {
            Align::Start => {}
            Align::Center => {
                self.bounds.y += (space.height - self.bounds.height) / 2.0;
            }
            Align::End => {
                self.bounds.y += space.height - self.bounds.height;
            }
        }
    }

    /// Moves the [`Node`] to the given position.
    ///
    /// [`Node`]: struct.Node.html
    pub fn move_to(&mut self, position: Point) {
        self.bounds.x = position.x;
        self.bounds.y = position.y;
    }
}

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

    /// Gets the bounds of the [`Layout`].
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
    pub fn children(&'a self) -> impl Iterator<Item = Layout<'a>> {
        self.node.children().iter().map(move |node| {
            Layout::with_offset(
                Vector::new(self.position.x, self.position.y),
                node,
            )
        })
    }
}
