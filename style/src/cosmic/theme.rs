pub mod palette;

pub use self::palette::Palette;

use crate::application;
use crate::button;
use crate::checkbox;
use crate::container;
use crate::menu;
use crate::pane_grid;
use crate::pick_list;
use crate::progress_bar;
use crate::radio;
use crate::rule;
use crate::scrollable;
use crate::slider;
use crate::text;
use crate::text_input;
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

    pub fn extended_palette(&self) -> &palette::Extended {
        match self {
            Self::Light => &palette::EXTENDED_LIGHT,
            Self::Dark => &palette::EXTENDED_DARK,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::Dark
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Application {
    Default,
    Custom(fn(Theme) -> application::Appearance),
}

impl Default for Application {
    fn default() -> Self {
        Self::Default
    }
}

impl application::StyleSheet for Theme {
    type Style = Application;

    fn appearance(&self, style: Self::Style) -> application::Appearance {
        let palette = self.extended_palette();

        match style {
            Application::Default => application::Appearance {
                background_color: palette.background.base.color,
                text_color: palette.background.base.text,
            },
            Application::Custom(f) => f(*self),
        }
    }
}

/*
 * TODO: Button
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
 * TODO: Checkbox
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Checkbox {
    Primary,
    Secondary,
    Success,
    Danger,
}

impl Default for Checkbox {
    fn default() -> Self {
        Self::Primary
    }
}

impl checkbox::StyleSheet for Theme {
    type Style = Checkbox;

    fn active(
        &self,
        style: Self::Style,
        is_checked: bool,
    ) -> checkbox::Appearance {
        let palette = self.extended_palette();

        match style {
            Checkbox::Primary => checkbox_appearance(
                palette.primary.strong.text,
                palette.background.base,
                palette.primary.strong,
                is_checked,
            ),
            Checkbox::Secondary => checkbox_appearance(
                palette.background.base.text,
                palette.background.base,
                palette.background.base,
                is_checked,
            ),
            Checkbox::Success => checkbox_appearance(
                palette.success.base.text,
                palette.background.base,
                palette.success.base,
                is_checked,
            ),
            Checkbox::Danger => checkbox_appearance(
                palette.danger.base.text,
                palette.background.base,
                palette.danger.base,
                is_checked,
            ),
        }
    }

    fn hovered(
        &self,
        style: Self::Style,
        is_checked: bool,
    ) -> checkbox::Appearance {
        let palette = self.extended_palette();

        match style {
            Checkbox::Primary => checkbox_appearance(
                palette.primary.strong.text,
                palette.background.weak,
                palette.primary.base,
                is_checked,
            ),
            Checkbox::Secondary => checkbox_appearance(
                palette.background.base.text,
                palette.background.weak,
                palette.background.base,
                is_checked,
            ),
            Checkbox::Success => checkbox_appearance(
                palette.success.base.text,
                palette.background.weak,
                palette.success.base,
                is_checked,
            ),
            Checkbox::Danger => checkbox_appearance(
                palette.danger.base.text,
                palette.background.weak,
                palette.danger.base,
                is_checked,
            ),
        }
    }
}

fn checkbox_appearance(
    checkmark_color: Color,
    base: palette::Pair,
    accent: palette::Pair,
    is_checked: bool,
) -> checkbox::Appearance {
    checkbox::Appearance {
        background: Background::Color(if is_checked {
            accent.color
        } else {
            base.color
        }),
        checkmark_color,
        border_radius: 2.0,
        border_width: 1.0,
        border_color: accent.color,
        text_color: None,
    }
}

/*
 * TODO: Container
 */
#[derive(Clone, Copy)]
pub enum Container {
    Transparent,
    Box,
    Custom(fn(&Theme) -> container::Appearance),
}

impl Default for Container {
    fn default() -> Self {
        Self::Transparent
    }
}

impl From<fn(&Theme) -> container::Appearance> for Container {
    fn from(f: fn(&Theme) -> container::Appearance) -> Self {
        Self::Custom(f)
    }
}

impl container::StyleSheet for Theme {
    type Style = Container;

    fn appearance(&self, style: Self::Style) -> container::Appearance {
        match style {
            Container::Transparent => Default::default(),
            Container::Box => {
                let palette = self.extended_palette();

                container::Appearance {
                    text_color: None,
                    background: palette.background.weak.color.into(),
                    border_radius: 2.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                }
            }
            Container::Custom(f) => f(self),
        }
    }
}

/*
 * Slider
 */
impl slider::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: Self::Style) -> slider::Appearance {
        //TODO: use palette
        //TODO: no way to set rail thickness
        match self {
            Theme::Dark => slider::Appearance {
                rail_colors: (
                    Color::from_rgba8(0x94, 0xEB, 0xEB, 0.75),
                    //TODO: no way to set color before/after slider: Color::from_rgb8(0x78, 0x78, 0x78),
                    Color::TRANSPARENT,
                ),
                handle: slider::Handle {
                    shape: slider::HandleShape::Circle {
                        radius: 10.0,
                    },
                    color: Color::from_rgb8(0x94, 0xEB, 0xEB),
                    border_color: Color::from_rgba8(0, 0, 0, 0.0),
                    border_width: 0.0,
                }
            },
            Theme::Light => slider::Appearance {
                rail_colors: (
                    Color::from_rgba8(0x00, 0x49, 0x6D, 0.75),
                    //TODO: no way to set color before/after slider: Color::from_rgb8(0x93, 0x93, 0x93),
                    Color::TRANSPARENT,
                ),
                handle: slider::Handle {
                    shape: slider::HandleShape::Circle {
                        radius: 10.0,
                    },
                    color: Color::from_rgb8(0x00, 0x49, 0x6D),
                    border_width: 0.0,
                    border_color: Color::from_rgba8(0, 0, 0, 0.0),
                }
            },
        }
    }

    fn hovered(&self, style: Self::Style) -> slider::Appearance {
        let mut style = self.active(style);
        style.handle.shape = slider::HandleShape::Circle {
            radius: 16.0
        };
        style.handle.border_width = 6.0;
        style.handle.border_color = match self {
            Theme::Dark => Color::from_rgba8(0xFF, 0xFF, 0xFF, 0.1),
            Theme::Light => Color::from_rgba8(0, 0, 0, 0.1),
        };
        style
    }

    fn dragging(&self, style: Self::Style) -> slider::Appearance {
        let mut style = self.hovered(style);
        style.handle.border_color = match self {
            Theme::Dark => Color::from_rgba8(0xFF, 0xFF, 0xFF, 0.2),
            Theme::Light => Color::from_rgba8(0, 0, 0, 0.2),
        };
        style
    }
}

/*
 * TODO: Menu
 */
impl menu::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: Self::Style) -> menu::Appearance {
        let palette = self.extended_palette();

        menu::Appearance {
            text_color: palette.background.weak.text,
            background: palette.background.weak.color.into(),
            border_width: 1.0,
            border_radius: 0.0,
            border_color: palette.background.strong.color,
            selected_text_color: palette.primary.strong.text,
            selected_background: palette.primary.strong.color.into(),
        }
    }
}

/*
 * TODO: Pick List
 */
impl pick_list::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: ()) -> pick_list::Appearance {
        let palette = self.extended_palette();

        pick_list::Appearance {
            text_color: palette.background.weak.text,
            background: palette.background.weak.color.into(),
            placeholder_color: palette.background.strong.color,
            border_radius: 2.0,
            border_width: 1.0,
            border_color: palette.background.strong.color,
            icon_size: 0.7,
        }
    }

    fn hovered(&self, _style: ()) -> pick_list::Appearance {
        let palette = self.extended_palette();

        pick_list::Appearance {
            text_color: palette.background.weak.text,
            background: palette.background.weak.color.into(),
            placeholder_color: palette.background.strong.color,
            border_radius: 2.0,
            border_width: 1.0,
            border_color: palette.primary.strong.color,
            icon_size: 0.7,
        }
    }
}

/*
 * TODO: Radio
 */
impl radio::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: Self::Style) -> radio::Appearance {
        let palette = self.extended_palette();

        radio::Appearance {
            background: Color::TRANSPARENT.into(),
            dot_color: palette.primary.strong.color,
            border_width: 1.0,
            border_color: palette.primary.strong.color,
            text_color: None,
        }
    }

    fn hovered(&self, style: Self::Style) -> radio::Appearance {
        let active = self.active(style);
        let palette = self.extended_palette();

        radio::Appearance {
            dot_color: palette.primary.strong.color,
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
        //TODO: use palette
        match self {
            Theme::Dark => toggler::Appearance {
                background: if is_active {
                    Color::from_rgb8(0x94, 0xEB, 0xEB)
                } else {
                    Color::from_rgb8(0x78, 0x78, 0x78)
                },
                background_border: None,
                foreground: Color::from_rgb8(0x27, 0x27, 0x27),
                foreground_border: None,
            },
            Theme::Light => toggler::Appearance {
                background: if is_active {
                    Color::from_rgb8(0x00, 0x49, 0x6D)
                } else {
                    Color::from_rgb8(0x93, 0x93, 0x93)
                },
                background_border: None,
                foreground: Color::from_rgb8(0xE4, 0xE4, 0xE4),
                foreground_border: None,
            }
        }
    }

    fn hovered(
        &self,
        style: Self::Style,
        is_active: bool,
    ) -> toggler::Appearance {
        match self {
            Theme::Dark  => toggler::Appearance {
                background: if is_active {
                    Color::from_rgb8(0x9f, 0xed, 0xed)
                } else {
                    Color::from_rgb8(0xb6, 0xb6, 0xb6)
                },
                ..self.active(style, is_active)
            },
            Theme::Light  => toggler::Appearance {
                background: if is_active {
                    Color::from_rgb8(0x00, 0x42, 0x62)
                } else {
                    Color::from_rgb8(0x54, 0x54, 0x54)
                },
                ..self.active(style, is_active)
            }
        }
    }
}

/*
 * TODO: Pane Grid
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
 * TODO: Progress Bar
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
            background: palette.background.strong.color.into(),
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
 * TODO: Rule
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

/*
 * TODO: Scrollable
 */
impl scrollable::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: Self::Style) -> scrollable::Scrollbar {
        let palette = self.extended_palette();

        scrollable::Scrollbar {
            background: palette.background.weak.color.into(),
            border_radius: 2.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            scroller: scrollable::Scroller {
                color: palette.background.strong.color,
                border_radius: 2.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }

    fn hovered(&self, _style: Self::Style) -> scrollable::Scrollbar {
        let palette = self.extended_palette();

        scrollable::Scrollbar {
            background: palette.background.weak.color.into(),
            border_radius: 2.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            scroller: scrollable::Scroller {
                color: palette.primary.strong.color,
                border_radius: 2.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }
}

/*
 * TODO: Text
 */
#[derive(Clone, Copy)]
pub enum Text {
    Default,
    Color(Color),
    Custom(fn(&Theme) -> text::Appearance),
}

impl Default for Text {
    fn default() -> Self {
        Self::Default
    }
}

impl From<Color> for Text {
    fn from(color: Color) -> Self {
        Text::Color(color)
    }
}

impl text::StyleSheet for Theme {
    type Style = Text;

    fn appearance(&self, style: Self::Style) -> text::Appearance {
        match style {
            Text::Default => Default::default(),
            Text::Color(c) => text::Appearance { color: Some(c) },
            Text::Custom(f) => f(self),
        }
    }
}

/*
 * TODO: Text Input
 */
impl text_input::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: Self::Style) -> text_input::Appearance {
        let palette = self.extended_palette();

        text_input::Appearance {
            background: palette.background.base.color.into(),
            border_radius: 2.0,
            border_width: 1.0,
            border_color: palette.background.strong.color,
        }
    }

    fn hovered(&self, _style: Self::Style) -> text_input::Appearance {
        let palette = self.extended_palette();

        text_input::Appearance {
            background: palette.background.base.color.into(),
            border_radius: 2.0,
            border_width: 1.0,
            border_color: palette.background.base.text,
        }
    }

    fn focused(&self, _style: Self::Style) -> text_input::Appearance {
        let palette = self.extended_palette();

        text_input::Appearance {
            background: palette.background.base.color.into(),
            border_radius: 2.0,
            border_width: 1.0,
            border_color: palette.primary.strong.color,
        }
    }

    fn placeholder_color(&self, _style: Self::Style) -> Color {
        let palette = self.extended_palette();

        palette.background.strong.color
    }

    fn value_color(&self, _style: Self::Style) -> Color {
        let palette = self.extended_palette();

        palette.background.base.text
    }

    fn selection_color(&self, _style: Self::Style) -> Color {
        let palette = self.extended_palette();

        palette.primary.weak.color
    }
}
