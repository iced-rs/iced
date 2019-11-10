use crate::{Rectangle, Size};

#[derive(Debug, Clone, Default)]
pub struct Node {
    pub bounds: Rectangle,
    children: Vec<Node>,
}

impl Node {
    pub fn new(size: Size) -> Self {
        Self::with_children(size, Vec::new())
    }

    pub fn with_children(size: Size, children: Vec<Node>) -> Self {
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

    pub fn size(&self) -> Size {
        Size::new(self.bounds.width, self.bounds.height)
    }

    pub fn bounds(&self) -> Rectangle {
        self.bounds
    }

    pub fn children(&self) -> &[Node] {
        &self.children
    }
}
