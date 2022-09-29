//TODO: GET CORRECT PALETTE FROM COSMIC-THEME
use iced_core::Color;

use lazy_static::lazy_static;
use palette::{FromColor, Hsl, Mix, RelativeContrast, Srgb};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    pub background: Color,
    pub text: Color,
    pub primary: Color,
    pub success: Color,
    pub danger: Color,
}

impl Palette {
    pub const LIGHT: Self = Self {
        background: Color::from_rgb(
            0xee as f32 / 255.0,
            0xee as f32 / 255.0,
            0xee as f32 / 255.0,
        ),
        text: Color::from_rgb(
            0x00 as f32 / 255.0,
            0x00 as f32 / 255.0,
            0x00 as f32 / 255.0,
        ),
        primary: Color::from_rgb(
            0x00 as f32 / 255.0,
            0x49 as f32 / 255.0,
            0x6d as f32 / 255.0,
        ),
        success: Color::from_rgb(
            0x3b as f32 / 255.0,
            0x6e as f32 / 255.0,
            0x43 as f32 / 255.0,
        ),
        danger: Color::from_rgb(
            0xa0 as f32 / 255.0,
            0x25 as f32 / 255.0,
            0x2b as f32 / 255.0,
        ),
    };

    pub const DARK: Self = Self {
        background: Color::from_rgb(
            0x29 as f32 / 255.0,
            0x29 as f32 / 255.0,
            0x29 as f32 / 255.0
        ),
        text: Color::from_rgb(
            0xe4 as f32 / 255.0,
            0xe4 as f32 / 255.0,
            0xe4 as f32 / 255.0,
        ),
        primary: Color::from_rgb(
            0x94 as f32 / 255.0,
            0xeb as f32 / 255.0,
            0xeb as f32 / 255.0,
        ),
        success: Color::from_rgb(
            0xac as f32 / 255.0,
            0xf7 as f32 / 255.0,
            0xd2 as f32 / 255.0,
        ),
        danger: Color::from_rgb(
            0xff as f32 / 255.0,
            0xb5 as f32 / 255.0,
            0xb5 as f32 / 255.0,
        ),
    };
}

pub struct Extended {
    pub background: Background,
    pub primary: Primary,
    pub secondary: Secondary,
    pub success: Success,
    pub danger: Danger,
}

lazy_static! {
    pub static ref EXTENDED_LIGHT: Extended =
        Extended::generate(Palette::LIGHT);
    pub static ref EXTENDED_DARK: Extended = Extended::generate(Palette::DARK);
}

impl Extended {
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
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Pair {
    pub color: Color,
    pub text: Color,
}

impl Pair {
    pub fn new(color: Color, text: Color) -> Self {
        Self {
            color,
            text: readable(color, text),
        }
    }
}

pub struct Background {
    pub base: Pair,
    pub weak: Pair,
    pub strong: Pair,
}

impl Background {
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

pub struct Primary {
    pub base: Pair,
    pub weak: Pair,
    pub strong: Pair,
}

impl Primary {
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

pub struct Secondary {
    pub base: Pair,
    pub weak: Pair,
    pub strong: Pair,
}

impl Secondary {
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

pub struct Success {
    pub base: Pair,
    pub weak: Pair,
    pub strong: Pair,
}

impl Success {
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

pub struct Danger {
    pub base: Pair,
    pub weak: Pair,
    pub strong: Pair,
}

impl Danger {
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
    let a_lin = Srgb::from(a).into_linear();
    let b_lin = Srgb::from(b).into_linear();

    let mixed = a_lin.mix(&b_lin, factor);
    Srgb::from_linear(mixed).into()
}

fn readable(background: Color, text: Color) -> Color {
    if is_readable(background, text) {
        text
    } else if is_dark(background) {
        Color::WHITE
    } else {
        Color::BLACK
    }
}

fn is_dark(color: Color) -> bool {
    to_hsl(color).lightness < 0.6
}

fn is_readable(a: Color, b: Color) -> bool {
    let a_srgb = Srgb::from(a);
    let b_srgb = Srgb::from(b);

    a_srgb.has_enhanced_contrast_text(&b_srgb)
}

fn to_hsl(color: Color) -> Hsl {
    Hsl::from_color(Srgb::from(color))
}

fn from_hsl(hsl: Hsl) -> Color {
    Srgb::from_color(hsl).into()
}
