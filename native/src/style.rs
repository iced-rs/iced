use crate::{Align, Justify, Length};

use stretch::style;

/// The appearance of a [`Node`].
///
/// [`Node`]: struct.Node.html
#[derive(Debug, Clone, Copy)]
pub struct Style(pub(crate) style::Style);

impl Default for Style {
    fn default() -> Style {
        Style::new()
    }
}

impl Style {
    /// Creates a new [`Style`].
    ///
    /// [`Style`]: struct.Style.html
    pub fn new() -> Self {
        Style(style::Style {
            align_items: style::AlignItems::FlexStart,
            justify_content: style::JustifyContent::FlexStart,
            ..style::Style::default()
        })
    }

    /// Defines the width of a [`Node`].
    ///
    /// [`Node`]: struct.Node.html
    pub fn width(mut self, width: Length) -> Self {
        self.0.size.width = into_dimension(width);
        self
    }

    /// Defines the height of a [`Node`].
    ///
    /// [`Node`]: struct.Node.html
    pub fn height(mut self, height: Length) -> Self {
        self.0.size.height = into_dimension(height);
        self
    }

    /// Defines the minimum width of a [`Node`].
    ///
    /// [`Node`]: struct.Node.html
    pub fn min_width(mut self, min_width: Length) -> Self {
        self.0.min_size.width = into_dimension(min_width);
        self
    }

    /// Defines the maximum width of a [`Node`].
    ///
    /// [`Node`]: struct.Node.html
    pub fn max_width(mut self, max_width: Length) -> Self {
        self.0.max_size.width = into_dimension(max_width);
        self
    }

    /// Defines the minimum height of a [`Node`].
    ///
    /// [`Node`]: struct.Node.html
    pub fn min_height(mut self, min_height: Length) -> Self {
        self.0.min_size.height = into_dimension(min_height);
        self
    }

    /// Defines the maximum height of a [`Node`].
    ///
    /// [`Node`]: struct.Node.html
    pub fn max_height(mut self, max_height: Length) -> Self {
        self.0.max_size.height = into_dimension(max_height);
        self
    }

    pub fn align_items(mut self, align: Align) -> Self {
        self.0.align_items = into_align_items(align);
        self
    }

    pub fn justify_content(mut self, justify: Justify) -> Self {
        self.0.justify_content = into_justify_content(justify);
        self
    }

    /// Sets the alignment of a [`Node`].
    ///
    /// If the [`Node`] is inside a...
    ///
    ///   * [`Column`], this setting will affect its __horizontal__ alignment.
    ///   * [`Row`], this setting will affect its __vertical__ alignment.
    ///
    /// [`Node`]: struct.Node.html
    /// [`Column`]: widget/struct.Column.html
    /// [`Row`]: widget/struct.Row.html
    pub fn align_self(mut self, align: Option<Align>) -> Self {
        self.0.align_self = match align {
            Some(align) => into_align_self(align),
            None => stretch::style::AlignSelf::Auto,
        };

        self
    }

    /// Sets the padding of a [`Node`].
    ///
    /// [`Node`]: struct.Node.html
    pub fn padding(mut self, units: u16) -> Self {
        self.0.padding = stretch::geometry::Rect {
            start: style::Dimension::Points(units as f32),
            end: style::Dimension::Points(units as f32),
            top: style::Dimension::Points(units as f32),
            bottom: style::Dimension::Points(units as f32),
        };

        self
    }
}

fn into_dimension(length: Length) -> style::Dimension {
    match length {
        Length::Shrink => style::Dimension::Undefined,
        Length::Fill => style::Dimension::Percent(1.0),
        Length::Units(units) => style::Dimension::Points(units as f32),
    }
}

fn into_align_items(align: Align) -> style::AlignItems {
    match align {
        Align::Start => style::AlignItems::FlexStart,
        Align::Center => style::AlignItems::Center,
        Align::End => style::AlignItems::FlexEnd,
        Align::Stretch => style::AlignItems::Stretch,
    }
}

fn into_align_self(align: Align) -> style::AlignSelf {
    match align {
        Align::Start => style::AlignSelf::FlexStart,
        Align::Center => style::AlignSelf::Center,
        Align::End => style::AlignSelf::FlexEnd,
        Align::Stretch => style::AlignSelf::Stretch,
    }
}

fn into_justify_content(justify: Justify) -> style::JustifyContent {
    match justify {
        Justify::Start => style::JustifyContent::FlexStart,
        Justify::Center => style::JustifyContent::Center,
        Justify::End => style::JustifyContent::FlexEnd,
        Justify::SpaceBetween => style::JustifyContent::SpaceBetween,
        Justify::SpaceAround => style::JustifyContent::SpaceAround,
        Justify::SpaceEvenly => style::JustifyContent::SpaceEvenly,
    }
}
