use crate::widget::canvas::path::{arc, Arc, Path};

use iced_native::{Point, Size};
use lyon::path::builder::SvgPathBuilder;

/// A [`Path`] builder.
///
/// Once a [`Path`] is built, it can no longer be mutated.
#[allow(missing_debug_implementations)]
pub struct Builder {
    raw: lyon::path::builder::WithSvg<lyon::path::path::BuilderImpl>,
}

impl Builder {
    /// Creates a new [`Builder`].
    pub fn new() -> Builder {
        Builder {
            raw: lyon::path::Path::builder().with_svg(),
        }
    }

    /// Moves the starting point of a new sub-path to the given `Point`.
    #[inline]
    pub fn move_to(&mut self, point: Point) {
        let _ = self.raw.move_to(lyon::math::Point::new(point.x, point.y));
    }

    /// Connects the last point in the [`Path`] to the given `Point` with a
    /// straight line.
    #[inline]
    pub fn line_to(&mut self, point: Point) {
        let _ = self.raw.line_to(lyon::math::Point::new(point.x, point.y));
    }

    /// Adds an [`Arc`] to the [`Path`] from `start_angle` to `end_angle` in
    /// a clockwise direction.
    #[inline]
    pub fn arc(&mut self, arc: Arc) {
        self.ellipse(arc.into());
    }

    /// Adds a circular arc to the [`Path`] with the given control points and
    /// radius.
    ///
    /// This essentially draws a straight line segment from the current
    /// position to `a`, but fits a circular arc of `radius` tangent to that
    /// segment and tangent to the line between `a` and `b`.
    ///
    /// With another `.line_to(b)`, the result will be a path connecting the
    /// starting point and `b` with straight line segments towards `a` and a
    /// circular arc smoothing out the corner at `a`.
    ///
    /// See [the HTML5 specification of `arcTo`](https://html.spec.whatwg.org/multipage/canvas.html#building-paths:dom-context-2d-arcto)
    /// for more details and examples.
    pub fn arc_to(&mut self, a: Point, b: Point, radius: f32) {
        use lyon::{math, path};

        let start = self.raw.current_position();
        let mid = math::Point::new(a.x, a.y);
        let end = math::Point::new(b.x, b.y);

        if start == mid || mid == end || radius == 0.0 {
            let _ = self.raw.line_to(mid);
            return;
        }

        let double_area = start.x * (mid.y - end.y)
            + mid.x * (end.y - start.y)
            + end.x * (start.y - mid.y);

        if double_area == 0.0 {
            let _ = self.raw.line_to(mid);
            return;
        }

        let to_start = (start - mid).normalize();
        let to_end = (end - mid).normalize();

        let inner_angle = to_start.dot(to_end).acos();

        let origin_angle = inner_angle / 2.0;

        let origin_adjacent = radius / origin_angle.tan();

        let arc_start = mid + to_start * origin_adjacent;
        let arc_end = mid + to_end * origin_adjacent;

        let sweep = to_start.cross(to_end) < 0.0;

        let _ = self.raw.line_to(arc_start);

        self.raw.arc_to(
            math::Vector::new(radius, radius),
            math::Angle::radians(0.0),
            path::ArcFlags {
                large_arc: false,
                sweep,
            },
            arc_end,
        );
    }

    /// Adds an ellipse to the [`Path`] using a clockwise direction.
    pub fn ellipse(&mut self, arc: arc::Elliptical) {
        use lyon::{geom, math};

        let arc = geom::Arc {
            center: math::Point::new(arc.center.x, arc.center.y),
            radii: math::Vector::new(arc.radii.x, arc.radii.y),
            x_rotation: math::Angle::radians(arc.rotation),
            start_angle: math::Angle::radians(arc.start_angle),
            sweep_angle: math::Angle::radians(arc.end_angle - arc.start_angle),
        };

        let _ = self.raw.move_to(arc.sample(0.0));

        arc.for_each_quadratic_bezier(&mut |curve| {
            let _ = self.raw.quadratic_bezier_to(curve.ctrl, curve.to);
        });
    }

    /// Adds a cubic Bézier curve to the [`Path`] given its two control points
    /// and its end point.
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

    /// Adds a quadratic Bézier curve to the [`Path`] given its control point
    /// and its end point.
    #[inline]
    pub fn quadratic_curve_to(&mut self, control: Point, to: Point) {
        use lyon::math;

        let _ = self.raw.quadratic_bezier_to(
            math::Point::new(control.x, control.y),
            math::Point::new(to.x, to.y),
        );
    }

    /// Adds a rectangle to the [`Path`] given its top-left corner coordinate
    /// and its `Size`.
    #[inline]
    pub fn rectangle(&mut self, top_left: Point, size: Size) {
        self.move_to(top_left);
        self.line_to(Point::new(top_left.x + size.width, top_left.y));
        self.line_to(Point::new(
            top_left.x + size.width,
            top_left.y + size.height,
        ));
        self.line_to(Point::new(top_left.x, top_left.y + size.height));
        self.close();
    }

    /// Adds a circle to the [`Path`] given its center coordinate and its
    /// radius.
    #[inline]
    pub fn circle(&mut self, center: Point, radius: f32) {
        self.arc(Arc {
            center,
            radius,
            start_angle: 0.0,
            end_angle: 2.0 * std::f32::consts::PI,
        });
    }

    /// Closes the current sub-path in the [`Path`] with a straight line to
    /// the starting point.
    #[inline]
    pub fn close(&mut self) {
        self.raw.close()
    }

    /// Builds the [`Path`] of this [`Builder`].
    #[inline]
    pub fn build(self) -> Path {
        Path {
            raw: self.raw.build(),
        }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}
