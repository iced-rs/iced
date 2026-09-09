//! Handle tablet tool input.

use std::cell::LazyCell;
use std::cmp::Ordering;

use crate::pointer::mouse;
use crate::pointer::touch::Force;

/// The kind of tool used with a tablet.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// A pen.
    #[default]
    Pen,

    /// An eraser.
    Eraser,

    /// A brush.
    Brush,

    /// A pencil.
    Pencil,

    /// An airbrush.
    Airbrush,

    /// A finger-like tool.
    Finger,

    /// A mouse-like tool.
    Mouse,

    /// A lens cursor.
    Lens,
}

/// Data describing how a tablet tool is held and used.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Data {
    /// The force applied to the tool against the surface.
    ///
    /// When the force information is not available, [`None`] is returned.
    ///
    /// ## Platform-specific
    ///
    /// **Web:** Has no mechanism to detect support, so this will always be [`Some`].
    pub force: Option<Force>,

    /// Represents normalized tangential pressure, also known as barrel pressure. In the range of
    /// -1 to 1. 0 means no tangential pressure is applied. [`None`] means backend or device has no
    /// support.
    ///
    /// ## Platform-specific
    ///
    /// **Web:** Has no mechanism to detect support, so this will always be [`Some`] with a value
    /// of 0.
    pub tangential_force: Option<f32>,

    /// The clockwise rotation in degrees of a tool around its own major axis. E.g. twisting a pen
    /// around its length. In the range of 0 to 359. [`None`] means backend or device has no
    /// support.
    ///
    /// ## Platform-specific
    ///
    /// **Web:** Has no mechanism to detect support, so this will always be [`Some`] with a value
    /// of 0.
    pub twist: Option<u16>,

    /// The plane angle in degrees. [`None`] means backend or device has no support.
    ///
    /// ## Platform-specific
    ///
    /// **Web:** Has no mechanism to detect support, so this will always be [`Some`] with default
    /// values.
    pub tilt: Option<Tilt>,

    /// The angular position in radians. [`None`] means backend or device has no support.
    ///
    /// ## Platform-specific
    ///
    /// **Web:** Has no mechanism to detect device support, so this will always be [`Some`] with
    /// default values unless browser support is lacking.
    pub angle: Option<Angle>,
}

impl Data {
    /// Returns [`Tilt`] if present or calculates it from [`Angle`].
    pub fn tilt(self) -> Option<Tilt> {
        if let Some(tilt) = self.tilt {
            Some(tilt)
        } else {
            self.angle.map(Angle::tilt)
        }
    }

    /// Returns [`Angle`] if present or calculates it from [`Tilt`].
    pub fn angle(self) -> Option<Angle> {
        if let Some(angle) = self.angle {
            Some(angle)
        } else {
            self.tilt.map(Tilt::angle)
        }
    }
}

/// The plane angle of a tablet tool in degrees.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tilt {
    /// The plane angle in degrees between the surface Y-Z plane and the plane containing the tool
    /// and the surface Y axis. Positive values are to the right. In the range of -90 to 90. 0
    /// means the tool is perpendicular to the surface and is the default.
    ///
    /// ![Tilt X](https://raw.githubusercontent.com/rust-windowing/winit/master/winit/docs/res/tool_tilt_x.webp)
    ///
    /// <sub>
    ///   For image attribution, see the
    ///   <a href="https://github.com/rust-windowing/winit/blob/master/winit/docs/ATTRIBUTION.md">
    ///     ATTRIBUTION.md
    ///   </a>
    ///   file.
    /// </sub>
    pub x: i8,

    /// The plane angle in degrees between the surface X-Z plane and the plane containing the tool
    /// and the surface X axis. Positive values are towards the user. In the range of -90 to
    /// 90. 0 means the tool is perpendicular to the surface and is the default.
    ///
    /// ![Tilt Y](https://raw.githubusercontent.com/rust-windowing/winit/master/winit/docs/res/tool_tilt_y.webp)
    ///
    /// <sub>
    ///   For image attribution, see the
    ///   <a href="https://github.com/rust-windowing/winit/blob/master/winit/docs/ATTRIBUTION.md">
    ///     ATTRIBUTION.md
    ///   </a>
    ///   file.
    /// </sub>
    pub y: i8,
}

impl Tilt {
    /// Converts this tilt to [`Angle`].
    pub fn angle(self) -> Angle {
        // See <https://www.w3.org/TR/2024/WD-pointerevents3-20240326/#converting-between-tiltx-tilty-and-altitudeangle-azimuthangle>.

        use std::f64::consts::*;

        const PI_0_5: f64 = FRAC_PI_2;
        const PI_1_5: f64 = 3. * FRAC_PI_2;
        const PI_2: f64 = 2. * PI;

        let x = LazyCell::new(|| f64::from(self.x).to_radians());
        let y = LazyCell::new(|| f64::from(self.y).to_radians());

        let mut azimuth = 0.;

        if self.x == 0 {
            match self.y.cmp(&0) {
                Ordering::Greater => azimuth = PI_0_5,
                Ordering::Less => azimuth = PI_1_5,
                Ordering::Equal => (),
            }
        } else if self.y == 0 {
            if self.x < 0 {
                azimuth = PI;
            }
        } else if self.x.abs() == 90 || self.y.abs() == 90 {
            // not enough information to calculate azimuth
            azimuth = 0.;
        } else {
            // Non-boundary case: neither tiltX nor tiltY is equal to 0 or +-90
            azimuth = f64::atan2(y.tan(), x.tan());

            if azimuth < 0. {
                azimuth += PI_2;
            }
        }

        let altitude = if self.x.abs() == 90 || self.y.abs() == 90 {
            0.
        } else if self.x == 0 {
            PI_0_5 - y.abs()
        } else if self.y == 0 {
            PI_0_5 - x.abs()
        } else {
            // Non-boundary case: neither tiltX nor tiltY is equal to 0 or +-90
            f64::atan(1. / f64::sqrt(x.tan().powi(2) + y.tan().powi(2)))
        };

        Angle { altitude, azimuth }
    }
}

/// The angular position of a tablet tool in radians.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Angle {
    /// The altitude angle in radians between the tools perpendicular position to the surface and
    /// the surface X-Y plane. In the range of 0, parallel to the surface, to π/2, perpendicular to
    /// the surface. π/2 means the tool is perpendicular to the surface and is the default.
    ///
    /// ![Altitude angle](https://raw.githubusercontent.com/rust-windowing/winit/master/docs/res/tool_altitude.webp)
    ///
    /// <sub>
    ///   For image attribution, see the
    ///   <a href="https://github.com/rust-windowing/winit/blob/master/docs/res/ATTRIBUTION.md">
    ///     ATTRIBUTION.md
    ///   </a>
    ///   file.
    /// </sub>
    pub altitude: f64,

    /// The azimuth angle in radiants representing the rotation between the major axis of the tool
    /// and the surface X-Y plane. In the range of 0, 3 o'clock, progressively increasing clockwise
    /// to 2π. 0 means the tool is at 3 o'clock or is perpendicular to the surface (`altitude` of
    /// π/2) and is the default.
    ///
    /// ![Azimuth angle](https://raw.githubusercontent.com/rust-windowing/winit/master/docs/res/tool_azimuth.webp)
    ///
    /// <sub>
    ///   For image attribution, see the
    ///   <a href="https://github.com/rust-windowing/winit/blob/master/docs/res/ATTRIBUTION.md">
    ///     ATTRIBUTION.md
    ///   </a>
    ///   file.
    /// </sub>
    pub azimuth: f64,
}

impl Default for Angle {
    fn default() -> Self {
        Self {
            altitude: std::f64::consts::FRAC_2_PI,
            azimuth: 0.0,
        }
    }
}

impl Angle {
    /// Converts this angle to [`Tilt`].
    pub fn tilt(self) -> Tilt {
        // See <https://www.w3.org/TR/2024/WD-pointerevents3-20240326/#converting-between-tiltx-tilty-and-altitudeangle-azimuthangle>.

        use std::f64::consts::*;

        const PI_0_5: f64 = FRAC_PI_2;
        const PI_1_5: f64 = 3. * FRAC_PI_2;
        const PI_2: f64 = 2. * PI;

        let mut x = 0.;
        let mut y = 0.;

        if self.altitude == 0. {
            if self.azimuth == 0. || self.azimuth == PI_2 {
                x = FRAC_PI_2;
            } else if self.azimuth == PI_0_5 {
                y = FRAC_PI_2;
            } else if self.azimuth == PI {
                x = -FRAC_PI_2;
            } else if self.azimuth == PI_1_5 {
                y = -FRAC_PI_2;
            } else if self.azimuth > 0. && self.azimuth < PI_0_5 {
                x = FRAC_PI_2;
                y = FRAC_PI_2;
            } else if self.azimuth > PI_0_5 && self.azimuth < PI {
                x = -FRAC_PI_2;
                y = FRAC_PI_2;
            } else if self.azimuth > PI && self.azimuth < PI_1_5 {
                x = -FRAC_PI_2;
                y = -FRAC_PI_2;
            } else if self.azimuth > PI_1_5 && self.azimuth < PI_2 {
                x = FRAC_PI_2;
                y = -FRAC_PI_2;
            }
        }

        if self.altitude != 0. {
            let altitude = self.altitude.tan();

            x = f64::atan(f64::cos(self.azimuth) / altitude);
            y = f64::atan(f64::sin(self.azimuth) / altitude);
        }

        Tilt {
            x: x.to_degrees().round() as i8,
            y: y.to_degrees().round() as i8,
        }
    }
}

/// A button of a tablet tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Button {
    /// Contact between the tool and tablet surface.
    Contact,

    /// The tool's barrel button.
    Barrel,

    /// Another tablet tool button.
    Other(u16),
}

impl From<Button> for Option<mouse::Button> {
    fn from(button: Button) -> Self {
        Some(match button {
            Button::Contact => mouse::Button::Left,
            Button::Barrel => mouse::Button::Right,
            Button::Other(1) => mouse::Button::Middle,
            Button::Other(3) => mouse::Button::Back,
            Button::Other(4) => mouse::Button::Forward,
            Button::Other(_) => return None,
        })
    }
}
