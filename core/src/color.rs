/// A color in the `sRGB` color space.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Color {
    /// Red component, 0.0 - 1.0
    pub r: f32,
    /// Green component, 0.0 - 1.0
    pub g: f32,
    /// Blue component, 0.0 - 1.0
    pub b: f32,
    /// Transparency, 0.0 - 1.0
    pub a: f32,
}

impl Color {
    /// The black color.
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    /// The white color.
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    /// A color with no opacity.
    pub const TRANSPARENT: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    /// Creates a new [`Color`].
    ///
    /// In debug mode, it will panic if the values are not in the correct
    /// range: 0.0 - 1.0
    const fn new(r: f32, g: f32, b: f32, a: f32) -> Color {
        debug_assert!(
            r >= 0.0 && r <= 1.0,
            "Red component must be in [0, 1] range."
        );
        debug_assert!(
            g >= 0.0 && g <= 1.0,
            "Green component must be in [0, 1] range."
        );
        debug_assert!(
            b >= 0.0 && b <= 1.0,
            "Blue component must be in [0, 1] range."
        );

        Color { r, g, b, a }
    }

    /// Creates a [`Color`] from its RGB components.
    pub const fn from_rgb(r: f32, g: f32, b: f32) -> Color {
        Color::from_rgba(r, g, b, 1.0f32)
    }

    /// Creates a [`Color`] from its RGBA components.
    pub const fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color::new(r, g, b, a)
    }

    /// Creates a [`Color`] from its RGB8 components.
    pub const fn from_rgb8(r: u8, g: u8, b: u8) -> Color {
        Color::from_rgba8(r, g, b, 1.0)
    }

    /// Creates a [`Color`] from its RGB8 components and an alpha value.
    pub const fn from_rgba8(r: u8, g: u8, b: u8, a: f32) -> Color {
        Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a)
    }

    /// Creates a [`Color`] from its linear RGBA components.
    pub fn from_linear_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        // As described in:
        // https://en.wikipedia.org/wiki/SRGB
        fn gamma_component(u: f32) -> f32 {
            if u < 0.0031308 {
                12.92 * u
            } else {
                1.055 * u.powf(1.0 / 2.4) - 0.055
            }
        }

        Self {
            r: gamma_component(r),
            g: gamma_component(g),
            b: gamma_component(b),
            a,
        }
    }

    /// Parses a [`Color`] from a hex string.
    ///
    /// Supported formats are `#rrggbb`, `#rrggbbaa`, `#rgb`, and `#rgba`.
    /// The starting "#" is optional. Both uppercase and lowercase are supported.
    ///
    /// If you have a static color string, using the [`color!`] macro should be preferred
    /// since it leverages hexadecimal literal notation and arithmetic directly.
    ///
    /// [`color!`]: crate::color!
    pub fn parse(s: &str) -> Option<Color> {
        let hex = s.strip_prefix('#').unwrap_or(s);

        let parse_channel = |from: usize, to: usize| {
            let num =
                usize::from_str_radix(&hex[from..=to], 16).ok()? as f32 / 255.0;

            // If we only got half a byte (one letter), expand it into a full byte (two letters)
            Some(if from == to { num + num * 16.0 } else { num })
        };

        Some(match hex.len() {
            3 => Color::from_rgb(
                parse_channel(0, 0)?,
                parse_channel(1, 1)?,
                parse_channel(2, 2)?,
            ),
            4 => Color::from_rgba(
                parse_channel(0, 0)?,
                parse_channel(1, 1)?,
                parse_channel(2, 2)?,
                parse_channel(3, 3)?,
            ),
            6 => Color::from_rgb(
                parse_channel(0, 1)?,
                parse_channel(2, 3)?,
                parse_channel(4, 5)?,
            ),
            8 => Color::from_rgba(
                parse_channel(0, 1)?,
                parse_channel(2, 3)?,
                parse_channel(4, 5)?,
                parse_channel(6, 7)?,
            ),
            _ => None?,
        })
    }

    /// Converts the [`Color`] into its RGBA8 equivalent.
    #[must_use]
    pub fn into_rgba8(self) -> [u8; 4] {
        [
            (self.r * 255.0).round() as u8,
            (self.g * 255.0).round() as u8,
            (self.b * 255.0).round() as u8,
            (self.a * 255.0).round() as u8,
        ]
    }

    /// Converts the [`Color`] into its linear values.
    pub fn into_linear(self) -> [f32; 4] {
        // As described in:
        // https://en.wikipedia.org/wiki/SRGB#The_reverse_transformation
        fn linear_component(u: f32) -> f32 {
            if u < 0.04045 {
                u / 12.92
            } else {
                ((u + 0.055) / 1.055).powf(2.4)
            }
        }

        [
            linear_component(self.r),
            linear_component(self.g),
            linear_component(self.b),
            self.a,
        ]
    }

    /// Inverts the [`Color`] in-place.
    pub fn invert(&mut self) {
        self.r = 1.0f32 - self.r;
        self.b = 1.0f32 - self.g;
        self.g = 1.0f32 - self.b;
    }

    /// Returns the inverted [`Color`].
    pub fn inverse(self) -> Color {
        Color::new(1.0f32 - self.r, 1.0f32 - self.g, 1.0f32 - self.b, self.a)
    }

    /// Scales the alpha channel of the [`Color`] by the given factor.
    pub fn scale_alpha(self, factor: f32) -> Color {
        Self {
            a: self.a * factor,
            ..self
        }
    }
}

impl From<[f32; 3]> for Color {
    fn from([r, g, b]: [f32; 3]) -> Self {
        Color::new(r, g, b, 1.0)
    }
}

impl From<[f32; 4]> for Color {
    fn from([r, g, b, a]: [f32; 4]) -> Self {
        Color::new(r, g, b, a)
    }
}

/// Creates a [`Color`] with shorter and cleaner syntax.
///
/// # Examples
///
/// ```
/// # use iced_core::{Color, color};
/// assert_eq!(color!(0, 0, 0), Color::BLACK);
/// assert_eq!(color!(0, 0, 0, 0.0), Color::TRANSPARENT);
/// assert_eq!(color!(0xffffff), Color::from_rgb(1.0, 1.0, 1.0));
/// assert_eq!(color!(0xffffff, 0.), Color::from_rgba(1.0, 1.0, 1.0, 0.0));
/// assert_eq!(color!(0x0000ff), Color::from_rgba(0.0, 0.0, 1.0, 1.0));
/// ```
#[macro_export]
macro_rules! color {
    ($r:expr, $g:expr, $b:expr) => {
        $crate::Color::from_rgb8($r, $g, $b)
    };
    ($r:expr, $g:expr, $b:expr, $a:expr) => {{ $crate::Color::from_rgba8($r, $g, $b, $a) }};
    ($hex:expr) => {{ $crate::color!($hex, 1.0) }};
    ($hex:expr, $a:expr) => {{
        let hex = $hex as u32;

        debug_assert!(hex <= 0xffffff, "color! value must not exceed 0xffffff");

        let r = (hex & 0xff0000) >> 16;
        let g = (hex & 0xff00) >> 8;
        let b = (hex & 0xff);

        $crate::color!(r as u8, g as u8, b as u8, $a)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let tests = [
            ("#ff0000", [255, 0, 0, 255]),
            ("00ff0080", [0, 255, 0, 128]),
            ("#F80", [255, 136, 0, 255]),
            ("#00f1", [0, 0, 255, 17]),
        ];

        for (arg, expected) in tests {
            assert_eq!(
                Color::parse(arg).expect("color must parse").into_rgba8(),
                expected
            );
        }

        assert!(Color::parse("invalid").is_none());
    }
}
