//! Use the built-in theme and styles.
pub mod palette;

pub use palette::Palette;

use crate::application;
use crate::button;
use crate::checkbox;
use crate::container;
use crate::core::widget::text;
use crate::menu;
use crate::pane_grid;
use crate::pick_list;
use crate::progress_bar;
use crate::qr_code;
use crate::radio;
use crate::rule;
use crate::scrollable;
use crate::slider;
use crate::svg;
use crate::text_editor;
use crate::text_input;
use crate::toggler;

use crate::core::{Background, Border, Color, Shadow, Vector};

use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

/// A built-in theme.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Theme {
    /// The built-in light variant.
    #[default]
    Light,
    /// The built-in dark variant.
    Dark,
    /// The built-in Dracula variant.
    Dracula,
    /// The built-in Nord variant.
    Nord,
    /// The built-in Solarized Light variant.
    SolarizedLight,
    /// The built-in Solarized Dark variant.
    SolarizedDark,
    /// The built-in Gruvbox Light variant.
    GruvboxLight,
    /// The built-in Gruvbox Dark variant.
    GruvboxDark,
    /// The built-in Catppuccin Latte variant.
    CatppuccinLatte,
    /// The built-in Catppuccin Frappé variant.
    CatppuccinFrappe,
    /// The built-in Catppuccin Macchiato variant.
    CatppuccinMacchiato,
    /// The built-in Catppuccin Mocha variant.
    CatppuccinMocha,
    /// The built-in Tokyo Night variant.
    TokyoNight,
    /// The built-in Tokyo Night Storm variant.
    TokyoNightStorm,
    /// The built-in Tokyo Night Light variant.
    TokyoNightLight,
    /// The built-in Kanagawa Wave variant.
    KanagawaWave,
    /// The built-in Kanagawa Dragon variant.
    KanagawaDragon,
    /// The built-in Kanagawa Lotus variant.
    KanagawaLotus,
    /// The built-in Moonfly variant.
    Moonfly,
    /// The built-in Nightfly variant.
    Nightfly,
    /// The built-in Oxocarbon variant.
    Oxocarbon,
    /// A [`Theme`] that uses a [`Custom`] palette.
    Custom(Arc<Custom>),
}

impl Theme {
    /// A list with all the defined themes.
    pub const ALL: &'static [Self] = &[
        Self::Light,
        Self::Dark,
        Self::Dracula,
        Self::Nord,
        Self::SolarizedLight,
        Self::SolarizedDark,
        Self::GruvboxLight,
        Self::GruvboxDark,
        Self::CatppuccinLatte,
        Self::CatppuccinFrappe,
        Self::CatppuccinMacchiato,
        Self::CatppuccinMocha,
        Self::TokyoNight,
        Self::TokyoNightStorm,
        Self::TokyoNightLight,
        Self::KanagawaWave,
        Self::KanagawaDragon,
        Self::KanagawaLotus,
        Self::Moonfly,
        Self::Nightfly,
        Self::Oxocarbon,
    ];

    /// Creates a new custom [`Theme`] from the given [`Palette`].
    pub fn custom(name: String, palette: Palette) -> Self {
        Self::custom_with_fn(name, palette, palette::Extended::generate)
    }

    /// Creates a new custom [`Theme`] from the given [`Palette`], with
    /// a custom generator of a [`palette::Extended`].
    pub fn custom_with_fn(
        name: String,
        palette: Palette,
        generate: impl FnOnce(Palette) -> palette::Extended,
    ) -> Self {
        Self::Custom(Arc::new(Custom::with_fn(name, palette, generate)))
    }

    /// Returns the [`Palette`] of the [`Theme`].
    pub fn palette(&self) -> Palette {
        match self {
            Self::Light => Palette::LIGHT,
            Self::Dark => Palette::DARK,
            Self::Dracula => Palette::DRACULA,
            Self::Nord => Palette::NORD,
            Self::SolarizedLight => Palette::SOLARIZED_LIGHT,
            Self::SolarizedDark => Palette::SOLARIZED_DARK,
            Self::GruvboxLight => Palette::GRUVBOX_LIGHT,
            Self::GruvboxDark => Palette::GRUVBOX_DARK,
            Self::CatppuccinLatte => Palette::CATPPUCCIN_LATTE,
            Self::CatppuccinFrappe => Palette::CATPPUCCIN_FRAPPE,
            Self::CatppuccinMacchiato => Palette::CATPPUCCIN_MACCHIATO,
            Self::CatppuccinMocha => Palette::CATPPUCCIN_MOCHA,
            Self::TokyoNight => Palette::TOKYO_NIGHT,
            Self::TokyoNightStorm => Palette::TOKYO_NIGHT_STORM,
            Self::TokyoNightLight => Palette::TOKYO_NIGHT_LIGHT,
            Self::KanagawaWave => Palette::KANAGAWA_WAVE,
            Self::KanagawaDragon => Palette::KANAGAWA_DRAGON,
            Self::KanagawaLotus => Palette::KANAGAWA_LOTUS,
            Self::Moonfly => Palette::MOONFLY,
            Self::Nightfly => Palette::NIGHTFLY,
            Self::Oxocarbon => Palette::OXOCARBON,
            Self::Custom(custom) => custom.palette,
        }
    }

    /// Returns the [`palette::Extended`] of the [`Theme`].
    pub fn extended_palette(&self) -> &palette::Extended {
        match self {
            Self::Light => &palette::EXTENDED_LIGHT,
            Self::Dark => &palette::EXTENDED_DARK,
            Self::Dracula => &palette::EXTENDED_DRACULA,
            Self::Nord => &palette::EXTENDED_NORD,
            Self::SolarizedLight => &palette::EXTENDED_SOLARIZED_LIGHT,
            Self::SolarizedDark => &palette::EXTENDED_SOLARIZED_DARK,
            Self::GruvboxLight => &palette::EXTENDED_GRUVBOX_LIGHT,
            Self::GruvboxDark => &palette::EXTENDED_GRUVBOX_DARK,
            Self::CatppuccinLatte => &palette::EXTENDED_CATPPUCCIN_LATTE,
            Self::CatppuccinFrappe => &palette::EXTENDED_CATPPUCCIN_FRAPPE,
            Self::CatppuccinMacchiato => {
                &palette::EXTENDED_CATPPUCCIN_MACCHIATO
            }
            Self::CatppuccinMocha => &palette::EXTENDED_CATPPUCCIN_MOCHA,
            Self::TokyoNight => &palette::EXTENDED_TOKYO_NIGHT,
            Self::TokyoNightStorm => &palette::EXTENDED_TOKYO_NIGHT_STORM,
            Self::TokyoNightLight => &palette::EXTENDED_TOKYO_NIGHT_LIGHT,
            Self::KanagawaWave => &palette::EXTENDED_KANAGAWA_WAVE,
            Self::KanagawaDragon => &palette::EXTENDED_KANAGAWA_DRAGON,
            Self::KanagawaLotus => &palette::EXTENDED_KANAGAWA_LOTUS,
            Self::Moonfly => &palette::EXTENDED_MOONFLY,
            Self::Nightfly => &palette::EXTENDED_NIGHTFLY,
            Self::Oxocarbon => &palette::EXTENDED_OXOCARBON,
            Self::Custom(custom) => &custom.extended,
        }
    }
}

impl fmt::Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Light => write!(f, "Light"),
            Self::Dark => write!(f, "Dark"),
            Self::Dracula => write!(f, "Dracula"),
            Self::Nord => write!(f, "Nord"),
            Self::SolarizedLight => write!(f, "Solarized Light"),
            Self::SolarizedDark => write!(f, "Solarized Dark"),
            Self::GruvboxLight => write!(f, "Gruvbox Light"),
            Self::GruvboxDark => write!(f, "Gruvbox Dark"),
            Self::CatppuccinLatte => write!(f, "Catppuccin Latte"),
            Self::CatppuccinFrappe => write!(f, "Catppuccin Frappé"),
            Self::CatppuccinMacchiato => write!(f, "Catppuccin Macchiato"),
            Self::CatppuccinMocha => write!(f, "Catppuccin Mocha"),
            Self::TokyoNight => write!(f, "Tokyo Night"),
            Self::TokyoNightStorm => write!(f, "Tokyo Night Storm"),
            Self::TokyoNightLight => write!(f, "Tokyo Night Light"),
            Self::KanagawaWave => write!(f, "Kanagawa Wave"),
            Self::KanagawaDragon => write!(f, "Kanagawa Dragon"),
            Self::KanagawaLotus => write!(f, "Kanagawa Lotus"),
            Self::Moonfly => write!(f, "Moonfly"),
            Self::Nightfly => write!(f, "Nightfly"),
            Self::Oxocarbon => write!(f, "Oxocarbon"),
            Self::Custom(custom) => custom.fmt(f),
        }
    }
}

/// A [`Theme`] with a customized [`Palette`].
#[derive(Debug, Clone, PartialEq)]
pub struct Custom {
    name: String,
    palette: Palette,
    extended: palette::Extended,
}

impl Custom {
    /// Creates a [`Custom`] theme from the given [`Palette`].
    pub fn new(name: String, palette: Palette) -> Self {
        Self::with_fn(name, palette, palette::Extended::generate)
    }

    /// Creates a [`Custom`] theme from the given [`Palette`] with
    /// a custom generator of a [`palette::Extended`].
    pub fn with_fn(
        name: String,
        palette: Palette,
        generate: impl FnOnce(Palette) -> palette::Extended,
    ) -> Self {
        Self {
            name,
            palette,
            extended: generate(palette),
        }
    }
}

impl fmt::Display for Custom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// The style of an application.
#[derive(Default)]
pub enum Application {
    /// The default style.
    #[default]
    Default,
    /// A custom style.
    Custom(Box<dyn application::StyleSheet<Style = Theme>>),
}

impl Application {
    /// Creates a custom [`Application`] style.
    pub fn custom(
        custom: impl application::StyleSheet<Style = Theme> + 'static,
    ) -> Self {
        Self::Custom(Box::new(custom))
    }
}

impl application::StyleSheet for Theme {
    type Style = Application;

    fn appearance(&self, style: &Self::Style) -> application::Appearance {
        let palette = self.extended_palette();

        match style {
            Application::Default => application::Appearance {
                background_color: palette.background.base.color,
                text_color: palette.background.base.text,
            },
            Application::Custom(custom) => custom.appearance(self),
        }
    }
}

impl<T: Fn(&Theme) -> application::Appearance> application::StyleSheet for T {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> application::Appearance {
        (self)(style)
    }
}

/// The style of a button.
#[derive(Default)]
pub enum Button {
    /// The primary style.
    #[default]
    Primary,
    /// The secondary style.
    Secondary,
    /// The positive style.
    Positive,
    /// The destructive style.
    Destructive,
    /// The text style.
    ///
    /// Useful for links!
    Text,
    /// A custom style.
    Custom(Box<dyn button::StyleSheet<Style = Theme>>),
}

impl Button {
    /// Creates a custom [`Button`] style variant.
    pub fn custom(
        style_sheet: impl button::StyleSheet<Style = Theme> + 'static,
    ) -> Self {
        Self::Custom(Box::new(style_sheet))
    }
}

impl button::StyleSheet for Theme {
    type Style = Button;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let palette = self.extended_palette();

        let appearance = button::Appearance {
            border: Border::with_radius(2),
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
            Button::Custom(custom) => custom.active(self),
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let palette = self.extended_palette();

        if let Button::Custom(custom) = style {
            return custom.hovered(self);
        }

        let active = self.active(style);

        let background = match style {
            Button::Primary => Some(palette.primary.base.color),
            Button::Secondary => Some(palette.background.strong.color),
            Button::Positive => Some(palette.success.strong.color),
            Button::Destructive => Some(palette.danger.strong.color),
            Button::Text | Button::Custom(_) => None,
        };

        button::Appearance {
            background: background.map(Background::from),
            ..active
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        if let Button::Custom(custom) = style {
            return custom.pressed(self);
        }

        button::Appearance {
            shadow_offset: Vector::default(),
            ..self.active(style)
        }
    }

    fn disabled(&self, style: &Self::Style) -> button::Appearance {
        if let Button::Custom(custom) = style {
            return custom.disabled(self);
        }

        let active = self.active(style);

        button::Appearance {
            shadow_offset: Vector::default(),
            background: active.background.map(|background| match background {
                Background::Color(color) => Background::Color(Color {
                    a: color.a * 0.5,
                    ..color
                }),
                Background::Gradient(gradient) => {
                    Background::Gradient(gradient.mul_alpha(0.5))
                }
            }),
            text_color: Color {
                a: active.text_color.a * 0.5,
                ..active.text_color
            },
            ..active
        }
    }
}

/// The style of a checkbox.
#[derive(Default)]
pub enum Checkbox {
    /// The primary style.
    #[default]
    Primary,
    /// The secondary style.
    Secondary,
    /// The success style.
    Success,
    /// The danger style.
    Danger,
    /// A custom style.
    Custom(Box<dyn checkbox::StyleSheet<Style = Theme>>),
}

impl checkbox::StyleSheet for Theme {
    type Style = Checkbox;

    fn active(
        &self,
        style: &Self::Style,
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
                palette.background.strong,
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
            Checkbox::Custom(custom) => custom.active(self, is_checked),
        }
    }

    fn hovered(
        &self,
        style: &Self::Style,
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
                palette.background.strong,
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
            Checkbox::Custom(custom) => custom.hovered(self, is_checked),
        }
    }

    fn disabled(
        &self,
        style: &Self::Style,
        is_checked: bool,
    ) -> checkbox::Appearance {
        let palette = self.extended_palette();

        match style {
            Checkbox::Primary => checkbox_appearance(
                palette.primary.strong.text,
                palette.background.weak,
                palette.background.strong,
                is_checked,
            ),
            Checkbox::Secondary => checkbox_appearance(
                palette.background.strong.color,
                palette.background.weak,
                palette.background.weak,
                is_checked,
            ),
            Checkbox::Success => checkbox_appearance(
                palette.success.base.text,
                palette.background.weak,
                palette.success.weak,
                is_checked,
            ),
            Checkbox::Danger => checkbox_appearance(
                palette.danger.base.text,
                palette.background.weak,
                palette.danger.weak,
                is_checked,
            ),
            Checkbox::Custom(custom) => custom.active(self, is_checked),
        }
    }
}

fn checkbox_appearance(
    icon_color: Color,
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
        icon_color,
        border: Border {
            radius: 2.0.into(),
            width: 1.0,
            color: accent.color,
        },
        text_color: None,
    }
}

/// The style of a container.
#[derive(Default)]
pub enum Container {
    /// No style.
    #[default]
    Transparent,
    /// A simple box.
    Box,
    /// A custom style.
    Custom(Box<dyn container::StyleSheet<Style = Theme>>),
}

impl From<container::Appearance> for Container {
    fn from(appearance: container::Appearance) -> Self {
        Self::Custom(Box::new(move |_: &_| appearance))
    }
}

impl<T: Fn(&Theme) -> container::Appearance + 'static> From<T> for Container {
    fn from(f: T) -> Self {
        Self::Custom(Box::new(f))
    }
}

impl container::StyleSheet for Theme {
    type Style = Container;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        match style {
            Container::Transparent => container::Appearance::default(),
            Container::Box => {
                let palette = self.extended_palette();

                container::Appearance {
                    text_color: None,
                    background: Some(palette.background.weak.color.into()),
                    border: Border::with_radius(2),
                    shadow: Shadow::default(),
                }
            }
            Container::Custom(custom) => custom.appearance(self),
        }
    }
}

impl<T: Fn(&Theme) -> container::Appearance> container::StyleSheet for T {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        (self)(style)
    }
}

/// The style of a slider.
#[derive(Default)]
pub enum Slider {
    /// The default style.
    #[default]
    Default,
    /// A custom style.
    Custom(Box<dyn slider::StyleSheet<Style = Theme>>),
}

impl slider::StyleSheet for Theme {
    type Style = Slider;

    fn active(&self, style: &Self::Style) -> slider::Appearance {
        match style {
            Slider::Default => {
                let palette = self.extended_palette();

                let handle = slider::Handle {
                    shape: slider::HandleShape::Rectangle {
                        width: 8,
                        border_radius: 4.0.into(),
                    },
                    color: Color::WHITE,
                    border_color: Color::WHITE,
                    border_width: 1.0,
                };

                slider::Appearance {
                    rail: slider::Rail {
                        colors: (
                            palette.primary.base.color,
                            palette.secondary.base.color,
                        ),
                        width: 4.0,
                        border_radius: 2.0.into(),
                    },
                    handle: slider::Handle {
                        color: palette.background.base.color,
                        border_color: palette.primary.base.color,
                        ..handle
                    },
                }
            }
            Slider::Custom(custom) => custom.active(self),
        }
    }

    fn hovered(&self, style: &Self::Style) -> slider::Appearance {
        match style {
            Slider::Default => {
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
            Slider::Custom(custom) => custom.hovered(self),
        }
    }

    fn dragging(&self, style: &Self::Style) -> slider::Appearance {
        match style {
            Slider::Default => {
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
            Slider::Custom(custom) => custom.dragging(self),
        }
    }
}

/// The style of a menu.
#[derive(Clone, Default)]
pub enum Menu {
    /// The default style.
    #[default]
    Default,
    /// A custom style.
    Custom(Rc<dyn menu::StyleSheet<Style = Theme>>),
}

impl menu::StyleSheet for Theme {
    type Style = Menu;

    fn appearance(&self, style: &Self::Style) -> menu::Appearance {
        match style {
            Menu::Default => {
                let palette = self.extended_palette();

                menu::Appearance {
                    text_color: palette.background.weak.text,
                    background: palette.background.weak.color.into(),
                    border: Border {
                        width: 1.0,
                        radius: 0.0.into(),
                        color: palette.background.strong.color,
                    },
                    selected_text_color: palette.primary.strong.text,
                    selected_background: palette.primary.strong.color.into(),
                }
            }
            Menu::Custom(custom) => custom.appearance(self),
        }
    }
}

impl From<PickList> for Menu {
    fn from(pick_list: PickList) -> Self {
        match pick_list {
            PickList::Default => Self::Default,
            PickList::Custom(_, menu) => Self::Custom(menu),
        }
    }
}

/// The style of a pick list.
#[derive(Clone, Default)]
pub enum PickList {
    /// The default style.
    #[default]
    Default,
    /// A custom style.
    Custom(
        Rc<dyn pick_list::StyleSheet<Style = Theme>>,
        Rc<dyn menu::StyleSheet<Style = Theme>>,
    ),
}

impl pick_list::StyleSheet for Theme {
    type Style = PickList;

    fn active(&self, style: &Self::Style) -> pick_list::Appearance {
        match style {
            PickList::Default => {
                let palette = self.extended_palette();

                pick_list::Appearance {
                    text_color: palette.background.weak.text,
                    background: palette.background.weak.color.into(),
                    placeholder_color: palette.background.strong.color,
                    handle_color: palette.background.weak.text,
                    border: Border {
                        radius: 2.0.into(),
                        width: 1.0,
                        color: palette.background.strong.color,
                    },
                }
            }
            PickList::Custom(custom, _) => custom.active(self),
        }
    }

    fn hovered(&self, style: &Self::Style) -> pick_list::Appearance {
        match style {
            PickList::Default => {
                let palette = self.extended_palette();

                pick_list::Appearance {
                    text_color: palette.background.weak.text,
                    background: palette.background.weak.color.into(),
                    placeholder_color: palette.background.strong.color,
                    handle_color: palette.background.weak.text,
                    border: Border {
                        radius: 2.0.into(),
                        width: 1.0,
                        color: palette.primary.strong.color,
                    },
                }
            }
            PickList::Custom(custom, _) => custom.hovered(self),
        }
    }
}

/// The style of a radio button.
#[derive(Default)]
pub enum Radio {
    /// The default style.
    #[default]
    Default,
    /// A custom style.
    Custom(Box<dyn radio::StyleSheet<Style = Theme>>),
}

impl radio::StyleSheet for Theme {
    type Style = Radio;

    fn active(
        &self,
        style: &Self::Style,
        is_selected: bool,
    ) -> radio::Appearance {
        match style {
            Radio::Default => {
                let palette = self.extended_palette();

                radio::Appearance {
                    background: Color::TRANSPARENT.into(),
                    dot_color: palette.primary.strong.color,
                    border_width: 1.0,
                    border_color: palette.primary.strong.color,
                    text_color: None,
                }
            }
            Radio::Custom(custom) => custom.active(self, is_selected),
        }
    }

    fn hovered(
        &self,
        style: &Self::Style,
        is_selected: bool,
    ) -> radio::Appearance {
        match style {
            Radio::Default => {
                let active = self.active(style, is_selected);
                let palette = self.extended_palette();

                radio::Appearance {
                    dot_color: palette.primary.strong.color,
                    background: palette.primary.weak.color.into(),
                    ..active
                }
            }
            Radio::Custom(custom) => custom.hovered(self, is_selected),
        }
    }
}

/// The style of a toggler.
#[derive(Default)]
pub enum Toggler {
    /// The default style.
    #[default]
    Default,
    /// A custom style.
    Custom(Box<dyn toggler::StyleSheet<Style = Theme>>),
}

impl toggler::StyleSheet for Theme {
    type Style = Toggler;

    fn active(
        &self,
        style: &Self::Style,
        is_active: bool,
    ) -> toggler::Appearance {
        match style {
            Toggler::Default => {
                let palette = self.extended_palette();

                toggler::Appearance {
                    background: if is_active {
                        palette.primary.strong.color
                    } else {
                        palette.background.strong.color
                    },
                    background_border_width: 0.0,
                    background_border_color: Color::TRANSPARENT,
                    foreground: if is_active {
                        palette.primary.strong.text
                    } else {
                        palette.background.base.color
                    },
                    foreground_border_width: 0.0,
                    foreground_border_color: Color::TRANSPARENT,
                }
            }
            Toggler::Custom(custom) => custom.active(self, is_active),
        }
    }

    fn hovered(
        &self,
        style: &Self::Style,
        is_active: bool,
    ) -> toggler::Appearance {
        match style {
            Toggler::Default => {
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
            Toggler::Custom(custom) => custom.hovered(self, is_active),
        }
    }
}

/// The style of a pane grid.
#[derive(Default)]
pub enum PaneGrid {
    /// The default style.
    #[default]
    Default,
    /// A custom style.
    Custom(Box<dyn pane_grid::StyleSheet<Style = Theme>>),
}

impl pane_grid::StyleSheet for Theme {
    type Style = PaneGrid;

    fn hovered_region(&self, style: &Self::Style) -> pane_grid::Appearance {
        match style {
            PaneGrid::Default => {
                let palette = self.extended_palette();

                pane_grid::Appearance {
                    background: Background::Color(Color {
                        a: 0.5,
                        ..palette.primary.base.color
                    }),
                    border: Border {
                        width: 2.0,
                        color: palette.primary.strong.color,
                        radius: 0.0.into(),
                    },
                }
            }
            PaneGrid::Custom(custom) => custom.hovered_region(self),
        }
    }

    fn picked_split(&self, style: &Self::Style) -> Option<pane_grid::Line> {
        match style {
            PaneGrid::Default => {
                let palette = self.extended_palette();

                Some(pane_grid::Line {
                    color: palette.primary.strong.color,
                    width: 2.0,
                })
            }
            PaneGrid::Custom(custom) => custom.picked_split(self),
        }
    }

    fn hovered_split(&self, style: &Self::Style) -> Option<pane_grid::Line> {
        match style {
            PaneGrid::Default => {
                let palette = self.extended_palette();

                Some(pane_grid::Line {
                    color: palette.primary.base.color,
                    width: 2.0,
                })
            }
            PaneGrid::Custom(custom) => custom.hovered_split(self),
        }
    }
}

/// The style of a progress bar.
#[derive(Default)]
pub enum ProgressBar {
    /// The primary style.
    #[default]
    Primary,
    /// The success style.
    Success,
    /// The danger style.
    Danger,
    /// A custom style.
    Custom(Box<dyn progress_bar::StyleSheet<Style = Theme>>),
}

impl<T: Fn(&Theme) -> progress_bar::Appearance + 'static> From<T>
    for ProgressBar
{
    fn from(f: T) -> Self {
        Self::Custom(Box::new(f))
    }
}

impl progress_bar::StyleSheet for Theme {
    type Style = ProgressBar;

    fn appearance(&self, style: &Self::Style) -> progress_bar::Appearance {
        if let ProgressBar::Custom(custom) = style {
            return custom.appearance(self);
        }

        let palette = self.extended_palette();

        let from_palette = |bar: Color| progress_bar::Appearance {
            background: palette.background.strong.color.into(),
            bar: bar.into(),
            border_radius: 2.0.into(),
        };

        match style {
            ProgressBar::Primary => from_palette(palette.primary.base.color),
            ProgressBar::Success => from_palette(palette.success.base.color),
            ProgressBar::Danger => from_palette(palette.danger.base.color),
            ProgressBar::Custom(custom) => custom.appearance(self),
        }
    }
}

impl<T: Fn(&Theme) -> progress_bar::Appearance> progress_bar::StyleSheet for T {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> progress_bar::Appearance {
        (self)(style)
    }
}

/// The style of a QR Code.
#[derive(Default)]
pub enum QRCode {
    /// The default style.
    #[default]
    Default,
    /// A custom style.
    Custom(Box<dyn qr_code::StyleSheet<Style = Theme>>),
}

impl<T: Fn(&Theme) -> qr_code::Appearance + 'static> From<T> for QRCode {
    fn from(f: T) -> Self {
        Self::Custom(Box::new(f))
    }
}

impl qr_code::StyleSheet for Theme {
    type Style = QRCode;

    fn appearance(&self, style: &Self::Style) -> qr_code::Appearance {
        let palette = self.palette();

        match style {
            QRCode::Default => qr_code::Appearance {
                cell: palette.text,
                background: palette.background,
            },
            QRCode::Custom(custom) => custom.appearance(self),
        }
    }
}

impl<T: Fn(&Theme) -> qr_code::Appearance> qr_code::StyleSheet for T {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> qr_code::Appearance {
        (self)(style)
    }
}

/// The style of a rule.
#[derive(Default)]
pub enum Rule {
    /// The default style.
    #[default]
    Default,
    /// A custom style.
    Custom(Box<dyn rule::StyleSheet<Style = Theme>>),
}

impl<T: Fn(&Theme) -> rule::Appearance + 'static> From<T> for Rule {
    fn from(f: T) -> Self {
        Self::Custom(Box::new(f))
    }
}

impl rule::StyleSheet for Theme {
    type Style = Rule;

    fn appearance(&self, style: &Self::Style) -> rule::Appearance {
        let palette = self.extended_palette();

        match style {
            Rule::Default => rule::Appearance {
                color: palette.background.strong.color,
                width: 1,
                radius: 0.0.into(),
                fill_mode: rule::FillMode::Full,
            },
            Rule::Custom(custom) => custom.appearance(self),
        }
    }
}

impl<T: Fn(&Theme) -> rule::Appearance> rule::StyleSheet for T {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> rule::Appearance {
        (self)(style)
    }
}

/**
 * Svg
 */
#[derive(Default)]
pub enum Svg {
    /// No filtering to the rendered SVG.
    #[default]
    Default,
    /// A custom style.
    Custom(Box<dyn svg::StyleSheet<Style = Theme>>),
}

impl Svg {
    /// Creates a custom [`Svg`] style.
    pub fn custom_fn(f: fn(&Theme) -> svg::Appearance) -> Self {
        Self::Custom(Box::new(f))
    }
}

impl svg::StyleSheet for Theme {
    type Style = Svg;

    fn appearance(&self, style: &Self::Style) -> svg::Appearance {
        match style {
            Svg::Default => svg::Appearance::default(),
            Svg::Custom(custom) => custom.appearance(self),
        }
    }

    fn hovered(&self, style: &Self::Style) -> svg::Appearance {
        self.appearance(style)
    }
}

impl svg::StyleSheet for fn(&Theme) -> svg::Appearance {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> svg::Appearance {
        (self)(style)
    }

    fn hovered(&self, style: &Self::Style) -> svg::Appearance {
        self.appearance(style)
    }
}

/// The style of a scrollable.
#[derive(Default)]
pub enum Scrollable {
    /// The default style.
    #[default]
    Default,
    /// A custom style.
    Custom(Box<dyn scrollable::StyleSheet<Style = Theme>>),
}

impl Scrollable {
    /// Creates a custom [`Scrollable`] theme.
    pub fn custom<T: scrollable::StyleSheet<Style = Theme> + 'static>(
        style: T,
    ) -> Self {
        Self::Custom(Box::new(style))
    }
}

impl scrollable::StyleSheet for Theme {
    type Style = Scrollable;

    fn active(&self, style: &Self::Style) -> scrollable::Appearance {
        match style {
            Scrollable::Default => {
                let palette = self.extended_palette();

                scrollable::Appearance {
                    container: container::Appearance::default(),
                    scrollbar: scrollable::Scrollbar {
                        background: Some(palette.background.weak.color.into()),
                        border: Border::with_radius(2),
                        scroller: scrollable::Scroller {
                            color: palette.background.strong.color,
                            border: Border::with_radius(2),
                        },
                    },
                    gap: None,
                }
            }
            Scrollable::Custom(custom) => custom.active(self),
        }
    }

    fn hovered(
        &self,
        style: &Self::Style,
        is_mouse_over_scrollbar: bool,
    ) -> scrollable::Appearance {
        match style {
            Scrollable::Default => {
                if is_mouse_over_scrollbar {
                    let palette = self.extended_palette();

                    scrollable::Appearance {
                        scrollbar: scrollable::Scrollbar {
                            background: Some(
                                palette.background.weak.color.into(),
                            ),
                            border: Border::with_radius(2),
                            scroller: scrollable::Scroller {
                                color: palette.primary.strong.color,
                                border: Border::with_radius(2),
                            },
                        },
                        ..self.active(style)
                    }
                } else {
                    self.active(style)
                }
            }
            Scrollable::Custom(custom) => {
                custom.hovered(self, is_mouse_over_scrollbar)
            }
        }
    }

    fn dragging(&self, style: &Self::Style) -> scrollable::Appearance {
        match style {
            Scrollable::Default => self.hovered(style, true),
            Scrollable::Custom(custom) => custom.dragging(self),
        }
    }
}

/// The style of text.
#[derive(Clone, Copy, Default)]
pub enum Text {
    /// The default style.
    #[default]
    Default,
    /// Colored text.
    Color(Color),
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
            Text::Default => text::Appearance::default(),
            Text::Color(c) => text::Appearance { color: Some(c) },
        }
    }
}

/// The style of a text input.
#[derive(Default)]
pub enum TextInput {
    /// The default style.
    #[default]
    Default,
    /// A custom style.
    Custom(Box<dyn text_input::StyleSheet<Style = Theme>>),
}

impl text_input::StyleSheet for Theme {
    type Style = TextInput;

    fn active(&self, style: &Self::Style) -> text_input::Appearance {
        if let TextInput::Custom(custom) = style {
            return custom.active(self);
        }

        let palette = self.extended_palette();

        text_input::Appearance {
            background: palette.background.base.color.into(),
            border: Border {
                radius: 2.0.into(),
                width: 1.0,
                color: palette.background.strong.color,
            },
            icon_color: palette.background.weak.text,
        }
    }

    fn hovered(&self, style: &Self::Style) -> text_input::Appearance {
        if let TextInput::Custom(custom) = style {
            return custom.hovered(self);
        }

        let palette = self.extended_palette();

        text_input::Appearance {
            background: palette.background.base.color.into(),
            border: Border {
                radius: 2.0.into(),
                width: 1.0,
                color: palette.background.base.text,
            },
            icon_color: palette.background.weak.text,
        }
    }

    fn focused(&self, style: &Self::Style) -> text_input::Appearance {
        if let TextInput::Custom(custom) = style {
            return custom.focused(self);
        }

        let palette = self.extended_palette();

        text_input::Appearance {
            background: palette.background.base.color.into(),
            border: Border {
                radius: 2.0.into(),
                width: 1.0,
                color: palette.primary.strong.color,
            },
            icon_color: palette.background.weak.text,
        }
    }

    fn placeholder_color(&self, style: &Self::Style) -> Color {
        if let TextInput::Custom(custom) = style {
            return custom.placeholder_color(self);
        }

        let palette = self.extended_palette();

        palette.background.strong.color
    }

    fn value_color(&self, style: &Self::Style) -> Color {
        if let TextInput::Custom(custom) = style {
            return custom.value_color(self);
        }

        let palette = self.extended_palette();

        palette.background.base.text
    }

    fn selection_color(&self, style: &Self::Style) -> Color {
        if let TextInput::Custom(custom) = style {
            return custom.selection_color(self);
        }

        let palette = self.extended_palette();

        palette.primary.weak.color
    }

    fn disabled(&self, style: &Self::Style) -> text_input::Appearance {
        if let TextInput::Custom(custom) = style {
            return custom.disabled(self);
        }

        let palette = self.extended_palette();

        text_input::Appearance {
            background: palette.background.weak.color.into(),
            border: Border {
                radius: 2.0.into(),
                width: 1.0,
                color: palette.background.strong.color,
            },
            icon_color: palette.background.strong.color,
        }
    }

    fn disabled_color(&self, style: &Self::Style) -> Color {
        if let TextInput::Custom(custom) = style {
            return custom.disabled_color(self);
        }

        self.placeholder_color(style)
    }
}

/// The style of a text input.
#[derive(Default)]
pub enum TextEditor {
    /// The default style.
    #[default]
    Default,
    /// A custom style.
    Custom(Box<dyn text_editor::StyleSheet<Style = Theme>>),
}

impl text_editor::StyleSheet for Theme {
    type Style = TextEditor;

    fn active(&self, style: &Self::Style) -> text_editor::Appearance {
        if let TextEditor::Custom(custom) = style {
            return custom.active(self);
        }

        let palette = self.extended_palette();

        text_editor::Appearance {
            background: palette.background.base.color.into(),
            border: Border {
                radius: 2.0.into(),
                width: 1.0,
                color: palette.background.strong.color,
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> text_editor::Appearance {
        if let TextEditor::Custom(custom) = style {
            return custom.hovered(self);
        }

        let palette = self.extended_palette();

        text_editor::Appearance {
            background: palette.background.base.color.into(),
            border: Border {
                radius: 2.0.into(),
                width: 1.0,
                color: palette.background.base.text,
            },
        }
    }

    fn focused(&self, style: &Self::Style) -> text_editor::Appearance {
        if let TextEditor::Custom(custom) = style {
            return custom.focused(self);
        }

        let palette = self.extended_palette();

        text_editor::Appearance {
            background: palette.background.base.color.into(),
            border: Border {
                radius: 2.0.into(),
                width: 1.0,
                color: palette.primary.strong.color,
            },
        }
    }

    fn placeholder_color(&self, style: &Self::Style) -> Color {
        if let TextEditor::Custom(custom) = style {
            return custom.placeholder_color(self);
        }

        let palette = self.extended_palette();

        palette.background.strong.color
    }

    fn value_color(&self, style: &Self::Style) -> Color {
        if let TextEditor::Custom(custom) = style {
            return custom.value_color(self);
        }

        let palette = self.extended_palette();

        palette.background.base.text
    }

    fn selection_color(&self, style: &Self::Style) -> Color {
        if let TextEditor::Custom(custom) = style {
            return custom.selection_color(self);
        }

        let palette = self.extended_palette();

        palette.primary.weak.color
    }

    fn disabled(&self, style: &Self::Style) -> text_editor::Appearance {
        if let TextEditor::Custom(custom) = style {
            return custom.disabled(self);
        }

        let palette = self.extended_palette();

        text_editor::Appearance {
            background: palette.background.weak.color.into(),
            border: Border {
                radius: 2.0.into(),
                width: 1.0,
                color: palette.background.strong.color,
            },
        }
    }

    fn disabled_color(&self, style: &Self::Style) -> Color {
        if let TextEditor::Custom(custom) = style {
            return custom.disabled_color(self);
        }

        self.placeholder_color(style)
    }
}
