//! Decorate content and apply alignment.
use crate::alignment::{self, Alignment};
use crate::bumpalo;
use crate::css;
use crate::{Bus, Css, Element, Length, Padding, Widget};

pub use iced_style::container::{Style, StyleSheet};

/// An element decorating some content.
///
/// It is normally used for alignment purposes.
#[allow(missing_debug_implementations)]
pub struct Container<'a, Message> {
    padding: Padding,
    width: Length,
    height: Length,
    max_width: u32,
    #[allow(dead_code)]
    max_height: u32,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    style_sheet: Box<dyn StyleSheet + 'a>,
    content: Element<'a, Message>,
}

impl<'a, Message> Container<'a, Message> {
    /// Creates an empty [`Container`].
    pub fn new<T>(content: T) -> Self
    where
        T: Into<Element<'a, Message>>,
    {
        use std::u32;

        Container {
            padding: Padding::ZERO,
            width: Length::Shrink,
            height: Length::Shrink,
            max_width: u32::MAX,
            max_height: u32::MAX,
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
            style_sheet: Default::default(),
            content: content.into(),
        }
    }

    /// Sets the [`Padding`] of the [`Container`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the width of the [`Container`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Container`].
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the maximum width of the [`Container`].
    pub fn max_width(mut self, max_width: u32) -> Self {
        self.max_width = max_width;
        self
    }

    /// Sets the maximum height of the [`Container`] in pixels.
    pub fn max_height(mut self, max_height: u32) -> Self {
        self.max_height = max_height;
        self
    }

    /// Centers the contents in the horizontal axis of the [`Container`].
    pub fn center_x(mut self) -> Self {
        self.horizontal_alignment = alignment::Horizontal::Center;

        self
    }

    /// Centers the contents in the vertical axis of the [`Container`].
    pub fn center_y(mut self) -> Self {
        self.vertical_alignment = alignment::Vertical::Center;

        self
    }

    /// Sets the style of the [`Container`].
    pub fn style(mut self, style: impl Into<Box<dyn StyleSheet + 'a>>) -> Self {
        self.style_sheet = style.into();
        self
    }
}

impl<'a, Message> Widget<Message> for Container<'a, Message>
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

        let column_class = style_sheet.insert(bump, css::Rule::Column);

        let style = self.style_sheet.style();

        let node = div(bump)
            .attr(
                "class",
                bumpalo::format!(in bump, "{}", column_class).into_bump_str(),
            )
            .attr(
                "style",
                bumpalo::format!(
                    in bump,
                    "width: {}; height: {}; max-width: {}; padding: {}; align-items: {}; justify-content: {}; background: {}; color: {}; border-width: {}px; border-color: {}; border-radius: {}px",
                    css::length(self.width),
                    css::length(self.height),
                    css::max_length(self.max_width),
                    css::padding(self.padding),
                    css::alignment(Alignment::from(self.horizontal_alignment)),
                    css::alignment(Alignment::from(self.vertical_alignment)),
                    style.background.map(css::background).unwrap_or(String::from("initial")),
                    style.text_color.map(css::color).unwrap_or(String::from("inherit")),
                    style.border_width,
                    css::color(style.border_color),
                    style.border_radius
                )
                .into_bump_str(),
            )
            .children(vec![self.content.node(bump, bus, style_sheet)]);

        // TODO: Complete styling

        node.finish()
    }
}

impl<'a, Message> From<Container<'a, Message>> for Element<'a, Message>
where
    Message: 'static,
{
    fn from(container: Container<'a, Message>) -> Element<'a, Message> {
        Element::new(container)
    }
}
