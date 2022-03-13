//! The styling library of Iced.
//!
//! It contains a set of styles and stylesheets for most of the built-in
//! widgets.
//!
//! ![The foundations of the Iced ecosystem](https://github.com/iced-rs/iced/blob/0525d76ff94e828b7b21634fa94a747022001c83/docs/graphs/foundations.png?raw=true)
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
pub use iced_core::{Background, Color};

pub mod button;
// pub mod checkbox;
pub mod container;
// pub mod menu;
// pub mod pane_grid;
// pub mod pick_list;
// pub mod progress_bar;
pub mod radio;
// pub mod rule;
// pub mod scrollable;
// pub mod slider;
// pub mod text_input;
// pub mod toggler;

pub trait Styling: Default {
    type Theme: Default;

    fn default_text_color(theme: &Self::Theme) -> Color;
}

/// The styling attributes of a [`Renderer`].
#[allow(missing_debug_implementations)]
pub struct IcedTheme {
    pub text: Color,
    pub needs_better_naming: Color,
    pub surface: Color,
    pub accent: Color,
    pub active: Color,
    pub hover: Color,
    pub highlight: Color,
    pub text_highlight: Color,
}

const DEFAULT_ICED_THEME: IcedTheme = IcedTheme {
    text: Color::BLACK,
    active: Color::from_rgb(0.3, 0.9, 0.3),
    accent: Color::from_rgb(0.7, 0.7, 0.7),
    surface: Color::from_rgb(0.9, 0.9, 0.9),
    hover: Color::from_rgb(0.8, 0.8, 0.8),
    needs_better_naming: Color::from_rgb(0.3, 0.3, 0.3),
    highlight: Color::from_rgb(0.4, 0.4, 0.1),
    text_highlight: Color::from_rgb(0.8, 0.8, 0.1),
};

impl Default for IcedTheme {
    fn default() -> Self {
        DEFAULT_ICED_THEME
    }
}
