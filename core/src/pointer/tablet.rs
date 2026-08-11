use super::Force;

/// The kind of tool used with a tablet.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TabletToolKind {
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
pub struct TabletToolData {
    /// The force applied to the tool against the surface.
    pub force: Option<Force>,
    /// The normalized tangential, or barrel, pressure in the range `-1.0..=1.0`.
    pub tangential_force: Option<f32>,
    /// The clockwise rotation of the tool in degrees, in the range `0..=359`.
    pub twist: Option<u16>,
    /// The plane angle of the tool in degrees.
    pub tilt: Option<TabletToolTilt>,
    /// The angular position of the tool in radians.
    pub angle: Option<TabletToolAngle>,
}

/// The plane angle of a tablet tool in degrees.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TabletToolTilt {
    /// The angle between the surface Y-Z plane and the tool's surface Y plane.
    pub x: i8,
    /// The angle between the surface X-Z plane and the tool's surface X plane.
    pub y: i8,
}

/// The angular position of a tablet tool in radians.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TabletToolAngle {
    /// The angle between the tool and the surface X-Y plane.
    pub altitude: f64,
    /// The clockwise rotation between the tool's major axis and the surface X-Y plane.
    pub azimuth: f64,
}

impl Default for TabletToolAngle {
    fn default() -> Self {
        Self {
            altitude: std::f64::consts::FRAC_2_PI,
            azimuth: 0.0,
        }
    }
}
