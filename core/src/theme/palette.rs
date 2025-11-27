//! Define the colors of a theme.
use crate::{Color, color};

use std::sync::LazyLock;

/// A color palette.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Palette {
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

impl Palette {
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

/// An extended set of colors generated from a [`Palette`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Extended {
    /// The set of background colors.
    pub background: Background,
    /// The set of primary colors.
    pub primary: Primary,
    /// The set of secondary colors.
    pub secondary: Secondary,
    /// The set of success colors.
    pub success: Success,
    /// The set of warning colors.
    pub warning: Warning,
    /// The set of danger colors.
    pub danger: Danger,
    /// Whether the palette is dark or not.
    pub is_dark: bool,
}

/// The built-in light variant of an [`Extended`] palette.
pub static EXTENDED_LIGHT: LazyLock<Extended> =
    LazyLock::new(|| Extended::generate(Palette::LIGHT));

/// The built-in dark variant of an [`Extended`] palette.
pub static EXTENDED_DARK: LazyLock<Extended> =
    LazyLock::new(|| Extended::generate(Palette::DARK));

/// The built-in Dracula variant of an [`Extended`] palette.
pub static EXTENDED_DRACULA: LazyLock<Extended> =
    LazyLock::new(|| Extended::generate(Palette::DRACULA));

/// The built-in Nord variant of an [`Extended`] palette.
pub static EXTENDED_NORD: LazyLock<Extended> =
    LazyLock::new(|| Extended::generate(Palette::NORD));

/// The built-in Solarized Light variant of an [`Extended`] palette.
pub static EXTENDED_SOLARIZED_LIGHT: LazyLock<Extended> =
    LazyLock::new(|| Extended::generate(Palette::SOLARIZED_LIGHT));

/// The built-in Solarized Dark variant of an [`Extended`] palette.
pub static EXTENDED_SOLARIZED_DARK: LazyLock<Extended> =
    LazyLock::new(|| Extended::generate(Palette::SOLARIZED_DARK));

/// The built-in Gruvbox Light variant of an [`Extended`] palette.
pub static EXTENDED_GRUVBOX_LIGHT: LazyLock<Extended> =
    LazyLock::new(|| Extended::generate(Palette::GRUVBOX_LIGHT));

/// The built-in Gruvbox Dark variant of an [`Extended`] palette.
pub static EXTENDED_GRUVBOX_DARK: LazyLock<Extended> =
    LazyLock::new(|| Extended::generate(Palette::GRUVBOX_DARK));

/// The built-in Catppuccin Latte variant of an [`Extended`] palette.
pub static EXTENDED_CATPPUCCIN_LATTE: LazyLock<Extended> =
    LazyLock::new(|| Extended::generate(Palette::CATPPUCCIN_LATTE));

/// The built-in Catppuccin Frappé variant of an [`Extended`] palette.
pub static EXTENDED_CATPPUCCIN_FRAPPE: LazyLock<Extended> =
    LazyLock::new(|| Extended::generate(Palette::CATPPUCCIN_FRAPPE));

/// The built-in Catppuccin Macchiato variant of an [`Extended`] palette.
pub static EXTENDED_CATPPUCCIN_MACCHIATO: LazyLock<Extended> =
    LazyLock::new(|| Extended::generate(Palette::CATPPUCCIN_MACCHIATO));

/// The built-in Catppuccin Mocha variant of an [`Extended`] palette.
pub static EXTENDED_CATPPUCCIN_MOCHA: LazyLock<Extended> =
    LazyLock::new(|| Extended::generate(Palette::CATPPUCCIN_MOCHA));

/// The built-in Tokyo Night variant of an [`Extended`] palette.
pub static EXTENDED_TOKYO_NIGHT: LazyLock<Extended> =
    LazyLock::new(|| Extended::generate(Palette::TOKYO_NIGHT));

/// The built-in Tokyo Night Storm variant of an [`Extended`] palette.
pub static EXTENDED_TOKYO_NIGHT_STORM: LazyLock<Extended> =
    LazyLock::new(|| Extended::generate(Palette::TOKYO_NIGHT_STORM));

/// The built-in Tokyo Night variant of an [`Extended`] palette.
pub static EXTENDED_TOKYO_NIGHT_LIGHT: LazyLock<Extended> =
    LazyLock::new(|| Extended::generate(Palette::TOKYO_NIGHT_LIGHT));

/// The built-in Kanagawa Wave variant of an [`Extended`] palette.
pub static EXTENDED_KANAGAWA_WAVE: LazyLock<Extended> =
    LazyLock::new(|| Extended::generate(Palette::KANAGAWA_WAVE));

/// The built-in Kanagawa Dragon variant of an [`Extended`] palette.
pub static EXTENDED_KANAGAWA_DRAGON: LazyLock<Extended> =
    LazyLock::new(|| Extended::generate(Palette::KANAGAWA_DRAGON));

/// The built-in Kanagawa Lotus variant of an [`Extended`] palette.
pub static EXTENDED_KANAGAWA_LOTUS: LazyLock<Extended> =
    LazyLock::new(|| Extended::generate(Palette::KANAGAWA_LOTUS));

/// The built-in Moonfly variant of an [`Extended`] palette.
pub static EXTENDED_MOONFLY: LazyLock<Extended> =
    LazyLock::new(|| Extended::generate(Palette::MOONFLY));

/// The built-in Nightfly variant of an [`Extended`] palette.
pub static EXTENDED_NIGHTFLY: LazyLock<Extended> =
    LazyLock::new(|| Extended::generate(Palette::NIGHTFLY));

/// The built-in Oxocarbon variant of an [`Extended`] palette.
pub static EXTENDED_OXOCARBON: LazyLock<Extended> =
    LazyLock::new(|| Extended::generate(Palette::OXOCARBON));

/// The built-in Ferra variant of an [`Extended`] palette.
pub static EXTENDED_FERRA: LazyLock<Extended> =
    LazyLock::new(|| Extended::generate(Palette::FERRA));

impl Extended {
    /// Generates an [`Extended`] palette from a simple [`Palette`].
    pub fn generate(palette: Palette) -> Self {
        Self {
            background: Background::new(palette.background, palette.text),
            primary: Primary::generate(
                palette.primary,
                palette.background,
                palette.text,
            ),
            secondary: Secondary::generate(palette.background, palette.text),
            success: Success::generate(
                palette.success,
                palette.background,
                palette.text,
            ),
            warning: Warning::generate(
                palette.warning,
                palette.background,
                palette.text,
            ),
            danger: Danger::generate(
                palette.danger,
                palette.background,
                palette.text,
            ),
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

/// A set of primary colors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Primary {
    /// The base primary color.
    pub base: Pair,
    /// A weaker version of the base primary color.
    pub weak: Pair,
    /// A stronger version of the base primary color.
    pub strong: Pair,
}

impl Primary {
    /// Generates a set of [`Primary`] colors from the base, background, and text colors.
    pub fn generate(base: Color, background: Color, text: Color) -> Self {
        let weak = mix(base, background, 0.4);
        let strong = deviate(base, 0.1);

        Self {
            base: Pair::new(base, text),
            weak: Pair::new(weak, text),
            strong: Pair::new(strong, text),
        }
    }
}

/// A set of secondary colors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Secondary {
    /// The base secondary color.
    pub base: Pair,
    /// A weaker version of the base secondary color.
    pub weak: Pair,
    /// A stronger version of the base secondary color.
    pub strong: Pair,
}

impl Secondary {
    /// Generates a set of [`Secondary`] colors from the base and text colors.
    pub fn generate(base: Color, text: Color) -> Self {
        let factor = if is_dark(base) { 0.2 } else { 0.4 };

        let weak = mix(deviate(base, 0.1), text, factor);
        let strong = mix(deviate(base, 0.3), text, factor);
        let base = mix(deviate(base, 0.2), text, factor);

        Self {
            base: Pair::new(base, text),
            weak: Pair::new(weak, text),
            strong: Pair::new(strong, text),
        }
    }
}

/// A set of success colors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Success {
    /// The base success color.
    pub base: Pair,
    /// A weaker version of the base success color.
    pub weak: Pair,
    /// A stronger version of the base success color.
    pub strong: Pair,
}

impl Success {
    /// Generates a set of [`Success`] colors from the base, background, and text colors.
    pub fn generate(base: Color, background: Color, text: Color) -> Self {
        let weak = mix(base, background, 0.4);
        let strong = deviate(base, 0.1);

        Self {
            base: Pair::new(base, text),
            weak: Pair::new(weak, text),
            strong: Pair::new(strong, text),
        }
    }
}

/// A set of warning colors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Warning {
    /// The base warning color.
    pub base: Pair,
    /// A weaker version of the base warning color.
    pub weak: Pair,
    /// A stronger version of the base warning color.
    pub strong: Pair,
}

impl Warning {
    /// Generates a set of [`Warning`] colors from the base, background, and text colors.
    pub fn generate(base: Color, background: Color, text: Color) -> Self {
        let weak = mix(base, background, 0.4);
        let strong = deviate(base, 0.1);

        Self {
            base: Pair::new(base, text),
            weak: Pair::new(weak, text),
            strong: Pair::new(strong, text),
        }
    }
}

/// A set of danger colors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Danger {
    /// The base danger color.
    pub base: Pair,
    /// A weaker version of the base danger color.
    pub weak: Pair,
    /// A stronger version of the base danger color.
    pub strong: Pair,
}

impl Danger {
    /// Generates a set of [`Danger`] colors from the base, background, and text colors.
    pub fn generate(base: Color, background: Color, text: Color) -> Self {
        let weak = mix(base, background, 0.4);
        let strong = deviate(base, 0.1);

        Self {
            base: Pair::new(base, text),
            weak: Pair::new(weak, text),
            strong: Pair::new(strong, text),
        }
    }
}

struct Oklch {
    l: f32,
    c: f32,
    h: f32,
    a: f32,
}

/// Darkens a [`Color`] by the given factor.
pub fn darken(color: Color, amount: f32) -> Color {
    let mut oklch = to_oklch(color);

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

    from_oklch(oklch)
}

/// Lightens a [`Color`] by the given factor.
pub fn lighten(color: Color, amount: f32) -> Color {
    let mut oklch = to_oklch(color);

    // We try to bump the chroma a bit for more colorful palettes
    // Formula empirically and cluelessly derived
    oklch.c *= 1.0 + 2.0 * amount / oklch.l.max(0.05);

    oklch.l = if oklch.l + amount > 1.0 {
        1.0
    } else {
        oklch.l + amount
    };

    from_oklch(oklch)
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

/// Mixes two colors by the given factor.
pub fn mix(a: Color, b: Color, factor: f32) -> Color {
    let b_amount = factor.clamp(0.0, 1.0);
    let a_amount = 1.0 - b_amount;

    let a_linear = a.into_linear().map(|c| c * a_amount);
    let b_linear = b.into_linear().map(|c| c * b_amount);

    Color::from_linear_rgba(
        a_linear[0] + b_linear[0],
        a_linear[1] + b_linear[1],
        a_linear[2] + b_linear[2],
        a_linear[3] + b_linear[3],
    )
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
        mix(Color::WHITE, background, 0.05)
    } else {
        mix(Color::BLACK, background, 0.05)
    }
}

/// Returns true if the [`Color`] is dark.
pub fn is_dark(color: Color) -> bool {
    to_oklch(color).l < 0.6
}

// https://en.wikipedia.org/wiki/Oklab_color_space#Conversions_between_color_spaces
fn to_oklch(color: Color) -> Oklch {
    let [r, g, b, alpha] = color.into_linear();

    // linear RGB → LMS
    let l = 0.41222146 * r + 0.53633255 * g + 0.051445995 * b;
    let m = 0.2119035 * r + 0.6806995 * g + 0.10739696 * b;
    let s = 0.08830246 * r + 0.28171885 * g + 0.6299787 * b;

    // Nonlinear transform (cube root)
    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    // LMS → Oklab
    let l = 0.21045426 * l_ + 0.7936178 * m_ - 0.004072047 * s_;
    let a = 1.9779985 * l_ - 2.4285922 * m_ + 0.4505937 * s_;
    let b = 0.025904037 * l_ + 0.78277177 * m_ - 0.80867577 * s_;

    // Oklab → Oklch
    let c = (a * a + b * b).sqrt();
    let h = b.atan2(a); // radians

    Oklch { l, c, h, a: alpha }
}

// https://en.wikipedia.org/wiki/Oklab_color_space#Conversions_between_color_spaces
fn from_oklch(oklch: Oklch) -> Color {
    let Oklch { l, c, h, a: alpha } = oklch;

    let a = c * h.cos();
    let b = c * h.sin();

    // Oklab → LMS (nonlinear)
    let l_ = l + 0.39633778 * a + 0.21580376 * b;
    let m_ = l - 0.105561346 * a - 0.06385417 * b;
    let s_ = l - 0.08948418 * a - 1.2914855 * b;

    // Cubing back
    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    let r = 4.0767417 * l - 3.3077116 * m + 0.23096994 * s;
    let g = -1.268438 * l + 2.6097574 * m - 0.34131938 * s;
    let b = -0.0041960863 * l - 0.7034186 * m + 1.7076147 * s;

    Color::from_linear_rgba(
        r.clamp(0.0, 1.0),
        g.clamp(0.0, 1.0),
        b.clamp(0.0, 1.0),
        alpha,
    )
}
