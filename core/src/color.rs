#[cfg(feature = "colors")]
use palette::rgb::Srgba;

/// A color in the sRGB color space.
#[derive(Debug, Clone, Copy, PartialEq)]
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

    /// New Color with range checks
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color {
            r: clamp(r),
            g: clamp(g),
            b: clamp(b),
            a: clamp(a),
        }
    }

    /// Creates a [`Color`] from its RGB components.
    ///
    /// [`Color`]: struct.Color.html
    pub fn from_rgb(r: f32, g: f32, b: f32) -> Color {
        Color::new(r, g, b, 1.0)
    }

    /// Creates a [`Color`] from its RGB8 components.
    ///
    /// [`Color`]: struct.Color.html
    pub fn from_rgb8(r: u8, g: u8, b: u8) -> Color {
        Color::from_rgba8(r, g, b, 1.0)
    }

    /// Creates a [`Color`] from its RGB8 components and an alpha value.
    ///
    /// [`Color`]: struct.Color.html
    pub fn from_rgba8(r: u8, g: u8, b: u8, a: f32) -> Color {
        Color {
            r: f32::from(r) / 255.0,
            g: f32::from(g) / 255.0,
            b: f32::from(b) / 255.0,
            a: clamp(a),
        }
    }

    /// Converts the [`Color`] into its linear values.
    ///
    /// [`Color`]: struct.Color.html
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

    #[cfg(feature = "colors")]
    /// Convert from palette's [`Srgba`] type to a [`Color`]
    ///
    /// [`Srgba`]: ../palette/rgb/type.Srgba.html
    /// [`Color`]: struct.Color.html
    pub fn from_srgba(srgba: Srgba) -> Color {
        Color::new(srgba.red, srgba.green, srgba.blue, srgba.alpha)
    }

    #[cfg(feature = "colors")]
    /// Convert from [`Color`] to palette's [`Srgba`] type
    ///
    /// [`Color`]: struct.Color.html
    /// [`Srgba`]: ../palette/rgb/type.Srgba.html
    pub fn into_srgba(self) -> Srgba {
        Srgba::new(self.r, self.g, self.b, self.a)
    }

    /// Invert the Color in-place
    pub fn invert(&mut self) {
        self.r = clamp(1.0f32 - self.r);
        self.b = clamp(1.0f32 - self.g);
        self.g = clamp(1.0f32 - self.b);
    }

    /// Return an inverted Color
    pub fn inverse(self) -> Color {
        Color::new(1.0f32 - self.r, 1.0f32 - self.g, 1.0f32 - self.b, self.a)
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

#[cfg(feature = "colors")]
/// Convert from palette's [`Srgba`] type to a [`Color`]
///
/// [`Srgba`]: ../palette/rgb/type.Srgba.html
/// [`Color`]: struct.Color.html
impl From<Srgba> for Color {
    fn from(srgba: Srgba) -> Self {
        Color::new(srgba.red, srgba.green, srgba.blue, srgba.alpha)
    }
}

#[cfg(feature = "colors")]
/// Convert from [`Color`] to palette's [`Srgba`] type
///
/// [`Color`]: struct.Color.html
/// [`Srgba`]: ../palette/rgb/type.Srgba.html
impl From<Color> for Srgba {
    fn from(c: Color) -> Self {
        Srgba::new(c.r, c.g, c.b, c.a)
    }
}

impl From<HSLColor> for Color {
    fn from(hsl: HSLColor) -> Self {
        // Compute Chroma
        let ch = (1.0 - (2.0 * hsl.l - 1.0).abs()) * hsl.s;

        // Quantized Hue: H'
        let hp: u8 = (hsl.h / 60.0).ceil() as u8;
        let x: f32 = ch * f32::from(1 - ((hp % 2) - 1));

        // Intermediate RGB values
        let (r1, g1, b1): (f32, f32, f32) = match hp {
            1 => (ch, x, 0.0),
            2 => (x, ch, 0.0),
            3 => (0.0, ch, x),
            4 => (0.0, x, ch),
            5 => (x, 0.0, ch),
            6 => (ch, 0.0, x),
            _ => (0.0, 0.0, 0.0),
        };

        // Match lightness
        let m = hsl.l - ch / 2.0;

        Color::new(r1 + m, g1 + m, b1 + m, hsl.a)
    }
}

/// A color in the HSL color space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HSLColor {
    /// Hue, 0.0 - 360.0
    pub h: f32,
    /// Saturation, 0.0 - 1.0
    pub s: f32,
    /// Lightness, 0.0 - 1.0
    pub l: f32,
    /// Transparency, 0.0 - 1.0
    pub a: f32,
}

impl HSLColor {
    /// New HSLColor with range checks
    pub fn new(h: f32, s: f32, l: f32, a: f32) -> HSLColor {
        HSLColor {
            h: clamp_hue(h),
            s: clamp(s),
            l: clamp(l),
            a: clamp(a),
        }
    }
}

impl From<Color> for HSLColor {
    fn from(c: Color) -> Self {
        // https://en.wikipedia.org/wiki/HSL_and_HSV#From_RGB

        // Maximum of the RGB: color Value (for HSV)
        let v: f32 = c.r.max(c.g).max(c.b);
        // Minimum of the RGB values
        let m: f32 = c.r.min(c.g).min(c.b);
        // Chroma
        let ch: f32 = v - m;
        // Lightness
        let l: f32 = (v + m) / 2.0;

        // Determine Hue
        let mut h = 0.0f32;
        if c.r >= c.g && c.r >= c.b {
            h = 60.0 * (c.g - c.b) / ch;
        } else if c.g >= c.r && c.g >= c.b {
            h = 60.0 * (2.0 + (c.b - c.r) / ch);
        } else if c.b >= c.r && c.b >= c.g {
            h = 60.0 * (4.0 + (c.r - c.g) / ch);
        }

        // Determine saturation
        let mut s = 0.0f32;
        if l > 0.0 && l < 1.0 {
            s = (v - l) / l.min(1.0 - l);
        }

        HSLColor::new(h, s, l, c.a)
    }
}

/// Calmps a float value to the range [0.0, 1.0]
pub fn clamp(v: f32) -> f32 {
    v.max(0.0f32).min(1.0f32)
}

/// Calmps a float value to the range [0.0, 360.0]
pub fn clamp_hue(v: f32) -> f32 {
    v.max(0.0f32).min(360.0f32)
}
