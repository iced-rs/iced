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

    /// Creates a new custom [`Theme`] from the given [`Seed`](palette::Seed).
    pub fn custom(name: impl Into<Cow<'static, str>>, seed: palette::Seed) -> Self {
        Self::custom_with_fn(name, seed, Palette::generate)
    }

    /// Creates a new custom [`Theme`] from the given [`Seed`](palette::Seed), with
    /// a custom generator of a [`Palette`].
    pub fn custom_with_fn(
        name: impl Into<Cow<'static, str>>,
        palette: palette::Seed,
        generate: impl FnOnce(palette::Seed) -> Palette,
    ) -> Self {
        Self::Custom(Arc::new(Custom::with_fn(name, palette, generate)))
    }

    /// Returns the [`Palette`] of the [`Theme`].
    pub fn palette(&self) -> &palette::Palette {
        match self {
            Self::Light => &palette::LIGHT,
            Self::Dark => &palette::DARK,
            Self::Dracula => &palette::DRACULA,
            Self::Nord => &palette::NORD,
            Self::SolarizedLight => &palette::SOLARIZED_LIGHT,
            Self::SolarizedDark => &palette::SOLARIZED_DARK,
            Self::GruvboxLight => &palette::GRUVBOX_LIGHT,
            Self::GruvboxDark => &palette::GRUVBOX_DARK,
            Self::CatppuccinLatte => &palette::CATPPUCCIN_LATTE,
            Self::CatppuccinFrappe => &palette::CATPPUCCIN_FRAPPE,
            Self::CatppuccinMacchiato => &palette::CATPPUCCIN_MACCHIATO,
            Self::CatppuccinMocha => &palette::CATPPUCCIN_MOCHA,
            Self::TokyoNight => &palette::TOKYO_NIGHT,
            Self::TokyoNightStorm => &palette::TOKYO_NIGHT_STORM,
            Self::TokyoNightLight => &palette::TOKYO_NIGHT_LIGHT,
            Self::KanagawaWave => &palette::KANAGAWA_WAVE,
            Self::KanagawaDragon => &palette::KANAGAWA_DRAGON,
            Self::KanagawaLotus => &palette::KANAGAWA_LOTUS,
            Self::Moonfly => &palette::MOONFLY,
            Self::Nightfly => &palette::NIGHTFLY,
            Self::Oxocarbon => &palette::OXOCARBON,
            Self::Ferra => &palette::FERRA,
            Self::Custom(custom) => &custom.palette,
        }
    }

    /// Returns the [`Seed`](palette::Seed) of the [`Theme`].
    pub fn seed(&self) -> palette::Seed {
        match self {
            Self::Light => palette::Seed::LIGHT,
            Self::Dark => palette::Seed::DARK,
            Self::Dracula => palette::Seed::DRACULA,
            Self::Nord => palette::Seed::NORD,
            Self::SolarizedLight => palette::Seed::SOLARIZED_LIGHT,
            Self::SolarizedDark => palette::Seed::SOLARIZED_DARK,
            Self::GruvboxLight => palette::Seed::GRUVBOX_LIGHT,
            Self::GruvboxDark => palette::Seed::GRUVBOX_DARK,
            Self::CatppuccinLatte => palette::Seed::CATPPUCCIN_LATTE,
            Self::CatppuccinFrappe => palette::Seed::CATPPUCCIN_FRAPPE,
            Self::CatppuccinMacchiato => palette::Seed::CATPPUCCIN_MACCHIATO,
            Self::CatppuccinMocha => palette::Seed::CATPPUCCIN_MOCHA,
            Self::TokyoNight => palette::Seed::TOKYO_NIGHT,
            Self::TokyoNightStorm => palette::Seed::TOKYO_NIGHT_STORM,
            Self::TokyoNightLight => palette::Seed::TOKYO_NIGHT_LIGHT,
            Self::KanagawaWave => palette::Seed::KANAGAWA_WAVE,
            Self::KanagawaDragon => palette::Seed::KANAGAWA_DRAGON,
            Self::KanagawaLotus => palette::Seed::KANAGAWA_LOTUS,
            Self::Moonfly => palette::Seed::MOONFLY,
            Self::Nightfly => palette::Seed::NIGHTFLY,
            Self::Oxocarbon => palette::Seed::OXOCARBON,
            Self::Ferra => palette::Seed::FERRA,
            Self::Custom(custom) => custom.seed,
        }
    }
}

impl fmt::Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A [`Theme`] with a customized [`Palette`].
#[derive(Debug, Clone, PartialEq)]
pub struct Custom {
    name: Cow<'static, str>,
    seed: palette::Seed,
    palette: Palette,
}

impl Custom {
    /// Creates a [`Custom`] theme from the given [`Seed`](palette::Seed).
    pub fn new(name: impl Into<Cow<'static, str>>, seed: palette::Seed) -> Self {
        Self::with_fn(name, seed, Palette::generate)
    }

    /// Creates a [`Custom`] theme from the given [`Seed`](palette::Seed) with
    /// a custom generator of a [`Palette`].
    pub fn with_fn(
        name: impl Into<Cow<'static, str>>,
        seed: palette::Seed,
        generate: impl FnOnce(palette::Seed) -> Palette,
    ) -> Self {
        Self {
            name: name.into(),
            seed,
            palette: generate(seed),
        }
    }
}

impl fmt::Display for Custom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// A theme mode, denoting the tone or brightness of a theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// No specific tone.
    #[default]
    None,
    /// A mode referring to themes with light tones.
    Light,
    /// A mode referring to themes with dark tones.
    Dark,
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
    /// Returns the default theme for the preferred [`Mode`].
    fn default(preference: Mode) -> Self;

    /// Returns the [`Mode`] of the theme.
    fn mode(&self) -> Mode;

    /// Returns the default base [`Style`] of the theme.
    fn base(&self) -> Style;

    /// Returns the [`Seed`](palette::Seed) of the theme.
    ///
    /// This may be used by the runtime to recreate a [`Theme`] for
    /// debugging purposes; like displaying performance metrics or devtools.
    fn seed(&self) -> Option<palette::Seed>;

    /// Returns the unique name of the theme.
    ///
    /// This name may be used to efficiently detect theme
    /// changes in some widgets.
    fn name(&self) -> &str;
}

impl Base for Theme {
    fn default(preference: Mode) -> Self {
        use std::env;
        use std::sync::OnceLock;

        static SYSTEM: OnceLock<Option<Theme>> = OnceLock::new();

        let system = SYSTEM.get_or_init(|| {
            let name = env::var("ICED_THEME").ok()?;

            Theme::ALL
                .iter()
                .find(|theme| theme.to_string() == name)
                .cloned()
        });

        if let Some(system) = system {
            return system.clone();
        }

        match preference {
            Mode::None | Mode::Light => Self::Light,
            Mode::Dark => Self::Dark,
        }
    }

    fn mode(&self) -> Mode {
        if self.palette().is_dark {
            Mode::Dark
        } else {
            Mode::Light
        }
    }

    fn base(&self) -> Style {
        default(self)
    }

    fn seed(&self) -> Option<palette::Seed> {
        Some(self.seed())
    }

    fn name(&self) -> &str {
        match self {
            Self::Light => "Light",
            Self::Dark => "Dark",
            Self::Dracula => "Dracula",
            Self::Nord => "Nord",
            Self::SolarizedLight => "Solarized Light",
            Self::SolarizedDark => "Solarized Dark",
            Self::GruvboxLight => "Gruvbox Light",
            Self::GruvboxDark => "Gruvbox Dark",
            Self::CatppuccinLatte => "Catppuccin Latte",
            Self::CatppuccinFrappe => "Catppuccin Frappé",
            Self::CatppuccinMacchiato => "Catppuccin Macchiato",
            Self::CatppuccinMocha => "Catppuccin Mocha",
            Self::TokyoNight => "Tokyo Night",
            Self::TokyoNightStorm => "Tokyo Night Storm",
            Self::TokyoNightLight => "Tokyo Night Light",
            Self::KanagawaWave => "Kanagawa Wave",
            Self::KanagawaDragon => "Kanagawa Dragon",
            Self::KanagawaLotus => "Kanagawa Lotus",
            Self::Moonfly => "Moonfly",
            Self::Nightfly => "Nightfly",
            Self::Oxocarbon => "Oxocarbon",
            Self::Ferra => "Ferra",
            Self::Custom(custom) => &custom.name,
        }
    }
}

/// The default [`Style`] of a built-in [`Theme`].
pub fn default(theme: &Theme) -> Style {
    let palette = theme.palette();

    Style {
        background_color: palette.background.base.color,
        text_color: palette.background.base.text,
    }
}
