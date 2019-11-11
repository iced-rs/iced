use crate::{Align, Length};

use std::u32;

/// A container that distributes its contents horizontally.
///
/// A [`Row`] will try to fill the horizontal space of its container.
///
/// [`Row`]: struct.Row.html
pub struct Row<Element> {
    pub spacing: u16,
    pub padding: u16,
    pub width: Length,
    pub height: Length,
    pub max_width: u32,
    pub max_height: u32,
    pub align_items: Align,
    pub children: Vec<Element>,
}

impl<Element> Row<Element> {
    /// Creates an empty [`Row`].
    ///
    /// [`Row`]: struct.Row.html
    pub fn new() -> Self {
        Row {
            spacing: 0,
            padding: 0,
            width: Length::Fill,
            height: Length::Shrink,
            max_width: u32::MAX,
            max_height: u32::MAX,
            align_items: Align::Start,
            children: Vec::new(),
        }
    }

    /// Sets the horizontal spacing _between_ elements.
    ///
    /// Custom margins per element do not exist in Iced. You should use this
    /// method instead! While less flexible, it helps you keep spacing between
    /// elements consistent.
    pub fn spacing(mut self, units: u16) -> Self {
        self.spacing = units;
        self
    }

    /// Sets the padding of the [`Row`].
    ///
    /// [`Row`]: struct.Row.html
    pub fn padding(mut self, units: u16) -> Self {
        self.padding = units;
        self
    }

    /// Sets the width of the [`Row`].
    ///
    /// [`Row`]: struct.Row.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Row`].
    ///
    /// [`Row`]: struct.Row.html
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the maximum width of the [`Row`].
    ///
    /// [`Row`]: struct.Row.html
    pub fn max_width(mut self, max_width: u32) -> Self {
        self.max_width = max_width;
        self
    }

    /// Sets the maximum height of the [`Row`].
    ///
    /// [`Row`]: struct.Row.html
    pub fn max_height(mut self, max_height: u32) -> Self {
        self.max_height = max_height;
        self
    }

    /// Sets the vertical alignment of the contents of the [`Row`] .
    ///
    /// [`Row`]: struct.Row.html
    pub fn align_items(mut self, align: Align) -> Self {
        self.align_items = align;
        self
    }

    /// Adds an [`Element`] to the [`Row`].
    ///
    /// [`Element`]: ../struct.Element.html
    /// [`Row`]: struct.Row.html
    pub fn push<E>(mut self, child: E) -> Row<Element>
    where
        E: Into<Element>,
    {
        self.children.push(child.into());
        self
    }
}

impl<Element> std::fmt::Debug for Row<Element>
where
    Element: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Complete once stabilized
        f.debug_struct("Row")
            .field("spacing", &self.spacing)
            .field("children", &self.children)
            .finish()
    }
}
