mod palette;

pub use self::palette::Palette;

use crate::application;
use crate::button;
use crate::pane_grid;
use crate::progress_bar;
use crate::radio;
use crate::rule;
use crate::slider;
use crate::toggler;

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

        palette.background.base.color
    }

    fn text_color(&self) -> Color {
        let palette = self.extended_palette();

        palette.background.base.text
    }
}

/*
 * Button
 */
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

        let from_pair = |pair: palette::Pair| button::Appearance {
            background: Some(pair.color.into()),
            text_color: pair.text,
            ..appearance
        };

        match style {
            Button::Primary => from_pair(palette.primary.strong),
            Button::Secondary => from_pair(palette.secondary.base),
            Button::Positive => from_pair(palette.success.base),
            Button::Destructive => from_pair(palette.danger.base),
            Button::Text => button::Appearance {
                text_color: palette.background.base.text,
                ..appearance
            },
        }
    }

    fn hovered(&self, style: Self::Style) -> button::Appearance {
        let active = self.active(style);
        let palette = self.extended_palette();

        let background = match style {
            Button::Primary => Some(palette.primary.base.color),
            Button::Secondary => Some(palette.background.strong.color),
            Button::Positive => Some(palette.success.strong.color),
            Button::Destructive => Some(palette.danger.strong.color),
            Button::Text => None,
        };

        button::Appearance {
            background: background.map(Background::from),
            ..active
        }
    }
}

/*
 * Slider
 */
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
            rail_colors: (palette.primary.base.color, Color::TRANSPARENT),
            handle: slider::Handle {
                color: palette.background.base.color,
                border_color: palette.primary.base.color,
                ..handle
            },
        }
    }

    fn hovered(&self, style: Self::Style) -> slider::Appearance {
        let active = self.active(style);
        let palette = self.extended_palette();

        slider::Appearance {
            handle: slider::Handle {
                color: palette.primary.weak.color,
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
                color: palette.primary.base.color,
                ..active.handle
            },
            ..active
        }
    }
}

/*
 * Radio
 */
impl radio::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: Self::Style) -> radio::Appearance {
        let palette = self.extended_palette();

        radio::Appearance {
            background: Color::TRANSPARENT.into(),
            dot_color: palette.primary.strong.color.into(),
            border_width: 1.0,
            border_color: palette.primary.strong.color,
            text_color: None,
        }
    }

    fn hovered(&self, style: Self::Style) -> radio::Appearance {
        let active = self.active(style);
        let palette = self.extended_palette();

        radio::Appearance {
            dot_color: palette.primary.weak.text.into(),
            background: palette.primary.weak.color.into(),
            ..active
        }
    }
}

/*
 * Toggler
 */
impl toggler::StyleSheet for Theme {
    type Style = ();

    fn active(
        &self,
        _style: Self::Style,
        is_active: bool,
    ) -> toggler::Appearance {
        let palette = self.extended_palette();

        toggler::Appearance {
            background: if is_active {
                palette.primary.strong.color
            } else {
                palette.background.strong.color
            },
            background_border: None,
            foreground: if is_active {
                palette.primary.strong.text
            } else {
                palette.background.base.color
            },
            foreground_border: None,
        }
    }

    fn hovered(
        &self,
        style: Self::Style,
        is_active: bool,
    ) -> toggler::Appearance {
        let palette = self.extended_palette();

        toggler::Appearance {
            foreground: if is_active {
                Color {
                    a: 0.5,
                    ..palette.primary.strong.text
                }
            } else {
                palette.background.weak.color
            },
            ..self.active(style, is_active)
        }
    }
}

/*
 * Pane Grid
 */
impl pane_grid::StyleSheet for Theme {
    type Style = ();

    fn picked_split(&self, _style: Self::Style) -> Option<pane_grid::Line> {
        let palette = self.extended_palette();

        Some(pane_grid::Line {
            color: palette.primary.strong.color,
            width: 2.0,
        })
    }

    fn hovered_split(&self, _style: Self::Style) -> Option<pane_grid::Line> {
        let palette = self.extended_palette();

        Some(pane_grid::Line {
            color: palette.primary.base.color,
            width: 2.0,
        })
    }
}

/*
 * Progress Bar
 */
#[derive(Clone, Copy)]
pub enum ProgressBar {
    Primary,
    Success,
    Danger,
    Custom(fn(&Theme) -> progress_bar::Appearance),
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::Primary
    }
}

impl progress_bar::StyleSheet for Theme {
    type Style = ProgressBar;

    fn appearance(&self, style: Self::Style) -> progress_bar::Appearance {
        let palette = self.extended_palette();

        let from_palette = |bar: Color| progress_bar::Appearance {
            background: palette.background.weak.color.into(),
            bar: bar.into(),
            border_radius: 2.0,
        };

        match style {
            ProgressBar::Primary => from_palette(palette.primary.base.color),
            ProgressBar::Success => from_palette(palette.success.base.color),
            ProgressBar::Danger => from_palette(palette.danger.base.color),
            ProgressBar::Custom(f) => f(self),
        }
    }
}

/*
 * Rule
 */
#[derive(Clone, Copy)]
pub enum Rule {
    Default,
    Custom(fn(&Theme) -> rule::Appearance),
}

impl Default for Rule {
    fn default() -> Self {
        Self::Default
    }
}

impl rule::StyleSheet for Theme {
    type Style = Rule;

    fn style(&self, style: Self::Style) -> rule::Appearance {
        let palette = self.extended_palette();

        match style {
            Rule::Default => rule::Appearance {
                color: palette.background.strong.color,
                width: 1,
                radius: 0.0,
                fill_mode: rule::FillMode::Full,
            },
            Rule::Custom(f) => f(self),
        }
    }
}
