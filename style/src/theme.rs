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
        Self::Light
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
 * Checkbox
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
 * Container
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
 * Menu
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
 * Pick List
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
 * Radio
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

/*
 * Scrollable
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
 * Text
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
 * Text Input
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
