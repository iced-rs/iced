use crate::{Alignment, Padding, Point, Rectangle, Size, Vector};

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

    /// Creates a new [`Node`] that wraps a single child with some [`Padding`].
    pub fn container(child: Self, padding: Padding) -> Self {
        Self::with_children(
            child.bounds.size().expand(padding),
            vec![child.move_to(Point::new(padding.left, padding.top))],
        )
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
        mut self,
        horizontal_alignment: Alignment,
        vertical_alignment: Alignment,
        space: Size,
    ) -> Self {
        self.align_mut(horizontal_alignment, vertical_alignment, space);
        self
    }

    /// Mutable reference version of [`Self::align`].
    pub fn align_mut(
        &mut self,
        horizontal_alignment: Alignment,
        vertical_alignment: Alignment,
        space: Size,
    ) {
        match horizontal_alignment {
            Alignment::Start => {}
            Alignment::Center => {
                self.bounds.x += (space.width - self.bounds.width) / 2.0;
            }
            Alignment::End => {
                self.bounds.x += space.width - self.bounds.width;
            }
        }

        match vertical_alignment {
            Alignment::Start => {}
            Alignment::Center => {
                self.bounds.y += (space.height - self.bounds.height) / 2.0;
            }
            Alignment::End => {
                self.bounds.y += space.height - self.bounds.height;
            }
        }
    }

    /// Moves the [`Node`] to the given position.
    pub fn move_to(mut self, position: impl Into<Point>) -> Self {
        self.move_to_mut(position);
        self
    }

    /// Mutable reference version of [`Self::move_to`].
    pub fn move_to_mut(&mut self, position: impl Into<Point>) {
        let position = position.into();

        self.bounds.x = position.x;
        self.bounds.y = position.y;
    }

    /// Translates the [`Node`] by the given translation.
    pub fn translate(mut self, translation: impl Into<Vector>) -> Self {
        self.translate_mut(translation);
        self
    }

    /// Translates the [`Node`] by the given translation.
    pub fn translate_mut(&mut self, translation: impl Into<Vector>) {
        self.bounds = self.bounds + translation.into();
    }
}
