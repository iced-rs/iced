use crate::{css, Bus, Css, Element, Length, Widget};
use dodrio::bumpalo;

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
    pub fn new(width: Length, height: Length) -> Self {
        Space { width, height }
    }

    /// Creates an amount of horizontal [`Space`].
    pub fn with_width(width: Length) -> Self {
        Space {
            width,
            height: Length::Shrink,
        }
    }

    /// Creates an amount of vertical [`Space`].
    pub fn with_height(height: Length) -> Self {
        Space {
            width: Length::Shrink,
            height,
        }
    }
}

impl<'a, Message> Widget<Message> for Space {
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        _publish: &Bus<Message>,
        _css: &mut Css<'b>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::*;

        let width = css::length(self.width);
        let height = css::length(self.height);

        let style = bumpalo::format!(
            in bump,
            "width: {}; height: {};",
            width,
            height
        );

        div(bump).attr("style", style.into_bump_str()).finish()
    }
}

impl<'a, Message> From<Space> for Element<'a, Message> {
    fn from(space: Space) -> Element<'a, Message> {
        Element::new(space)
    }
}
