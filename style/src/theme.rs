mod palette;

pub use self::palette::Palette;

use crate::application;
use crate::button;
use crate::slider;

use iced_core::{Background, Color};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn palette(self) -> Palette {
        match self {
            Self::Light => Palette::LIGHT,
            Self::Dark => Palette::DARK,
        }
    }

    fn extended_palette(&self) -> &palette::Extended {
        match self {
            Self::Light => &palette::EXTENDED_LIGHT,
            Self::Dark => &palette::EXTENDED_DARK,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::Light
    }
}

impl application::StyleSheet for Theme {
    fn background_color(&self) -> Color {
        let palette = self.extended_palette();

        palette.background.base
    }

    fn text_color(&self) -> Color {
        let palette = self.extended_palette();

        palette.background.text
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Primary,
    Secondary,
    Positive,
    Destructive,
    Text,
}

impl Default for Button {
    fn default() -> Self {
        Self::Primary
    }
}

impl button::StyleSheet for Theme {
    type Style = Button;

    fn active(&self, style: Self::Style) -> button::Appearance {
        let palette = self.extended_palette();

        let appearance = button::Appearance {
            border_radius: 2.0,
            ..button::Appearance::default()
        };

        match style {
            Button::Primary => button::Appearance {
                background: Some(palette.primary.strong.into()),
                text_color: palette.primary.text,
                ..appearance
            },
            Button::Secondary => button::Appearance {
                background: Some(palette.background.weak.into()),
                text_color: palette.background.text,
                ..appearance
            },
            Button::Positive => button::Appearance {
                background: Some(palette.success.base.into()),
                text_color: palette.success.text,
                ..appearance
            },
            Button::Destructive => button::Appearance {
                background: Some(palette.danger.base.into()),
                text_color: palette.danger.text,
                ..appearance
            },
            Button::Text => button::Appearance {
                text_color: palette.background.text,
                ..appearance
            },
        }
    }

    fn hovered(&self, style: Self::Style) -> button::Appearance {
        let active = self.active(style);
        let palette = self.extended_palette();

        let background = match style {
            Button::Primary => Some(palette.primary.base),
            Button::Secondary => Some(palette.background.strong),
            Button::Positive => Some(palette.success.strong),
            Button::Destructive => Some(palette.danger.strong),
            Button::Text => None,
        };

        button::Appearance {
            background: background.map(Background::from),
            ..active
        }
    }
}

impl slider::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: Self::Style) -> slider::Appearance {
        let palette = self.extended_palette();

        let handle = slider::Handle {
            shape: slider::HandleShape::Rectangle {
                width: 8,
                border_radius: 4.0,
            },
            color: Color::WHITE,
            border_color: Color::WHITE,
            border_width: 1.0,
        };

        slider::Appearance {
            rail_colors: (palette.primary.base, palette.background.base),
            handle: slider::Handle {
                color: palette.background.base,
                border_color: palette.primary.base,
                ..handle
            },
        }
    }

    fn hovered(&self, style: Self::Style) -> slider::Appearance {
        let active = self.active(style);
        let palette = self.extended_palette();

        slider::Appearance {
            handle: slider::Handle {
                color: palette.primary.weak,
                ..active.handle
            },
            ..active
        }
    }

    fn dragging(&self, style: Self::Style) -> slider::Appearance {
        let active = self.active(style);
        let palette = self.extended_palette();

        slider::Appearance {
            handle: slider::Handle {
                color: palette.primary.base,
                ..active.handle
            },
            ..active
        }
    }
}
