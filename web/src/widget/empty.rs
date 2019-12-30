use crate::{style, Bus, Element, Length, Widget};
use dodrio::bumpalo;

/// An amount of empty space.
///
/// It can be useful if you want to fill some space with nothing.
///
/// [`Empty`]: struct.Empty.html
#[derive(Debug)]
pub struct Empty {
    width: Length,
    height: Length,
}

impl Empty {
    /// Creates an amount of [`Empty`] space.
    ///
    /// [`Empty`]: struct.Empty.html
    pub fn new() -> Self {
        Empty {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    /// Sets the width of the [`Empty`] space.
    ///
    /// [`Empty`]: struct..html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Empty`] space.
    ///
    /// [`Empty`]: struct.Column.html
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }
}

impl<'a, Message> Widget<Message> for Empty {
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        _publish: &Bus<Message>,
        _style_sheet: &mut style::Sheet<'b>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::*;

        let width = style::length(self.width);
        let height = style::length(self.height);

        let style = bumpalo::format!(
            in bump,
            "width: {}; height: {};",
            width,
            height
        );

        div(bump).attr("style", style.into_bump_str()).finish()
    }
}

impl<'a, Message> From<Empty> for Element<'a, Message> {
    fn from(empty: Empty) -> Element<'a, Message> {
        Element::new(empty)
    }
}
