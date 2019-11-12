use crate::{Align, Rectangle, Size};

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
}
