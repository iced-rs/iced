//! Define the colors of a theme.
use crate::{Color, color};

use std::sync::LazyLock;

/// An extended set of colors generated from a [`Seed`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    /// The set of background colors.
    pub background: Background,
    /// The set of primary colors.
    pub primary: Swatch,
    /// The set of secondary colors.
    pub secondary: Swatch,
    /// The set of success colors.
    pub success: Swatch,
    /// The set of warning colors.
    pub warning: Swatch,
    /// The set of danger colors.
    pub danger: Swatch,
    /// Whether the palette is dark or not.
    pub is_dark: bool,
}

impl Palette {
    /// Generates a [`Palette`] from the given [`Seed`].
    pub fn generate(palette: Seed) -> Self {
        Self {
            background: Background::new(palette.background, palette.text),
            primary: Swatch::generate(palette.primary, palette.background, palette.text),
            secondary: Swatch::derive(palette.background, palette.text),
            success: Swatch::generate(palette.success, palette.background, palette.text),
            warning: Swatch::generate(palette.warning, palette.background, palette.text),
            danger: Swatch::generate(palette.danger, palette.background, palette.text),
            is_dark: is_dark(palette.background),
        }
    }
}

/// A pair of background and text colors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pair {
    /// The background color.
    pub color: Color,

    /// The text color.
    ///
    /// It's guaranteed to be readable on top of the background [`color`].
    ///
    /// [`color`]: Self::color
    pub text: Color,
}

impl Pair {
    /// Creates a new [`Pair`] from a background [`Color`] and some text [`Color`].
    pub fn new(color: Color, text: Color) -> Self {
        Self {
            color,
            text: readable(color, text),
        }
    }
}

/// A set of background colors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Background {
    /// The base background color.
    pub base: Pair,
    /// The weakest version of the base background color.
    pub weakest: Pair,
    /// A weaker version of the base background color.
    pub weaker: Pair,
    /// A weak version of the base background color.
    pub weak: Pair,
    /// A neutral version of the base background color, between weak and strong.
    pub neutral: Pair,
    /// A strong version of the base background color.
    pub strong: Pair,
    /// A stronger version of the base background color.
    pub stronger: Pair,
    /// The strongest version of the base background color.
    pub strongest: Pair,
}

impl Background {
    /// Generates a set of [`Background`] colors from the base and text colors.
    pub fn new(base: Color, text: Color) -> Self {
        let weakest = deviate(base, 0.03);
        let weaker = deviate(base, 0.07);
        let weak = deviate(base, 0.1);
        let neutral = deviate(base, 0.125);
        let strong = deviate(base, 0.15);
        let stronger = deviate(base, 0.175);
        let strongest = deviate(base, 0.20);

        Self {
            base: Pair::new(base, text),
            weakest: Pair::new(weakest, text),
            weaker: Pair::new(weaker, text),
            weak: Pair::new(weak, text),
            neutral: Pair::new(neutral, text),
            strong: Pair::new(strong, text),
            stronger: Pair::new(stronger, text),
            strongest: Pair::new(strongest, text),
        }
    }
}

/// A color sample in a palette of colors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Swatch {
    /// The base color.
    pub base: Pair,
    /// A weaker version of the base color.
    pub weak: Pair,
    /// A stronger version of the base color.
    pub strong: Pair,
}

impl Swatch {
    /// Generates a [`Swatch`] from a base, background and text color.
    pub fn generate(base: Color, background: Color, text: Color) -> Self {
        let weak = base.mix(background, 0.4);
        let strong = deviate(base, 0.1);

        Self {
            base: Pair::new(base, text),
            weak: Pair::new(weak, text),
            strong: Pair::new(strong, text),
        }
    }

    /// Derives a [`Swatch`] from a base color and text color.
    pub fn derive(base: Color, text: Color) -> Self {
        let factor = if is_dark(base) { 0.2 } else { 0.4 };

        let weak = deviate(base, 0.1).mix(text, factor);
        let strong = deviate(base, 0.3).mix(text, factor);
        let base = deviate(base, 0.2).mix(text, factor);

        Self {
            base: Pair::new(base, text),
            weak: Pair::new(weak, text),
            strong: Pair::new(strong, text),
        }
    }
}

/// The base set of colors of a [`Palette`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Seed {
    /// The background [`Color`] of the [`Palette`].
    pub background: Color,
    /// The text [`Color`] of the [`Palette`].
    pub text: Color,
    /// The primary [`Color`] of the [`Palette`].
    pub primary: Color,
    /// The success [`Color`] of the [`Palette`].
    pub success: Color,
    /// The warning [`Color`] of the [`Palette`].
    pub warning: Color,
    /// The danger [`Color`] of the [`Palette`].
    pub danger: Color,
}

impl Seed {
    /// The built-in light variant of a [`Palette`].
    pub const LIGHT: Self = Self {
        background: Color::WHITE,
        text: Color::BLACK,
        primary: color!(0x5865F2),
        success: color!(0x12664f),
        warning: color!(0xb77e33),
        danger: color!(0xc3423f),
    };

    /// The built-in dark variant of a [`Palette`].
    pub const DARK: Self = Self {
        background: color!(0x2B2D31),
        text: Color::from_rgb(0.90, 0.90, 0.90),
        primary: color!(0x5865F2),
        success: color!(0x12664f),
        warning: color!(0xffc14e),
        danger: color!(0xc3423f),
    };

    /// The built-in [Dracula] variant of a [`Palette`].
    ///
    /// [Dracula]: https://draculatheme.com
    pub const DRACULA: Self = Self {
        background: color!(0x282A36), // BACKGROUND
        text: color!(0xf8f8f2),       // FOREGROUND
        primary: color!(0xbd93f9),    // PURPLE
        success: color!(0x50fa7b),    // GREEN
        warning: color!(0xf1fa8c),    // YELLOW
        danger: color!(0xff5555),     // RED
    };

    /// The built-in [Nord] variant of a [`Palette`].
    ///
    /// [Nord]: https://www.nordtheme.com/docs/colors-and-palettes
    pub const NORD: Self = Self {
        background: color!(0x2e3440), // nord0
        text: color!(0xeceff4),       // nord6
        primary: color!(0x8fbcbb),    // nord7
        success: color!(0xa3be8c),    // nord14
        warning: color!(0xebcb8b),    // nord13
        danger: color!(0xbf616a),     // nord11
    };

    /// The built-in [Solarized] Light variant of a [`Palette`].
    ///
    /// [Solarized]: https://ethanschoonover.com/solarized
    pub const SOLARIZED_LIGHT: Self = Self {
        background: color!(0xfdf6e3), // base3
        text: color!(0x657b83),       // base00
        primary: color!(0x2aa198),    // cyan
        success: color!(0x859900),    // green
        warning: color!(0xb58900),    // yellow
        danger: color!(0xdc322f),     // red
    };

    /// The built-in [Solarized] Dark variant of a [`Palette`].
    ///
    /// [Solarized]: https://ethanschoonover.com/solarized
    pub const SOLARIZED_DARK: Self = Self {
        background: color!(0x002b36), // base03
        text: color!(0x839496),       // base0
        primary: color!(0x2aa198),    // cyan
        success: color!(0x859900),    // green
        warning: color!(0xb58900),    // yellow
        danger: color!(0xdc322f),     // red
    };

    /// The built-in [Gruvbox] Light variant of a [`Palette`].
    ///
    /// [Gruvbox]: https://github.com/morhetz/gruvbox
    pub const GRUVBOX_LIGHT: Self = Self {
        background: color!(0xfbf1c7), // light BG_0
        text: color!(0x282828),       // light FG0_29
        primary: color!(0x458588),    // light BLUE_4
        success: color!(0x98971a),    // light GREEN_2
        warning: color!(0xd79921),    // light YELLOW_3
        danger: color!(0xcc241d),     // light RED_1
    };

    /// The built-in [Gruvbox] Dark variant of a [`Palette`].
    ///
    /// [Gruvbox]: https://github.com/morhetz/gruvbox
    pub const GRUVBOX_DARK: Self = Self {
        background: color!(0x282828), // dark BG_0
        text: color!(0xfbf1c7),       // dark FG0_29
        primary: color!(0x458588),    // dark BLUE_4
        success: color!(0x98971a),    // dark GREEN_2
        warning: color!(0xd79921),    // dark YELLOW_3
        danger: color!(0xcc241d),     // dark RED_1
    };

    /// The built-in [Catppuccin] Latte variant of a [`Palette`].
    ///
    /// [Catppuccin]: https://github.com/catppuccin/catppuccin
    pub const CATPPUCCIN_LATTE: Self = Self {
        background: color!(0xeff1f5), // Base
        text: color!(0x4c4f69),       // Text
        primary: color!(0x1e66f5),    // Blue
        success: color!(0x40a02b),    // Green
        warning: color!(0xdf8e1d),    // Yellow
        danger: color!(0xd20f39),     // Red
    };

    /// The built-in [Catppuccin] Frappé variant of a [`Palette`].
    ///
    /// [Catppuccin]: https://github.com/catppuccin/catppuccin
    pub const CATPPUCCIN_FRAPPE: Self = Self {
        background: color!(0x303446), // Base
        text: color!(0xc6d0f5),       // Text
        primary: color!(0x8caaee),    // Blue
        success: color!(0xa6d189),    // Green
        warning: color!(0xe5c890),    // Yellow
        danger: color!(0xe78284),     // Red
    };

    /// The built-in [Catppuccin] Macchiato variant of a [`Palette`].
    ///
    /// [Catppuccin]: https://github.com/catppuccin/catppuccin
    pub const CATPPUCCIN_MACCHIATO: Self = Self {
        background: color!(0x24273a), // Base
        text: color!(0xcad3f5),       // Text
        primary: color!(0x8aadf4),    // Blue
        success: color!(0xa6da95),    // Green
        warning: color!(0xeed49f),    // Yellow
        danger: color!(0xed8796),     // Red
    };

    /// The built-in [Catppuccin] Mocha variant of a [`Palette`].
    ///
    /// [Catppuccin]: https://github.com/catppuccin/catppuccin
    pub const CATPPUCCIN_MOCHA: Self = Self {
        background: color!(0x1e1e2e), // Base
        text: color!(0xcdd6f4),       // Text
        primary: color!(0x89b4fa),    // Blue
        success: color!(0xa6e3a1),    // Green
        warning: color!(0xf9e2af),    // Yellow
        danger: color!(0xf38ba8),     // Red
    };

    /// The built-in [Tokyo Night] variant of a [`Palette`].
    ///
    /// [Tokyo Night]: https://github.com/enkia/tokyo-night-vscode-theme
    pub const TOKYO_NIGHT: Self = Self {
        background: color!(0x1a1b26), // Background (Night)
        text: color!(0x9aa5ce),       // Text
        primary: color!(0x2ac3de),    // Blue
        success: color!(0x9ece6a),    // Green
        warning: color!(0xe0af68),    // Yellow
        danger: color!(0xf7768e),     // Red
    };

    /// The built-in [Tokyo Night] Storm variant of a [`Palette`].
    ///
    /// [Tokyo Night]: https://github.com/enkia/tokyo-night-vscode-theme
    pub const TOKYO_NIGHT_STORM: Self = Self {
        background: color!(0x24283b), // Background (Storm)
        text: color!(0x9aa5ce),       // Text
        primary: color!(0x2ac3de),    // Blue
        success: color!(0x9ece6a),    // Green
        warning: color!(0xe0af68),    // Yellow
        danger: color!(0xf7768e),     // Red
    };

    /// The built-in [Tokyo Night] Light variant of a [`Palette`].
    ///
    /// [Tokyo Night]: https://github.com/enkia/tokyo-night-vscode-theme
    pub const TOKYO_NIGHT_LIGHT: Self = Self {
        background: color!(0xd5d6db), // Background
        text: color!(0x565a6e),       // Text
        primary: color!(0x166775),    // Blue
        success: color!(0x485e30),    // Green
        warning: color!(0x8f5e15),    // Yellow
        danger: color!(0x8c4351),     // Red
    };

    /// The built-in [Kanagawa] Wave variant of a [`Palette`].
    ///
    /// [Kanagawa]: https://github.com/rebelot/kanagawa.nvim
    pub const KANAGAWA_WAVE: Self = Self {
        background: color!(0x1f1f28), // Sumi Ink 3
        text: color!(0xDCD7BA),       // Fuji White
        primary: color!(0x7FB4CA),    // Wave Blue
        success: color!(0x76946A),    // Autumn Green
        warning: color!(0xff9e3b),    // Ronin Yellow
        danger: color!(0xC34043),     // Autumn Red
    };

    /// The built-in [Kanagawa] Dragon variant of a [`Palette`].
    ///
    /// [Kanagawa]: https://github.com/rebelot/kanagawa.nvim
    pub const KANAGAWA_DRAGON: Self = Self {
        background: color!(0x181616), // Dragon Black 3
        text: color!(0xc5c9c5),       // Dragon White
        primary: color!(0x223249),    // Wave Blue 1
        success: color!(0x8a9a7b),    // Dragon Green 2
        warning: color!(0xff9e3b),    // Ronin Yellow
        danger: color!(0xc4746e),     // Dragon Red
    };

    /// The built-in [Kanagawa] Lotus variant of a [`Palette`].
    ///
    /// [Kanagawa]: https://github.com/rebelot/kanagawa.nvim
    pub const KANAGAWA_LOTUS: Self = Self {
        background: color!(0xf2ecbc), // Lotus White 3
        text: color!(0x545464),       // Lotus Ink 1
        primary: color!(0x4d699b),    // Lotus Blue
        success: color!(0x6f894e),    // Lotus Green
        warning: color!(0xe98a00),    // Lotus Orange 2
        danger: color!(0xc84053),     // Lotus Red
    };

    /// The built-in [Moonfly] variant of a [`Palette`].
    ///
    /// [Moonfly]: https://github.com/bluz71/vim-moonfly-colors
    pub const MOONFLY: Self = Self {
        background: color!(0x080808), // Background
        text: color!(0xbdbdbd),       // Foreground
        primary: color!(0x80a0ff),    // Blue (normal)
        success: color!(0x8cc85f),    // Green (normal)
        warning: color!(0xe3c78a),    // Yellow (normal)
        danger: color!(0xff5454),     // Red (normal)
    };

    /// The built-in [Nightfly] variant of a [`Palette`].
    ///
    /// [Nightfly]: https://github.com/bluz71/vim-nightfly-colors
    pub const NIGHTFLY: Self = Self {
        background: color!(0x011627), // Background
        text: color!(0xbdc1c6),       // Foreground
        primary: color!(0x82aaff),    // Blue (normal)
        success: color!(0xa1cd5e),    // Green (normal)
        warning: color!(0xe3d18a),    // Yellow (normal)
        danger: color!(0xfc514e),     // Red (normal)
    };

    /// The built-in [Oxocarbon] variant of a [`Palette`].
    ///
    /// [Oxocarbon]: https://github.com/nyoom-engineering/oxocarbon.nvim
    pub const OXOCARBON: Self = Self {
        background: color!(0x232323),
        text: color!(0xd0d0d0),
        primary: color!(0x00b4ff),
        success: color!(0x00c15a),
        warning: color!(0xbe95ff), // Base 14
        danger: color!(0xf62d0f),
    };

    /// The built-in [Ferra] variant of a [`Palette`].
    ///
    /// [Ferra]: https://github.com/casperstorm/ferra
    pub const FERRA: Self = Self {
        background: color!(0x2b292d),
        text: color!(0xfecdb2),
        primary: color!(0xd1d1e0),
        success: color!(0xb1b695),
        warning: color!(0xf5d76e), // Honey
        danger: color!(0xe06b75),
    };
}

/// The built-in light variant of a [`Palette`].
pub static LIGHT: LazyLock<Palette> = LazyLock::new(|| Palette::generate(Seed::LIGHT));

/// The built-in dark variant of a [`Palette`].
pub static DARK: LazyLock<Palette> = LazyLock::new(|| Palette::generate(Seed::DARK));

/// The built-in Dracula variant of a [`Palette`].
pub static DRACULA: LazyLock<Palette> = LazyLock::new(|| Palette::generate(Seed::DRACULA));

/// The built-in Nord variant of a [`Palette`].
pub static NORD: LazyLock<Palette> = LazyLock::new(|| Palette::generate(Seed::NORD));

/// The built-in Solarized Light variant of a [`Palette`].
pub static SOLARIZED_LIGHT: LazyLock<Palette> =
    LazyLock::new(|| Palette::generate(Seed::SOLARIZED_LIGHT));

/// The built-in Solarized Dark variant of a [`Palette`].
pub static SOLARIZED_DARK: LazyLock<Palette> =
    LazyLock::new(|| Palette::generate(Seed::SOLARIZED_DARK));

/// The built-in Gruvbox Light variant of a [`Palette`].
pub static GRUVBOX_LIGHT: LazyLock<Palette> =
    LazyLock::new(|| Palette::generate(Seed::GRUVBOX_LIGHT));

/// The built-in Gruvbox Dark variant of a [`Palette`].
pub static GRUVBOX_DARK: LazyLock<Palette> =
    LazyLock::new(|| Palette::generate(Seed::GRUVBOX_DARK));

/// The built-in Catppuccin Latte variant of a [`Palette`].
pub static CATPPUCCIN_LATTE: LazyLock<Palette> =
    LazyLock::new(|| Palette::generate(Seed::CATPPUCCIN_LATTE));

/// The built-in Catppuccin Frappé variant of a [`Palette`].
pub static CATPPUCCIN_FRAPPE: LazyLock<Palette> =
    LazyLock::new(|| Palette::generate(Seed::CATPPUCCIN_FRAPPE));

/// The built-in Catppuccin Macchiato variant of a [`Palette`].
pub static CATPPUCCIN_MACCHIATO: LazyLock<Palette> =
    LazyLock::new(|| Palette::generate(Seed::CATPPUCCIN_MACCHIATO));

/// The built-in Catppuccin Mocha variant of a [`Palette`].
pub static CATPPUCCIN_MOCHA: LazyLock<Palette> =
    LazyLock::new(|| Palette::generate(Seed::CATPPUCCIN_MOCHA));

/// The built-in Tokyo Night variant of a [`Palette`].
pub static TOKYO_NIGHT: LazyLock<Palette> = LazyLock::new(|| Palette::generate(Seed::TOKYO_NIGHT));

/// The built-in Tokyo Night Storm variant of a [`Palette`].
pub static TOKYO_NIGHT_STORM: LazyLock<Palette> =
    LazyLock::new(|| Palette::generate(Seed::TOKYO_NIGHT_STORM));

/// The built-in Tokyo Night variant of a [`Palette`].
pub static TOKYO_NIGHT_LIGHT: LazyLock<Palette> =
    LazyLock::new(|| Palette::generate(Seed::TOKYO_NIGHT_LIGHT));

/// The built-in Kanagawa Wave variant of a [`Palette`].
pub static KANAGAWA_WAVE: LazyLock<Palette> =
    LazyLock::new(|| Palette::generate(Seed::KANAGAWA_WAVE));

/// The built-in Kanagawa Dragon variant of a [`Palette`].
pub static KANAGAWA_DRAGON: LazyLock<Palette> =
    LazyLock::new(|| Palette::generate(Seed::KANAGAWA_DRAGON));

/// The built-in Kanagawa Lotus variant of a [`Palette`].
pub static KANAGAWA_LOTUS: LazyLock<Palette> =
    LazyLock::new(|| Palette::generate(Seed::KANAGAWA_LOTUS));

/// The built-in Moonfly variant of a [`Palette`].
pub static MOONFLY: LazyLock<Palette> = LazyLock::new(|| Palette::generate(Seed::MOONFLY));

/// The built-in Nightfly variant of a [`Palette`].
pub static NIGHTFLY: LazyLock<Palette> = LazyLock::new(|| Palette::generate(Seed::NIGHTFLY));

/// The built-in Oxocarbon variant of a [`Palette`].
pub static OXOCARBON: LazyLock<Palette> = LazyLock::new(|| Palette::generate(Seed::OXOCARBON));

/// The built-in Ferra variant of a [`Palette`].
pub static FERRA: LazyLock<Palette> = LazyLock::new(|| Palette::generate(Seed::FERRA));

/// Darkens a [`Color`] by the given factor.
pub fn darken(color: Color, amount: f32) -> Color {
    let mut oklch = color.into_oklch();

    // We try to bump the chroma a bit for more colorful palettes
    if oklch.c > 0.0 && oklch.c < (1.0 - oklch.l) / 2.0 {
        // Formula empirically and cluelessly derived
        oklch.c *= 1.0 + (0.2 / oklch.c).min(100.0) * amount;
    }

    oklch.l = if oklch.l - amount < 0.0 {
        0.0
    } else {
        oklch.l - amount
    };

    Color::from_oklch(oklch)
}

/// Lightens a [`Color`] by the given factor.
pub fn lighten(color: Color, amount: f32) -> Color {
    let mut oklch = color.into_oklch();

    // We try to bump the chroma a bit for more colorful palettes
    // Formula empirically and cluelessly derived
    oklch.c *= 1.0 + 2.0 * amount / oklch.l.max(0.05);

    oklch.l = if oklch.l + amount > 1.0 {
        1.0
    } else {
        oklch.l + amount
    };

    Color::from_oklch(oklch)
}

/// Deviates a [`Color`] by the given factor. Lightens if the [`Color`] is
/// dark, darkens otherwise.
pub fn deviate(color: Color, amount: f32) -> Color {
    if is_dark(color) {
        lighten(color, amount)
    } else {
        darken(color, amount)
    }
}

/// Computes a [`Color`] from the given text color that is
/// readable on top of the given background color.
pub fn readable(background: Color, text: Color) -> Color {
    if text.is_readable_on(background) {
        return text;
    }

    let improve = if is_dark(background) { lighten } else { darken };

    // TODO: Compute factor from relative contrast value
    let candidate = improve(text, 0.1);

    if candidate.is_readable_on(background) {
        return candidate;
    }

    let candidate = improve(text, 0.2);

    if candidate.is_readable_on(background) {
        return candidate;
    }

    let white_contrast = background.relative_contrast(Color::WHITE);
    let black_contrast = background.relative_contrast(Color::BLACK);

    if white_contrast >= black_contrast {
        Color::WHITE.mix(background, 0.05)
    } else {
        Color::BLACK.mix(background, 0.05)
    }
}

/// Returns true if the [`Color`] is dark.
pub fn is_dark(color: Color) -> bool {
    color.into_oklch().l < 0.6
}
