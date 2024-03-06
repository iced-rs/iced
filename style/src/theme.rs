//! Use the built-in theme and styles.
pub mod palette;

pub use palette::Palette;

use crate::application;
use crate::core::widget::text;
use crate::menu;
use crate::pick_list;

use crate::core::Border;

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

impl text::StyleSheet for Theme {}
