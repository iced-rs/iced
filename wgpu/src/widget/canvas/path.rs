use iced_native::{Point, Vector};

#[derive(Debug, Clone)]
pub struct Path {
    raw: lyon::path::Path,
}

impl Path {
    pub fn new(f: impl FnOnce(&mut Builder)) -> Self {
        let mut builder = Builder::new();

        // TODO: Make it pure instead of side-effect-based (?)
        f(&mut builder);

        builder.build()
    }

    #[inline]
    pub(crate) fn raw(&self) -> &lyon::path::Path {
        &self.raw
    }
}

#[allow(missing_debug_implementations)]
pub struct Builder {
    raw: lyon::path::Builder,
}

impl Builder {
    pub fn new() -> Builder {
        Builder {
            raw: lyon::path::Path::builder(),
        }
    }

    #[inline]
    pub fn move_to(&mut self, point: Point) {
        let _ = self.raw.move_to(lyon::math::Point::new(point.x, point.y));
    }

    #[inline]
    pub fn line_to(&mut self, point: Point) {
        let _ = self.raw.line_to(lyon::math::Point::new(point.x, point.y));
    }

    #[inline]
    pub fn arc(&mut self, arc: Arc) {
        self.ellipse(arc.into());
    }

    #[inline]
    pub fn ellipse(&mut self, ellipse: Ellipse) {
        let arc = lyon::geom::Arc {
            center: lyon::math::Point::new(ellipse.center.x, ellipse.center.y),
            radii: lyon::math::Vector::new(ellipse.radii.x, ellipse.radii.y),
            x_rotation: lyon::math::Angle::radians(ellipse.rotation),
            start_angle: lyon::math::Angle::radians(ellipse.start_angle),
            sweep_angle: lyon::math::Angle::radians(ellipse.end_angle),
        };

        arc.for_each_quadratic_bezier(&mut |curve| {
            let _ = self.raw.quadratic_bezier_to(curve.ctrl, curve.to);
        });
    }

    #[inline]
    pub fn close(&mut self) {
        self.raw.close()
    }

    #[inline]
    pub fn build(self) -> Path {
        Path {
            raw: self.raw.build(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Arc {
    pub center: Point,
    pub radius: f32,
    pub start_angle: f32,
    pub end_angle: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Ellipse {
    pub center: Point,
    pub radii: Vector,
    pub rotation: f32,
    pub start_angle: f32,
    pub end_angle: f32,
}

impl From<Arc> for Ellipse {
    fn from(arc: Arc) -> Ellipse {
        Ellipse {
            center: arc.center,
            radii: Vector::new(arc.radius, arc.radius),
            rotation: 0.0,
            start_angle: arc.start_angle,
            end_angle: arc.end_angle,
        }
    }
}
