use crate::css;
use crate::{Alignment, Bus, Css, Element, Length, Padding, Widget};

use dodrio::bumpalo;
use std::u32;

/// A container that distributes its contents vertically.
///
/// A [`Column`] will try to fill the horizontal space of its container.
#[allow(missing_debug_implementations)]
pub struct Column<'a, Message> {
    spacing: u16,
    padding: Padding,
    width: Length,
    height: Length,
    max_width: u32,
    max_height: u32,
    align_items: Alignment,
    children: Vec<Element<'a, Message>>,
}

impl<'a, Message> Column<'a, Message> {
    /// Creates an empty [`Column`].
    pub fn new() -> Self {
        Self::with_children(Vec::new())
    }

    /// Creates a [`Column`] with the given elements.
    pub fn with_children(children: Vec<Element<'a, Message>>) -> Self {
        Column {
            spacing: 0,
            padding: Padding::ZERO,
            width: Length::Fill,
            height: Length::Shrink,
            max_width: u32::MAX,
            max_height: u32::MAX,
            align_items: Alignment::Start,
            children,
        }
    }

    /// Sets the vertical spacing _between_ elements.
    ///
    /// Custom margins per element do not exist in Iced. You should use this
    /// method instead! While less flexible, it helps you keep spacing between
    /// elements consistent.
    pub fn spacing(mut self, units: u16) -> Self {
        self.spacing = units;
        self
    }

    /// Sets the [`Padding`] of the [`Column`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the width of the [`Column`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Column`].
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the maximum width of the [`Column`].
    pub fn max_width(mut self, max_width: u32) -> Self {
        self.max_width = max_width;
        self
    }

    /// Sets the maximum height of the [`Column`] in pixels.
    pub fn max_height(mut self, max_height: u32) -> Self {
        self.max_height = max_height;
        self
    }

    /// Sets the horizontal alignment of the contents of the [`Column`] .
    pub fn align_items(mut self, align: Alignment) -> Self {
        self.align_items = align;
        self
    }

    /// Adds an element to the [`Column`].
    pub fn push<E>(mut self, child: E) -> Self
    where
        E: Into<Element<'a, Message>>,
    {
        self.children.push(child.into());
        self
    }
}

impl<'a, Message> Widget<Message> for Column<'a, Message> {
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        publish: &Bus<Message>,
        style_sheet: &mut Css<'b>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::*;

        let children: Vec<_> = self
            .children
            .iter()
            .map(|element| element.widget.node(bump, publish, style_sheet))
            .collect();

        let column_class = style_sheet.insert(bump, css::Rule::Column);

        let spacing_class =
            style_sheet.insert(bump, css::Rule::Spacing(self.spacing));

        // TODO: Complete styling
        div(bump)
            .attr(
                "class",
                bumpalo::format!(in bump, "{} {}", column_class, spacing_class)
                    .into_bump_str(),
            )
            .attr("style", bumpalo::format!(
                    in bump,
                    "width: {}; height: {}; max-width: {}; max-height: {}; padding: {}; align-items: {}",
                    css::length(self.width),
                    css::length(self.height),
                    css::max_length(self.max_width),
                    css::max_length(self.max_height),
                    css::padding(self.padding),
                    css::alignment(self.align_items)
                ).into_bump_str()
            )
            .children(children)
            .finish()
    }
}

impl<'a, Message> From<Column<'a, Message>> for Element<'a, Message>
where
    Message: 'static,
{
    fn from(column: Column<'a, Message>) -> Element<'a, Message> {
        Element::new(column)
    }
}
