use iced::Point;

use lyon_algorithms::measure::PathMeasurements;
use lyon_algorithms::path::{builder::NoAttributes, path::BuilderImpl, Path};
use once_cell::sync::Lazy;

pub static EMPHASIZED: Lazy<Easing> = Lazy::new(|| {
    Easing::builder()
        .cubic_bezier_to([0.05, 0.0], [0.133333, 0.06], [0.166666, 0.4])
        .cubic_bezier_to([0.208333, 0.82], [0.25, 1.0], [1.0, 1.0])
        .build()
});

pub static EMPHASIZED_DECELERATE: Lazy<Easing> = Lazy::new(|| {
    Easing::builder()
        .cubic_bezier_to([0.05, 0.7], [0.1, 1.0], [1.0, 1.0])
        .build()
});

pub static EMPHASIZED_ACCELERATE: Lazy<Easing> = Lazy::new(|| {
    Easing::builder()
        .cubic_bezier_to([0.3, 0.0], [0.8, 0.15], [1.0, 1.0])
        .build()
});

pub static STANDARD: Lazy<Easing> = Lazy::new(|| {
    Easing::builder()
        .cubic_bezier_to([0.2, 0.0], [0.0, 1.0], [1.0, 1.0])
        .build()
});

pub static STANDARD_DECELERATE: Lazy<Easing> = Lazy::new(|| {
    Easing::builder()
        .cubic_bezier_to([0.0, 0.0], [0.0, 1.0], [1.0, 1.0])
        .build()
});

pub static STANDARD_ACCELERATE: Lazy<Easing> = Lazy::new(|| {
    Easing::builder()
        .cubic_bezier_to([0.3, 0.0], [1.0, 1.0], [1.0, 1.0])
        .build()
});

pub struct Easing {
    path: Path,
    measurements: PathMeasurements,
}

impl Easing {
    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn y_at_x(&self, x: f32) -> f32 {
        let mut sampler = self.measurements.create_sampler(
            &self.path,
            lyon_algorithms::measure::SampleType::Normalized,
        );
        let sample = sampler.sample(x);

        sample.position().y
    }
}

pub struct Builder(NoAttributes<BuilderImpl>);

impl Builder {
    pub fn new() -> Self {
        let mut builder = Path::builder();
        builder.begin(lyon_algorithms::geom::point(0.0, 0.0));

        Self(builder)
    }

    /// Adds a line segment. Points must be between 0,0 and 1,1
    pub fn line_to(mut self, to: impl Into<Point>) -> Self {
        self.0.line_to(Self::point(to));

        self
    }

    /// Adds a quadratic bézier curve. Points must be between 0,0 and 1,1
    pub fn quadratic_bezier_to(
        mut self,
        ctrl: impl Into<Point>,
        to: impl Into<Point>,
    ) -> Self {
        self.0
            .quadratic_bezier_to(Self::point(ctrl), Self::point(to));

        self
    }

    /// Adds a cubic bézier curve. Points must be between 0,0 and 1,1
    pub fn cubic_bezier_to(
        mut self,
        ctrl1: impl Into<Point>,
        ctrl2: impl Into<Point>,
        to: impl Into<Point>,
    ) -> Self {
        self.0.cubic_bezier_to(
            Self::point(ctrl1),
            Self::point(ctrl2),
            Self::point(to),
        );

        self
    }

    pub fn build(mut self) -> Easing {
        self.0.line_to(lyon_algorithms::geom::point(1.0, 1.0));
        self.0.end(false);

        let path = self.0.build();
        let measurements = PathMeasurements::from_path(&path, 0.0);

        Easing { path, measurements }
    }

    fn point(p: impl Into<Point>) -> lyon_algorithms::geom::Point<f32> {
        let p: Point = p.into();
        lyon_algorithms::geom::point(p.x.clamp(0.0, 1.0), p.y.clamp(0.0, 1.0))
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}
