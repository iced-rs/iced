mod palette;

pub use self::palette::Palette;

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

pub trait Definition {
    fn background_color(&self) -> Color;

    fn text_color(&self) -> Color;
}

impl Definition for Theme {
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
    type Variant = Button;

    fn active(&self, variant: Self::Variant) -> button::Style {
        let palette = self.extended_palette();

        let style = button::Style {
            border_radius: 2.0,
            ..button::Style::default()
        };

        match variant {
            Button::Primary => button::Style {
                background: Some(palette.primary.strong.into()),
                text_color: palette.primary.text,
                ..style
            },
            Button::Secondary => button::Style {
                background: Some(palette.background.weak.into()),
                text_color: palette.background.text,
                ..style
            },
            Button::Positive => button::Style {
                background: Some(palette.success.base.into()),
                text_color: palette.success.text,
                ..style
            },
            Button::Destructive => button::Style {
                background: Some(palette.danger.base.into()),
                text_color: palette.danger.text,
                ..style
            },
            Button::Text => button::Style {
                text_color: palette.background.text,
                ..style
            },
        }
    }

    fn hovered(&self, variant: Self::Variant) -> button::Style {
        let active = self.active(variant);
        let palette = self.extended_palette();

        let background = match variant {
            Button::Primary => Some(palette.primary.base),
            Button::Secondary => Some(palette.background.strong),
            Button::Positive => Some(palette.success.strong),
            Button::Destructive => Some(palette.danger.strong),
            Button::Text => None,
        };

        button::Style {
            background: background.map(Background::from),
            ..active
        }
    }
}

impl slider::StyleSheet for Theme {
    type Variant = ();

    fn active(&self, _variant: Self::Variant) -> slider::Style {
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

        slider::Style {
            rail_colors: (palette.primary.base, palette.background.base),
            handle: slider::Handle {
                color: palette.background.base,
                border_color: palette.primary.base,
                ..handle
            },
        }
    }

    fn hovered(&self, variant: Self::Variant) -> slider::Style {
        let active = self.active(variant);
        let palette = self.extended_palette();

        slider::Style {
            handle: slider::Handle {
                color: palette.primary.weak,
                ..active.handle
            },
            ..active
        }
    }

    fn dragging(&self, variant: Self::Variant) -> slider::Style {
        let active = self.active(variant);
        let palette = self.extended_palette();

        slider::Style {
            handle: slider::Handle {
                color: palette.primary.base,
                ..active.handle
            },
            ..active
        }
    }
}
