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
    pub force: Option<Force>,

    /// The normalized tangential, or barrel, pressure in the range `-1.0..=1.0`.
    pub tangential_force: Option<f32>,

    /// The clockwise rotation of the tool in degrees, in the range `0..=359`.
    pub twist: Option<u16>,

    /// The plane angle of the tool in degrees.
    pub tilt: Option<Tilt>,

    /// The angular position of the tool in radians.
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
    /// The angle between the surface Y-Z plane and the tool's surface Y plane.
    pub x: i8,

    /// The angle between the surface X-Z plane and the tool's surface X plane.
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
    /// The angle between the tool and the surface X-Y plane.
    pub altitude: f64,

    /// The clockwise rotation between the tool's major axis and the surface X-Y plane.
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
