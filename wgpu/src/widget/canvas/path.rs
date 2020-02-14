use iced_native::{Point, Size, Vector};

use lyon::path::builder::{Build, FlatPathBuilder, PathBuilder, SvgBuilder};

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
    raw: lyon::path::builder::SvgPathBuilder<lyon::path::Builder>,
}

impl Builder {
    pub fn new() -> Builder {
        Builder {
            raw: lyon::path::Path::builder().with_svg(),
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

    pub fn arc_to(&mut self, a: Point, b: Point, radius: f32) {
        use lyon::{math, path};

        let a = math::Point::new(a.x, a.y);

        if self.raw.current_position() != a {
            let _ = self.raw.line_to(a);
        }

        let _ = self.raw.arc_to(
            math::Vector::new(radius, radius),
            math::Angle::radians(0.0),
            path::ArcFlags::default(),
            math::Point::new(b.x, b.y),
        );
    }

    pub fn ellipse(&mut self, ellipse: Ellipse) {
        use lyon::{geom, math};

        let arc = geom::Arc {
            center: math::Point::new(ellipse.center.x, ellipse.center.y),
            radii: math::Vector::new(ellipse.radii.x, ellipse.radii.y),
            x_rotation: math::Angle::radians(ellipse.rotation),
            start_angle: math::Angle::radians(ellipse.start_angle),
            sweep_angle: math::Angle::radians(ellipse.end_angle),
        };

        let _ = self.raw.move_to(arc.sample(0.0));

        arc.for_each_quadratic_bezier(&mut |curve| {
            let _ = self.raw.quadratic_bezier_to(curve.ctrl, curve.to);
        });
    }

    #[inline]
    pub fn bezier_curve_to(
        &mut self,
        control_a: Point,
        control_b: Point,
        to: Point,
    ) {
        use lyon::math;

        let _ = self.raw.cubic_bezier_to(
            math::Point::new(control_a.x, control_a.y),
            math::Point::new(control_b.x, control_b.y),
            math::Point::new(to.x, to.y),
        );
    }

    #[inline]
    pub fn quadratic_curve_to(&mut self, control: Point, to: Point) {
        use lyon::math;

        let _ = self.raw.quadratic_bezier_to(
            math::Point::new(control.x, control.y),
            math::Point::new(to.x, to.y),
        );
    }

    #[inline]
    pub fn rectangle(&mut self, p: Point, size: Size) {
        self.move_to(p);
        self.line_to(Point::new(p.x + size.width, p.y));
        self.line_to(Point::new(p.x + size.width, p.y + size.height));
        self.line_to(Point::new(p.x, p.y + size.height));
        self.close();
    }

    #[inline]
    pub fn circle(&mut self, center: Point, radius: f32) {
        self.arc(Arc {
            center,
            radius,
            start_angle: 0.0,
            end_angle: 2.0 * std::f32::consts::PI,
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
