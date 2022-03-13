// mod dark;
// mod ugly;

use iced::radio::{Style, StyleSheet};
use iced::{button, container, radio, Background, Color, Vector};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeChoice {
    /// The default iced theme
    IcedDefault,
    /// The dark theme
    Dark,
    /// The ugliest color theme to exist
    Ugly,
}

impl ThemeChoice {
    pub const ALL: [ThemeChoice; 3] = [
        ThemeChoice::IcedDefault,
        ThemeChoice::Dark,
        ThemeChoice::Ugly,
    ];
}

impl std::default::Default for ThemeChoice {
    fn default() -> ThemeChoice {
        ThemeChoice::IcedDefault
    }
}

pub struct Theme {
    text: Color,
    background: Color,
    surface: Color,
    border: Color,
    radio_dot_color: Color,
    accent: Color,
    hover: Color,
}

const DEFAULT_THEME: Theme = Theme {
    text: Color::BLACK,
    background: Color::from_rgb(0.95, 0.95, 0.95),
    surface: Color::BLACK,
    border: Color::BLACK,
    radio_dot_color: Color::from_rgb(0.3, 0.3, 0.3),
    accent: Color::BLACK,
    hover: Color::from_rgb(0.90, 0.90, 0.90),
};

const DARK_THEME: Theme = Theme {
    text: Color::WHITE,
    background: Color::from_rgb(
        0x40 as f32 / 255.0,
        0x44 as f32 / 255.0,
        0x4B as f32 / 255.0,
    ),
    surface: Color::from_rgb(
        0x36 as f32 / 255.0,
        0x39 as f32 / 255.0,
        0x3F as f32 / 255.0,
    ),
    border: Color::BLACK,
    radio_dot_color: Color::from_rgb(0.3, 0.3, 0.3),
    accent: Color::BLACK,
    hover: Color::from_rgb(0.90, 0.90, 0.90),
};

const UGLY_THEME: Theme = Theme {
    text: Color::from_rgb(0.2, 0.1, 1.0),
    background: Color::from_rgb(1.0, 1.0, 0.0),
    surface: Color::BLACK,
    border: Color::BLACK,
    radio_dot_color: Color::from_rgb(0.3, 0.9, 0.5),
    accent: Color::BLACK,
    hover: Color::from_rgb(0.7, 0.07, 0.7),
};
impl From<ThemeChoice> for Theme {
    fn from(choice: ThemeChoice) -> Self {
        match choice {
            ThemeChoice::IcedDefault => DEFAULT_THEME,
            ThemeChoice::Dark => DARK_THEME,
            ThemeChoice::Ugly => UGLY_THEME,
        }
    }
}

impl std::default::Default for Theme {
    fn default() -> Self {
        DEFAULT_THEME
    }
}

pub struct Styling;

impl std::default::Default for Styling {
    fn default() -> Self {
        Styling {}
    }
}

impl iced::Styling for Styling {
    type Theme = Theme;

    fn default_text_color(_theme: &Self::Theme) -> Color {
        Color::BLACK
    }
}

impl button::StyleSheet for Styling {
    type Theme = Theme;

    fn active(&self, theme: &Self::Theme) -> button::Style {
        button::Style {
            shadow_offset: Vector::new(0.0, 0.0),
            background: Some(theme.hover.into()),
            border_radius: 2.0,
            border_width: 1.0,
            border_color: theme.border,
        }
    }

    fn hovered(&self, theme: &Self::Theme) -> button::Style {
        let active = button::StyleSheet::active(self, theme);

        button::Style {
            shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
            ..active
        }
    }

    fn pressed(&self, theme: &Self::Theme) -> button::Style {
        button::Style {
            shadow_offset: Vector::default(),
            ..iced::button::StyleSheet::active(self, theme)
        }
    }

    fn disabled(&self, theme: &Self::Theme) -> button::Style {
        let active = iced::button::StyleSheet::active(self, theme);

        button::Style {
            shadow_offset: Vector::default(),
            background: active.background.map(|background| match background {
                Background::Color(color) => Background::Color(Color {
                    a: color.a * 0.5,
                    ..color
                }),
            }),
            ..active
        }
    }
}

impl radio::StyleSheet for Styling {
    type Theme = Theme;

    fn active(&self, theme: &Self::Theme) -> Style {
        Style {
            background: theme.background.into(),
            dot_color: theme.radio_dot_color,
            border_width: 1.0,
            border_color: theme.accent,
            text_color: Some(theme.text),
        }
    }

    fn hovered(&self, theme: &Self::Theme) -> Style {
        Style {
            background: theme.hover.into(),
            ..self.active(theme)
        }
    }
}

impl container::StyleSheet for Styling {
    type Theme = Theme;

    fn style(&self, theme: &Self::Theme) -> container::Style {
        container::Style {
            text_color: Some(theme.text),
            background: theme.background.into(),
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }
}
