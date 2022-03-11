//! Navigate an endless amount of content with a scrollbar.
use crate::Theme;
use iced_core::{Background, Color};

/// The appearance of a scrollable.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub background: Option<Background>,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
    pub scroller: Scroller,
}

/// The appearance of the scroller of a scrollable.
#[derive(Debug, Clone, Copy)]
pub struct Scroller {
    pub color: Color,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
}

/// A set of rules that dictate the style of a scrollable.
pub trait StyleSheet<ColorPalette> {
    fn get_style(
        &self,
        color_palette: &ColorPalette,
        is_scroller_grabbed: bool,
        is_mouse_over_scrollbar: bool,
    ) -> Style {
        if is_scroller_grabbed {
            self.dragging(color_palette)
        } else if is_mouse_over_scrollbar {
            self.hovered(color_palette)
        } else {
            self.active(color_palette)
        }
    }

    /// Produces the style of an active scrollbar.
    fn active(&self, color_palette: &ColorPalette) -> Style;

    /// Produces the style of an hovered scrollbar.
    fn hovered(&self, color_palette: &ColorPalette) -> Style;

    /// Produces the style of a scrollbar that is being dragged.
    fn dragging(&self, color_palette: &ColorPalette) -> Style {
        self.hovered(color_palette)
    }
}

struct Default;

impl StyleSheet<IcedColorPalette> for Default {
    fn active(&self, color_palette: &ColorPalette) -> Style {
        Style {
            background: None,
            border_radius: 5.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            scroller: Scroller {
                color: color_palette.active,
                border_radius: 5.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }

    fn hovered(&self, color_palette: &ColorPalette) -> Style {
        Style {
            background: Some(
                Color {
                    a: 0.5,
                    ..color_palette.surface
                }
                .into(),
            ),
            ..self.active(color_palette)
        }
    }
}

impl<'a> std::default::Default for Box<dyn StyleSheet + 'a> {
    fn default() -> Self {
        Box::new(Default)
    }
}

impl<'a, T> From<T> for Box<dyn StyleSheet + 'a>
where
    T: StyleSheet + 'a,
{
    fn from(style_sheet: T) -> Self {
        Box::new(style_sheet)
    }
}
