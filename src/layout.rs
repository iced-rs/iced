use stretch::result;

use crate::{Point, Rectangle, Vector};

/// The computed bounds of a [`Node`] and its children.
///
/// This type is provided by the GUI runtime to [`Widget::on_event`] and
/// [`Widget::draw`], describing the layout of the [`Node`] produced by
/// [`Widget::node`].
///
/// [`Node`]: struct.Node.html
/// [`Widget::on_event`]: trait.Widget.html#method.on_event
/// [`Widget::draw`]: trait.Widget.html#tymethod.draw
/// [`Widget::node`]: trait.Widget.html#tymethod.node
#[derive(Debug)]
pub struct Layout<'a> {
    layout: &'a result::Layout,
    position: Point,
}

impl<'a> Layout<'a> {
    pub(crate) fn new(layout: &'a result::Layout) -> Self {
        Self::with_parent_position(layout, Point::new(0.0, 0.0))
    }

    fn with_parent_position(
        layout: &'a result::Layout,
        parent_position: Point,
    ) -> Self {
        let position =
            parent_position + Vector::new(layout.location.x, layout.location.y);

        Layout { layout, position }
    }

    /// Gets the bounds of the [`Layout`].
    ///
    /// The returned [`Rectangle`] describes the position and size of a
    /// [`Node`].
    ///
    /// [`Layout`]: struct.Layout.html
    /// [`Rectangle`]: ../../graphics/struct.Rectangle.html
    /// [`Node`]: struct.Node.html
    pub fn bounds(&self) -> Rectangle<f32> {
        Rectangle {
            x: self.position.x,
            y: self.position.y,
            width: self.layout.size.width,
            height: self.layout.size.height,
        }
    }

    /// Returns an iterator over the [`Layout`] of the children of a [`Node`].
    ///
    /// [`Layout`]: struct.Layout.html
    /// [`Node`]: struct.Node.html
    pub fn children(&'a self) -> impl Iterator<Item = Layout<'a>> {
        self.layout.children.iter().map(move |layout| {
            Layout::with_parent_position(layout, self.position)
        })
    }
}
