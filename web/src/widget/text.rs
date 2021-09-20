use crate::alignment;
use crate::css;
use crate::{Bus, Color, Css, Element, Font, Length, Widget};
use dodrio::bumpalo;

/// A paragraph of text.
///
/// # Example
///
/// ```
/// # use iced_web::Text;
///
/// Text::new("I <3 iced!")
///     .size(40);
/// ```
#[derive(Debug, Clone)]
pub struct Text {
    content: String,
    size: Option<u16>,
    color: Option<Color>,
    font: Font,
    width: Length,
    height: Length,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
}

impl Text {
    /// Create a new fragment of [`Text`] with the given contents.
    pub fn new<T: Into<String>>(label: T) -> Self {
        Text {
            content: label.into(),
            size: None,
            color: None,
            font: Font::Default,
            width: Length::Shrink,
            height: Length::Shrink,
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
        }
    }

    /// Sets the size of the [`Text`].
    pub fn size(mut self, size: u16) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets the [`Color`] of the [`Text`].
    pub fn color<C: Into<Color>>(mut self, color: C) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Sets the [`Font`] of the [`Text`].
    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }

    /// Sets the width of the [`Text`] boundaries.
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Text`] boundaries.
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the [`HorizontalAlignment`] of the [`Text`].
    pub fn horizontal_alignment(
        mut self,
        alignment: alignment::Horizontal,
    ) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    /// Sets the [`VerticalAlignment`] of the [`Text`].
    pub fn vertical_alignment(
        mut self,
        alignment: alignment::Vertical,
    ) -> Self {
        self.vertical_alignment = alignment;
        self
    }
}

impl<'a, Message> Widget<Message> for Text {
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        _publish: &Bus<Message>,
        _style_sheet: &mut Css<'b>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::*;

        let content = {
            use dodrio::bumpalo::collections::String;

            String::from_str_in(&self.content, bump)
        };

        let color = self
            .color
            .map(css::color)
            .unwrap_or(String::from("inherit"));

        let width = css::length(self.width);
        let height = css::length(self.height);

        let text_align = match self.horizontal_alignment {
            alignment::Horizontal::Left => "left",
            alignment::Horizontal::Center => "center",
            alignment::Horizontal::Right => "right",
        };

        let style = bumpalo::format!(
            in bump,
            "width: {}; height: {}; font-size: {}px; color: {}; \
            text-align: {}; font-family: {}",
            width,
            height,
            self.size.unwrap_or(20),
            color,
            text_align,
            match self.font {
                Font::Default => "inherit",
                Font::External { name, .. } => name,
            }
        );

        // TODO: Complete styling
        p(bump)
            .attr("style", style.into_bump_str())
            .children(vec![text(content.into_bump_str())])
            .finish()
    }
}

impl<'a, Message> From<Text> for Element<'a, Message> {
    fn from(text: Text) -> Element<'a, Message> {
        Element::new(text)
    }
}
