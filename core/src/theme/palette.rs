//! Define the colors of a theme.
use crate::{color, Color};

use once_cell::sync::Lazy;
use palette::color_difference::Wcag21RelativeContrast;
use palette::rgb::Rgb;
use palette::{FromColor, Hsl, Mix};

/// A color palette.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    /// The background [`Color`] of the [`Palette`].
    pub background: Color,
    /// The text [`Color`] of the [`Palette`].
    pub text: Color,
    /// The primary [`Color`] of the [`Palette`].
    pub primary: Color,
    /// The success [`Color`] of the [`Palette`].
    pub success: Color,
    /// The danger [`Color`] of the [`Palette`].
    pub danger: Color,
}

impl Palette {
    /// The built-in light variant of a [`Palette`].
    pub const LIGHT: Self = Self {
        background: Color::WHITE,
        text: Color::BLACK,
        primary: Color::from_rgb(
            0x5E as f32 / 255.0,
            0x7C as f32 / 255.0,
            0xE2 as f32 / 255.0,
        ),
        success: Color::from_rgb(
            0x12 as f32 / 255.0,
            0x66 as f32 / 255.0,
            0x4F as f32 / 255.0,
        ),
        danger: Color::from_rgb(
            0xC3 as f32 / 255.0,
            0x42 as f32 / 255.0,
            0x3F as f32 / 255.0,
        ),
    };

    /// The built-in dark variant of a [`Palette`].
    pub const DARK: Self = Self {
        background: Color::from_rgb(
            0x20 as f32 / 255.0,
            0x22 as f32 / 255.0,
            0x25 as f32 / 255.0,
        ),
        text: Color::from_rgb(0.90, 0.90, 0.90),
        primary: Color::from_rgb(
            0x5E as f32 / 255.0,
            0x7C as f32 / 255.0,
            0xE2 as f32 / 255.0,
        ),
        success: Color::from_rgb(
            0x12 as f32 / 255.0,
            0x66 as f32 / 255.0,
            0x4F as f32 / 255.0,
        ),
        danger: Color::from_rgb(
            0xC3 as f32 / 255.0,
            0x42 as f32 / 255.0,
            0x3F as f32 / 255.0,
        ),
    };

    /// The built-in [Dracula] variant of a [`Palette`].
    ///
    /// [Dracula]: https://draculatheme.com
    pub const DRACULA: Self = Self {
        background: color!(0x282A36), // BACKGROUND
        text: color!(0xf8f8f2),       // FOREGROUND
        primary: color!(0xbd93f9),    // PURPLE
        success: color!(0x50fa7b),    // GREEN
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
        danger: color!(0x8c4351),     // Red
    };

    /// The built-in [Kanagawa] Wave variant of a [`Palette`].
    ///
    /// [Kanagawa]: https://github.com/rebelot/kanagawa.nvim
    pub const KANAGAWA_WAVE: Self = Self {
        background: color!(0x363646), // Sumi Ink 3
        text: color!(0xCD7BA),        // Fuji White
        primary: color!(0x2D4F67),    // Wave Blue 2
        success: color!(0x76946A),    // Autumn Green
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
        danger: color!(0xc4746e),     // Dragon Red
    };

    /// The built-in [Kanagawa] Lotus variant of a [`Palette`].
    ///
    /// [Kanagawa]: https://github.com/rebelot/kanagawa.nvim
    pub const KANAGAWA_LOTUS: Self = Self {
        background: color!(0xf2ecbc), // Lotus White 3
        text: color!(0x545464),       // Lotus Ink 1
        primary: color!(0xc9cbd1),    // Lotus Violet 3
        success: color!(0x6f894e),    // Lotus Green
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
    /// The set of danger colors.
    pub danger: Danger,
    /// Whether the palette is dark or not.
    pub is_dark: bool,
}

/// The built-in light variant of an [`Extended`] palette.
pub static EXTENDED_LIGHT: Lazy<Extended> =
    Lazy::new(|| Extended::generate(Palette::LIGHT));

/// The built-in dark variant of an [`Extended`] palette.
pub static EXTENDED_DARK: Lazy<Extended> =
    Lazy::new(|| Extended::generate(Palette::DARK));

/// The built-in Dracula variant of an [`Extended`] palette.
pub static EXTENDED_DRACULA: Lazy<Extended> =
    Lazy::new(|| Extended::generate(Palette::DRACULA));

/// The built-in Nord variant of an [`Extended`] palette.
pub static EXTENDED_NORD: Lazy<Extended> =
    Lazy::new(|| Extended::generate(Palette::NORD));

/// The built-in Solarized Light variant of an [`Extended`] palette.
pub static EXTENDED_SOLARIZED_LIGHT: Lazy<Extended> =
    Lazy::new(|| Extended::generate(Palette::SOLARIZED_LIGHT));

/// The built-in Solarized Dark variant of an [`Extended`] palette.
pub static EXTENDED_SOLARIZED_DARK: Lazy<Extended> =
    Lazy::new(|| Extended::generate(Palette::SOLARIZED_DARK));

/// The built-in Gruvbox Light variant of an [`Extended`] palette.
pub static EXTENDED_GRUVBOX_LIGHT: Lazy<Extended> =
    Lazy::new(|| Extended::generate(Palette::GRUVBOX_LIGHT));

/// The built-in Gruvbox Dark variant of an [`Extended`] palette.
pub static EXTENDED_GRUVBOX_DARK: Lazy<Extended> =
    Lazy::new(|| Extended::generate(Palette::GRUVBOX_DARK));

/// The built-in Catppuccin Latte variant of an [`Extended`] palette.
pub static EXTENDED_CATPPUCCIN_LATTE: Lazy<Extended> =
    Lazy::new(|| Extended::generate(Palette::CATPPUCCIN_LATTE));

/// The built-in Catppuccin Frappé variant of an [`Extended`] palette.
pub static EXTENDED_CATPPUCCIN_FRAPPE: Lazy<Extended> =
    Lazy::new(|| Extended::generate(Palette::CATPPUCCIN_FRAPPE));

/// The built-in Catppuccin Macchiato variant of an [`Extended`] palette.
pub static EXTENDED_CATPPUCCIN_MACCHIATO: Lazy<Extended> =
    Lazy::new(|| Extended::generate(Palette::CATPPUCCIN_MACCHIATO));

/// The built-in Catppuccin Mocha variant of an [`Extended`] palette.
pub static EXTENDED_CATPPUCCIN_MOCHA: Lazy<Extended> =
    Lazy::new(|| Extended::generate(Palette::CATPPUCCIN_MOCHA));

/// The built-in Tokyo Night variant of an [`Extended`] palette.
pub static EXTENDED_TOKYO_NIGHT: Lazy<Extended> =
    Lazy::new(|| Extended::generate(Palette::TOKYO_NIGHT));

/// The built-in Tokyo Night Storm variant of an [`Extended`] palette.
pub static EXTENDED_TOKYO_NIGHT_STORM: Lazy<Extended> =
    Lazy::new(|| Extended::generate(Palette::TOKYO_NIGHT_STORM));

/// The built-in Tokyo Night variant of an [`Extended`] palette.
pub static EXTENDED_TOKYO_NIGHT_LIGHT: Lazy<Extended> =
    Lazy::new(|| Extended::generate(Palette::TOKYO_NIGHT_LIGHT));

/// The built-in Kanagawa Wave variant of an [`Extended`] palette.
pub static EXTENDED_KANAGAWA_WAVE: Lazy<Extended> =
    Lazy::new(|| Extended::generate(Palette::KANAGAWA_WAVE));

/// The built-in Kanagawa Dragon variant of an [`Extended`] palette.
pub static EXTENDED_KANAGAWA_DRAGON: Lazy<Extended> =
    Lazy::new(|| Extended::generate(Palette::KANAGAWA_DRAGON));

/// The built-in Kanagawa Lotus variant of an [`Extended`] palette.
pub static EXTENDED_KANAGAWA_LOTUS: Lazy<Extended> =
    Lazy::new(|| Extended::generate(Palette::KANAGAWA_LOTUS));

/// The built-in Moonfly variant of an [`Extended`] palette.
pub static EXTENDED_MOONFLY: Lazy<Extended> =
    Lazy::new(|| Extended::generate(Palette::MOONFLY));

/// The built-in Nightfly variant of an [`Extended`] palette.
pub static EXTENDED_NIGHTFLY: Lazy<Extended> =
    Lazy::new(|| Extended::generate(Palette::NIGHTFLY));

/// The built-in Oxocarbon variant of an [`Extended`] palette.
pub static EXTENDED_OXOCARBON: Lazy<Extended> =
    Lazy::new(|| Extended::generate(Palette::OXOCARBON));

/// The built-in Ferra variant of an [`Extended`] palette.
pub static EXTENDED_FERRA: Lazy<Extended> =
    Lazy::new(|| Extended::generate(Palette::FERRA));

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
    /// A weaker version of the base background color.
    pub weak: Pair,
    /// A stronger version of the base background color.
    pub strong: Pair,
}

impl Background {
    /// Generates a set of [`Background`] colors from the base and text colors.
    pub fn new(base: Color, text: Color) -> Self {
        let weak = mix(base, text, 0.15);
        let strong = mix(base, text, 0.40);

        Self {
            base: Pair::new(base, text),
            weak: Pair::new(weak, text),
            strong: Pair::new(strong, text),
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
        let base = mix(base, text, 0.2);
        let weak = mix(base, text, 0.1);
        let strong = mix(base, text, 0.3);

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

fn darken(color: Color, amount: f32) -> Color {
    let mut hsl = to_hsl(color);

    hsl.lightness = if hsl.lightness - amount < 0.0 {
        0.0
    } else {
        hsl.lightness - amount
    };

    from_hsl(hsl)
}

fn lighten(color: Color, amount: f32) -> Color {
    let mut hsl = to_hsl(color);

    hsl.lightness = if hsl.lightness + amount > 1.0 {
        1.0
    } else {
        hsl.lightness + amount
    };

    from_hsl(hsl)
}

fn deviate(color: Color, amount: f32) -> Color {
    if is_dark(color) {
        lighten(color, amount)
    } else {
        darken(color, amount)
    }
}

fn mix(a: Color, b: Color, factor: f32) -> Color {
    let a_lin = Rgb::from(a).into_linear();
    let b_lin = Rgb::from(b).into_linear();

    let mixed = a_lin.mix(b_lin, factor);
    Rgb::from_linear(mixed).into()
}

fn readable(background: Color, text: Color) -> Color {
    if is_readable(background, text) {
        text
    } else {
        let white_contrast = relative_contrast(background, Color::WHITE);
        let black_contrast = relative_contrast(background, Color::BLACK);

        if white_contrast >= black_contrast {
            Color::WHITE
        } else {
            Color::BLACK
        }
    }
}

fn is_dark(color: Color) -> bool {
    to_hsl(color).lightness < 0.6
}

fn is_readable(a: Color, b: Color) -> bool {
    let a_srgb = Rgb::from(a);
    let b_srgb = Rgb::from(b);

    a_srgb.has_enhanced_contrast_text(b_srgb)
}

fn relative_contrast(a: Color, b: Color) -> f32 {
    let a_srgb = Rgb::from(a);
    let b_srgb = Rgb::from(b);

    a_srgb.relative_contrast(b_srgb)
}

fn to_hsl(color: Color) -> Hsl {
    Hsl::from_color(Rgb::from(color))
}

fn from_hsl(hsl: Hsl) -> Color {
    Rgb::from_color(hsl).into()
}
