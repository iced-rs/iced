use crate::{Align, Point, Rectangle, Size};

/// The bounds of an element and its children.
#[derive(Debug, Clone, Default)]
pub struct Node {
    bounds: Rectangle,
    children: Vec<Node>,
}

impl Node {
    /// Creates a new [`Node`] with the given [`Size`].
    pub const fn new(size: Size) -> Self {
        Self::with_children(size, Vec::new())
    }

    /// Creates a new [`Node`] with the given [`Size`] and children.
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
    pub fn size(&self) -> Size {
        Size::new(self.bounds.width, self.bounds.height)
    }

    /// Returns the bounds of the [`Node`].
    pub fn bounds(&self) -> Rectangle {
        self.bounds
    }

    /// Returns the children of the [`Node`].
    pub fn children(&self) -> &[Node] {
        &self.children
    }

    /// Aligns the [`Node`] in the given space.
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
            Align::Fill => {
                self.bounds.width = space.width;
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
            Align::Fill => {
                self.bounds.height = space.height;
            }
        }
    }

    /// Moves the [`Node`] to the given position.
    pub fn move_to(&mut self, position: Point) {
        self.bounds.x = position.x;
        self.bounds.y = position.y;
    }
}
