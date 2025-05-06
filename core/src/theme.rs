//! Use the built-in theme and styles.
pub mod palette;

pub use palette::Palette;

use crate::Color;

use std::borrow::Cow;
use std::fmt;
use std::sync::Arc;

/// A built-in theme.
#[derive(Debug, Clone, PartialEq)]
pub enum Theme {
    /// The built-in light variant.
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
    /// The built-in Ferra variant:
    Ferra,
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
        Self::Ferra,
    ];

    /// Creates a new custom [`Theme`] from the given [`Palette`].
    pub fn custom(
        name: impl Into<Cow<'static, str>>,
        palette: Palette,
    ) -> Self {
        Self::custom_with_fn(name, palette, palette::Extended::generate)
    }

    /// Creates a new custom [`Theme`] from the given [`Palette`], with
    /// a custom generator of a [`palette::Extended`].
    pub fn custom_with_fn(
        name: impl Into<Cow<'static, str>>,
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
            Self::Ferra => Palette::FERRA,
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
            Self::Ferra => &palette::EXTENDED_FERRA,
            Self::Custom(custom) => &custom.extended,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        #[cfg(feature = "auto-detect-theme")]
        {
            use std::sync::LazyLock;

            static DEFAULT: LazyLock<Theme> = LazyLock::new(|| {
                match dark_light::detect()
                    .unwrap_or(dark_light::Mode::Unspecified)
                {
                    dark_light::Mode::Dark => Theme::Dark,
                    dark_light::Mode::Light | dark_light::Mode::Unspecified => {
                        Theme::Light
                    }
                }
            });

            DEFAULT.clone()
        }

        #[cfg(not(feature = "auto-detect-theme"))]
        Theme::Light
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
            Self::Ferra => write!(f, "Ferra"),
            Self::Custom(custom) => custom.fmt(f),
        }
    }
}

/// A [`Theme`] with a customized [`Palette`].
#[derive(Debug, Clone, PartialEq)]
pub struct Custom {
    name: Cow<'static, str>,
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
        name: impl Into<Cow<'static, str>>,
        palette: Palette,
        generate: impl FnOnce(Palette) -> palette::Extended,
    ) -> Self {
        Self {
            name: name.into(),
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

/// The base style of a theme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The background [`Color`] of the application.
    pub background_color: Color,

    /// The default text [`Color`] of the application.
    pub text_color: Color,
}

/// The default blank style of a theme.
pub trait Base {
    /// Returns the default base [`Style`] of a theme.
    fn base(&self) -> Style;

    /// Returns the color [`Palette`] of the theme.
    ///
    /// This [`Palette`] may be used by the runtime for
    /// debugging purposes; like displaying performance
    /// metrics or devtools.
    fn palette(&self) -> Option<Palette>;
}

impl Base for Theme {
    fn base(&self) -> Style {
        default(self)
    }

    fn palette(&self) -> Option<Palette> {
        Some(self.palette())
    }
}

/// The default [`Style`] of a built-in [`Theme`].
pub fn default(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    Style {
        background_color: palette.background.base.color,
        text_color: palette.background.base.text,
    }
}
