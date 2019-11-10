use crate::Rectangle;

mod limits;

pub use limits::Limits;

/// The computed bounds of a [`Node`] and its children.
///
/// This type is provided by the GUI runtime to [`Widget::on_event`] and
/// [`Widget::draw`], describing the layout of the [`Node`] produced by
/// [`Widget::node`].
///
/// [`Node`]: struct.Node.html
/// [`Widget::on_event`]: widget/trait.Widget.html#method.on_event
/// [`Widget::draw`]: widget/trait.Widget.html#tymethod.draw
/// [`Widget::node`]: widget/trait.Widget.html#tymethod.node
#[derive(Debug, Clone)]
pub struct Layout {
    bounds: Rectangle,
    children: Vec<Layout>,
}

impl Layout {
    pub fn new(bounds: Rectangle) -> Self {
        Layout {
            bounds,
            children: Vec::new(),
        }
    }

    pub fn push(&mut self, mut child: Layout) {
        child.bounds.x += self.bounds.x;
        child.bounds.y += self.bounds.y;

        self.children.push(child);
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
        self.bounds
    }

    /// Returns an iterator over the [`Layout`] of the children of a [`Node`].
    ///
    /// [`Layout`]: struct.Layout.html
    /// [`Node`]: struct.Node.html
    pub fn children(&self) -> impl Iterator<Item = &Layout> {
        self.children.iter()
    }
}
