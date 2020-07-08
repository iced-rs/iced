use crate::{Element, Length, Widget, Hasher, layout};
use std::hash::Hash;

/// An amount of empty space.
///
/// It can be useful if you want to fill some space with nothing.
#[derive(Debug)]
pub struct Space {
    width: Length,
    height: Length,
}

impl Space {
    /// Creates an amount of empty [`Space`] with the given width and height.
    ///
    /// [`Space`]: struct.Space.html
    pub fn new(width: Length, height: Length) -> Self {
        Space { width, height }
    }

    /// Creates an amount of horizontal [`Space`].
    ///
    /// [`Space`]: struct.Space.html
    pub fn with_width(width: Length) -> Self {
        Space {
            width,
            height: Length::Shrink,
        }
    }

    /// Creates an amount of vertical [`Space`].
    ///
    /// [`Space`]: struct.Space.html
    pub fn with_height(height: Length) -> Self {
        Space {
            width: Length::Shrink,
            height,
        }
    }
}

impl<'a, Message> Widget<Message> for Space {
    fn hash_layout(&self, state: &mut Hasher) {
        std::any::TypeId::of::<Space>().hash(state);

        self.width.hash(state);
        self.height.hash(state);
    }

    fn layout(
        &self,
        limits: &layout::Limits,
    ) -> layout::Node {
        todo!();
    }

    fn width(&self) -> Length {
        todo!();
    }

    fn height(&self) -> Length {
        todo!();
    }
}

impl<'a, Message> From<Space> for Element<'a, Message> {
    fn from(space: Space) -> Element<'a, Message> {
        Element::new(space)
    }
}
