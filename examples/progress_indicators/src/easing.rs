use flo_curves::bezier::Curve;
use flo_curves::*;
use lazy_static::lazy_static;

use std::borrow::Cow;

lazy_static! {
    pub static ref EXAMPLES: [Easing; 3] = [
        Easing::CubicBezier(Curve::from_points(
            Coord2(0.0, 0.0),
            (Coord2(0.05, 0.7), Coord2(0.1, 1.0)),
            Coord2(1.0, 1.0),
        )),
        Easing::CubicBezier(Curve::from_points(
            Coord2(0.0, 0.0),
            (Coord2(0.3, 0.0), Coord2(0.8, 0.15)),
            Coord2(1.0, 1.0),
        )),
        Easing::CubicBezier(Curve::from_points(
            Coord2(0.0, 0.0),
            (Coord2(0.2, 0.0), Coord2(0.0, 1.0)),
            Coord2(1.0, 1.0),
        ))
    ];
    pub static ref STANDARD: Easing = {
        Easing::CubicBezier(Curve::from_points(
            Coord2(0.0, 0.0),
            (Coord2(0.2, 0.0), Coord2(0.0, 1.0)),
            Coord2(1.0, 1.0),
        ))
    };
}

#[derive(Clone, Debug)]
pub enum Easing {
    BezierPath(Vec<Curve<Coord2>>),
    CubicBezier(Curve<Coord2>),
}

impl Easing {
    pub fn y_at_x(&self, x: f32) -> f32 {
        let x = x as f64;

        match self {
            Self::BezierPath(curves) => curves
                .iter()
                .find_map(|curve| {
                    (curve.start_point().0 <= x && curve.end_point().0 >= x)
                        .then(|| curve.point_at_pos(x).1 as f32)
                })
                .unwrap_or_default(),
            Self::CubicBezier(curve) => curve.point_at_pos(x).1 as f32,
        }
    }
}

impl<'a> From<Easing> for Cow<'a, Easing> {
    fn from(easing: Easing) -> Self {
        Cow::Owned(easing)
    }
}

impl<'a> From<&'a Easing> for Cow<'a, Easing> {
    fn from(easing: &'a Easing) -> Self {
        Cow::Borrowed(easing)
    }
}
