//! Navigate an endless amount of content with a scrollbar.
use crate::bumpalo;
use crate::css;
use crate::{Alignment, Bus, Column, Css, Element, Length, Padding, Widget};

pub use iced_style::scrollable::{Scrollbar, Scroller, StyleSheet};

/// A widget that can vertically display an infinite amount of content with a
/// scrollbar.
#[allow(missing_debug_implementations)]
pub struct Scrollable<'a, Message> {
    width: Length,
    height: Length,
    max_height: u32,
    content: Column<'a, Message>,
    #[allow(dead_code)]
    style_sheet: Box<dyn StyleSheet + 'a>,
}

impl<'a, Message> Scrollable<'a, Message> {
    /// Creates a new [`Scrollable`] with the given [`State`].
    pub fn new(_state: &'a mut State) -> Self {
        use std::u32;

        Scrollable {
            width: Length::Fill,
            height: Length::Shrink,
            max_height: u32::MAX,
            content: Column::new(),
            style_sheet: Default::default(),
        }
    }

    /// Sets the vertical spacing _between_ elements.
    ///
    /// Custom margins per element do not exist in Iced. You should use this
    /// method instead! While less flexible, it helps you keep spacing between
    /// elements consistent.
    pub fn spacing(mut self, units: u16) -> Self {
        self.content = self.content.spacing(units);
        self
    }

    /// Sets the [`Padding`] of the [`Scrollable`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.content = self.content.padding(padding);
        self
    }

    /// Sets the width of the [`Scrollable`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Scrollable`].
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the maximum width of the [`Scrollable`].
    pub fn max_width(mut self, max_width: u32) -> Self {
        self.content = self.content.max_width(max_width);
        self
    }

    /// Sets the maximum height of the [`Scrollable`] in pixels.
    pub fn max_height(mut self, max_height: u32) -> Self {
        self.max_height = max_height;
        self
    }

    /// Sets the horizontal alignment of the contents of the [`Scrollable`] .
    pub fn align_items(mut self, align_items: Alignment) -> Self {
        self.content = self.content.align_items(align_items);
        self
    }

    /// Sets the style of the [`Scrollable`] .
    pub fn style(
        mut self,
        style_sheet: impl Into<Box<dyn StyleSheet + 'a>>,
    ) -> Self {
        self.style_sheet = style_sheet.into();
        self
    }

    /// Adds an element to the [`Scrollable`].
    pub fn push<E>(mut self, child: E) -> Self
    where
        E: Into<Element<'a, Message>>,
    {
        self.content = self.content.push(child);
        self
    }
}

impl<'a, Message> Widget<Message> for Scrollable<'a, Message>
where
    Message: 'static,
{
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
        style_sheet: &mut Css<'b>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::*;

        let width = css::length(self.width);
        let height = css::length(self.height);

        // TODO: Scrollbar styling

        let node = div(bump)
            .attr(
                "style",
                bumpalo::format!(
                    in bump,
                    "width: {}; height: {}; max-height: {}px; overflow: auto",
                    width,
                    height,
                    self.max_height
                )
                .into_bump_str(),
            )
            .children(vec![self.content.node(bump, bus, style_sheet)]);

        node.finish()
    }
}

impl<'a, Message> From<Scrollable<'a, Message>> for Element<'a, Message>
where
    Message: 'static,
{
    fn from(scrollable: Scrollable<'a, Message>) -> Element<'a, Message> {
        Element::new(scrollable)
    }
}

/// The local state of a [`Scrollable`].
#[derive(Debug, Clone, Copy, Default)]
pub struct State;

impl State {
    /// Creates a new [`State`] with the scrollbar located at the top.
    pub fn new() -> Self {
        State::default()
    }
}
