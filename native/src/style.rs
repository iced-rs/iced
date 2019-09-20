use crate::{Align, Justify, Length};

use std::hash::{Hash, Hasher};
use stretch::{geometry, style};

/// The appearance of a [`Node`].
///
/// [`Node`]: struct.Node.html
#[derive(Debug, Clone, Copy)]
pub struct Style(pub(crate) style::Style);

impl Style {
    /// Defines the width of a [`Node`].
    ///
    /// [`Node`]: struct.Node.html
    pub fn width(mut self, width: Length) -> Self {
        self.0.size.width = length_to_dimension(width);
        self
    }

    /// Defines the height of a [`Node`].
    ///
    /// [`Node`]: struct.Node.html
    pub fn height(mut self, height: Length) -> Self {
        self.0.size.height = length_to_dimension(height);
        self
    }

    /// Defines the minimum width of a [`Node`].
    ///
    /// [`Node`]: struct.Node.html
    pub fn min_width(mut self, min_width: Length) -> Self {
        self.0.min_size.width = length_to_dimension(min_width);
        self
    }

    /// Defines the maximum width of a [`Node`].
    ///
    /// [`Node`]: struct.Node.html
    pub fn max_width(mut self, max_width: Length) -> Self {
        self.0.max_size.width = length_to_dimension(max_width);
        self
    }

    /// Defines the minimum height of a [`Node`].
    ///
    /// [`Node`]: struct.Node.html
    pub fn min_height(mut self, min_height: Length) -> Self {
        self.0.min_size.height = length_to_dimension(min_height);
        self
    }

    /// Defines the maximum height of a [`Node`].
    ///
    /// [`Node`]: struct.Node.html
    pub fn max_height(mut self, max_height: Length) -> Self {
        self.0.max_size.height = length_to_dimension(max_height);
        self
    }

    pub(crate) fn align_items(mut self, align: Align) -> Self {
        self.0.align_items = align.into();
        self
    }

    pub(crate) fn justify_content(mut self, justify: Justify) -> Self {
        self.0.justify_content = justify.into();
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
            Some(align) => align.into(),
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

fn length_to_dimension(length: Length) -> style::Dimension {
    match length {
        Length::Shrink => style::Dimension::Undefined,
        Length::Fill => style::Dimension::Percent(1.0),
        Length::Units(units) => style::Dimension::Points(units as f32),
    }
}

impl Default for Style {
    fn default() -> Style {
        Style(style::Style {
            align_items: style::AlignItems::FlexStart,
            justify_content: style::JustifyContent::FlexStart,
            ..style::Style::default()
        })
    }
}

impl Hash for Style {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_size(&self.0.size, state);
        hash_size(&self.0.min_size, state);
        hash_size(&self.0.max_size, state);

        hash_rect(&self.0.margin, state);

        (self.0.flex_direction as u8).hash(state);
        (self.0.align_items as u8).hash(state);
        (self.0.justify_content as u8).hash(state);
        (self.0.align_self as u8).hash(state);
        (self.0.flex_grow as u32).hash(state);
    }
}

fn hash_size<H: Hasher>(
    size: &geometry::Size<style::Dimension>,
    state: &mut H,
) {
    hash_dimension(size.width, state);
    hash_dimension(size.height, state);
}

fn hash_rect<H: Hasher>(
    rect: &geometry::Rect<style::Dimension>,
    state: &mut H,
) {
    hash_dimension(rect.start, state);
    hash_dimension(rect.end, state);
    hash_dimension(rect.top, state);
    hash_dimension(rect.bottom, state);
}

fn hash_dimension<H: Hasher>(dimension: style::Dimension, state: &mut H) {
    match dimension {
        style::Dimension::Undefined => state.write_u8(0),
        style::Dimension::Auto => state.write_u8(1),
        style::Dimension::Points(points) => {
            state.write_u8(2);
            (points as u32).hash(state);
        }
        style::Dimension::Percent(percent) => {
            state.write_u8(3);
            (percent as u32).hash(state);
        }
    }
}
