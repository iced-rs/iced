use crate::{Align, Length};

use std::u32;

#[derive(Debug)]
pub struct Container<Element> {
    pub width: Length,
    pub height: Length,
    pub max_width: u32,
    pub max_height: u32,
    pub horizontal_alignment: Align,
    pub vertical_alignment: Align,
    pub content: Element,
}

impl<Element> Container<Element> {
    /// Creates an empty [`Container`].
    ///
    /// [`Container`]: struct.Container.html
    pub fn new<T>(content: T) -> Self
    where
        T: Into<Element>,
    {
        Container {
            width: Length::Shrink,
            height: Length::Shrink,
            max_width: u32::MAX,
            max_height: u32::MAX,
            horizontal_alignment: Align::Start,
            vertical_alignment: Align::Start,
            content: content.into(),
        }
    }

    /// Sets the width of the [`Container`].
    ///
    /// [`Container`]: struct.Container.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Container`].
    ///
    /// [`Container`]: struct.Container.html
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the maximum width of the [`Container`].
    ///
    /// [`Container`]: struct.Container.html
    pub fn max_width(mut self, max_width: u32) -> Self {
        self.max_width = max_width;
        self
    }

    /// Sets the maximum height of the [`Container`] in pixels.
    ///
    /// [`Container`]: struct.Container.html
    pub fn max_height(mut self, max_height: u32) -> Self {
        self.max_height = max_height;
        self
    }

    /// Centers the contents in the horizontal axis of the [`Container`].
    ///
    /// [`Container`]: struct.Container.html
    pub fn center_x(mut self) -> Self {
        self.horizontal_alignment = Align::Center;

        self
    }

    /// Centers the contents in the vertical axis of the [`Container`].
    ///
    /// [`Container`]: struct.Container.html
    pub fn center_y(mut self) -> Self {
        self.vertical_alignment = Align::Center;

        self
    }
}
