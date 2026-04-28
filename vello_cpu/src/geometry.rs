use crate::core::text::LineHeight;
use crate::core::{self, Pixels, Point, Radians, Rectangle, Size, Svg, Vector};
use crate::graphics::cache::{self, Cached};
use crate::graphics::geometry::fill::{self, Fill};
use crate::graphics::geometry::stroke::{self, Stroke};
use crate::graphics::geometry::{self, Path, Style};
use crate::graphics::{self, Gradient, Image, Text};

use vello_cpu::kurbo;
use vello_cpu::peniko;

use std::sync::Arc;

#[derive(Debug)]
pub enum Geometry {
    Live {
        text: Vec<Text>,
        images: Vec<graphics::Image>,
        primitives: Vec<Primitive>,
        clip_bounds: Rectangle,
    },
    Cache(Cache),
}

#[derive(Debug, Clone)]
pub enum Primitive {
    /// A path filled with some paint.
    Fill {
        /// The path to fill.
        path: kurbo::BezPath,
        /// The paint to use.
        paint: vello_cpu::PaintType,
        /// The fill rule to follow.
        rule: peniko::Fill,
    },
    /// A path stroked with some paint.
    Stroke {
        /// The path to stroke.
        path: kurbo::BezPath,
        /// The paint to use.
        paint: vello_cpu::PaintType,
        /// The stroke settings.
        stroke: kurbo::Stroke,
    },
}

impl Primitive {
    /// Returns the visible bounds of the [`Primitive`].
    pub fn visible_bounds(&self) -> Rectangle {
        let bounds = match self {
            Primitive::Fill { path, .. } => path.control_box(),
            Primitive::Stroke { path, .. } => path.control_box(),
        };

        Rectangle {
            x: bounds.x0 as f32,
            y: bounds.y0 as f32,
            width: (bounds.x1 - bounds.x0) as f32,
            height: (bounds.y1 - bounds.y0) as f32,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cache {
    pub text: Arc<[Text]>,
    pub images: Arc<[graphics::Image]>,
    pub primitives: Arc<[Primitive]>,
    pub clip_bounds: Rectangle,
}

impl Cached for Geometry {
    type Cache = Cache;

    fn load(cache: &Cache) -> Self {
        Self::Cache(cache.clone())
    }

    fn cache(self, _group: cache::Group, _previous: Option<Cache>) -> Cache {
        match self {
            Self::Live {
                primitives,
                images,
                text,
                clip_bounds,
            } => Cache {
                primitives: Arc::from(primitives),
                images: Arc::from(images),
                text: Arc::from(text),
                clip_bounds,
            },
            Self::Cache(cache) => cache,
        }
    }
}

#[derive(Debug)]
pub struct Frame {
    clip_bounds: Rectangle,
    transform: kurbo::Affine,
    stack: Vec<kurbo::Affine>,
    primitives: Vec<Primitive>,
    images: Vec<graphics::Image>,
    text: Vec<Text>,
}

impl Frame {
    pub fn new(bounds: Rectangle) -> Self {
        Self {
            clip_bounds: bounds,
            stack: Vec::new(),
            primitives: Vec::new(),
            images: Vec::new(),
            text: Vec::new(),
            transform: kurbo::Affine::IDENTITY,
        }
    }
}

impl geometry::frame::Backend for Frame {
    type Geometry = Geometry;

    fn width(&self) -> f32 {
        self.clip_bounds.width
    }

    fn height(&self) -> f32 {
        self.clip_bounds.height
    }

    fn size(&self) -> Size {
        self.clip_bounds.size()
    }

    fn center(&self) -> Point {
        Point::new(self.clip_bounds.width / 2.0, self.clip_bounds.height / 2.0)
    }

    fn fill(&mut self, path: &Path, fill: impl Into<Fill>) {
        let path = self.transform * convert_path(path);
        let fill = fill.into();

        self.primitives.push(Primitive::Fill {
            path,
            paint: into_paint(fill.style, self.transform),
            rule: into_fill_rule(fill.rule),
        });
    }

    fn fill_rectangle(&mut self, top_left: Point, size: Size, fill: impl Into<Fill>) {
        let path = self.transform * convert_path(&Path::rectangle(top_left, size));
        let fill = fill.into();

        self.primitives.push(Primitive::Fill {
            path,
            paint: into_paint(fill.style, self.transform),
            rule: into_fill_rule(fill.rule),
        });
    }

    fn stroke<'a>(&mut self, path: &Path, stroke: impl Into<Stroke<'a>>) {
        let path = self.transform * convert_path(path);
        let stroke = stroke.into();

        self.primitives.push(Primitive::Stroke {
            path,
            paint: into_paint(stroke.style, self.transform),
            stroke: into_stroke(&stroke),
        });
    }

    fn stroke_rectangle<'a>(&mut self, top_left: Point, size: Size, stroke: impl Into<Stroke<'a>>) {
        self.stroke(&Path::rectangle(top_left, size), stroke);
    }

    fn fill_text(&mut self, text: impl Into<geometry::Text>) {
        let text = text.into();

        let coefficients = self.transform.as_coeffs();

        let (scale_x, scale_y) = (coefficients[0] as f32, coefficients[3] as f32);
        let has_skew = coefficients[1] != 0.0 || coefficients[2] != 0.0;

        if !has_skew && scale_x == scale_y && scale_x > 0.0 && scale_y > 0.0 {
            let (bounds, size, line_height) = if self.transform == kurbo::Affine::IDENTITY {
                (
                    Rectangle::new(text.position, Size::new(text.max_width, f32::INFINITY)),
                    text.size,
                    text.line_height,
                )
            } else {
                let position =
                    self.transform * kurbo::Point::from((text.position.x, text.position.y));

                let size = text.size.0 * scale_y;

                let line_height = match text.line_height {
                    LineHeight::Absolute(size) => LineHeight::Absolute(Pixels(size.0 * scale_y)),
                    LineHeight::Relative(factor) => LineHeight::Relative(factor),
                };

                (
                    Rectangle {
                        x: position.x as f32,
                        y: position.y as f32,
                        width: text.max_width * scale_x,
                        height: f32::INFINITY,
                    },
                    size.into(),
                    line_height,
                )
            };

            // TODO: Honor layering!
            self.text.push(Text::Cached {
                content: text.content,
                bounds,
                color: text.color,
                size,
                line_height: line_height.to_absolute(size),
                font: text.font,
                align_x: text.align_x,
                align_y: text.align_y,
                shaping: text.shaping,
                wrapping: text.wrapping,
                ellipsis: text.ellipsis,
                clip_bounds: Rectangle::with_size(Size::INFINITE),
            });
        } else {
            text.draw_with(|path, color| self.fill(&path, color));
        }
    }

    fn stroke_text<'a>(&mut self, text: impl Into<geometry::Text>, stroke: impl Into<Stroke<'a>>) {
        let text = text.into();
        let stroke = stroke.into();

        text.draw_with(|path, _color| self.stroke(&path, stroke));
    }

    fn push_transform(&mut self) {
        self.stack.push(self.transform);
    }

    fn pop_transform(&mut self) {
        self.transform = self.stack.pop().expect("Pop transform");
    }

    fn draft(&mut self, clip_bounds: Rectangle) -> Self {
        Self::new(clip_bounds)
    }

    fn paste(&mut self, frame: Self) {
        self.primitives.extend(frame.primitives);
        self.text.extend(frame.text);
        self.images.extend(frame.images);
    }

    fn translate(&mut self, translation: Vector) {
        self.transform = self.transform.pre_translate(kurbo::Vec2 {
            x: f64::from(translation.x),
            y: f64::from(translation.y),
        });
    }

    fn rotate(&mut self, angle: impl Into<Radians>) {
        self.transform = self.transform.pre_rotate(f64::from(angle.into().0));
    }

    fn scale(&mut self, scale: impl Into<f32>) {
        let scale = scale.into();

        self.scale_nonuniform(Vector { x: scale, y: scale });
    }

    fn scale_nonuniform(&mut self, scale: impl Into<Vector>) {
        let scale = scale.into();

        self.transform = self
            .transform
            .pre_scale_non_uniform(f64::from(scale.x), f64::from(scale.y));
    }

    fn into_geometry(self) -> Geometry {
        Geometry::Live {
            primitives: self.primitives,
            images: self.images,
            text: self.text,
            clip_bounds: self.clip_bounds,
        }
    }

    fn draw_image(&mut self, bounds: Rectangle, image: impl Into<core::Image>) {
        let mut image = image.into();

        let (bounds, external_rotation) = transform_rectangle(bounds, self.transform);

        image.rotation += external_rotation;

        self.images.push(graphics::Image::Raster {
            image,
            bounds,
            clip_bounds: self.clip_bounds,
        });
    }

    fn draw_svg(&mut self, bounds: Rectangle, svg: impl Into<Svg>) {
        let mut svg = svg.into();

        let (bounds, external_rotation) = transform_rectangle(bounds, self.transform);

        svg.rotation += external_rotation;

        self.images.push(Image::Vector {
            svg,
            bounds,
            clip_bounds: self.clip_bounds,
        });
    }
}

fn transform_rectangle(rectangle: Rectangle, transform: kurbo::Affine) -> (Rectangle, Radians) {
    let top_left = transform * kurbo::Point::from((rectangle.x, rectangle.y));
    let top_right = transform * kurbo::Point::from((rectangle.x + rectangle.width, rectangle.y));
    let bottom_left = transform * kurbo::Point::from((rectangle.x, rectangle.y + rectangle.height));

    Rectangle::with_vertices(
        Point::new(top_left.x as f32, top_left.y as f32),
        Point::new(top_right.x as f32, top_right.y as f32),
        Point::new(bottom_left.x as f32, bottom_left.y as f32),
    )
}

fn convert_path(path: &Path) -> kurbo::BezPath {
    use iced_graphics::geometry::path::lyon_path;

    let mut builder = kurbo::BezPath::new();
    let mut last_point = lyon_path::math::Point::default();

    for event in path.raw() {
        match event {
            lyon_path::Event::Begin { at } => {
                builder.move_to((at.x, at.y));

                last_point = at;
            }
            lyon_path::Event::Line { from, to } => {
                if last_point != from {
                    builder.move_to((from.x, from.y));
                }

                builder.line_to((to.x, to.y));

                last_point = to;
            }
            lyon_path::Event::Quadratic { from, ctrl, to } => {
                if last_point != from {
                    builder.move_to((from.x, from.y));
                }

                builder.quad_to((ctrl.x, ctrl.y), (to.x, to.y));

                last_point = to;
            }
            lyon_path::Event::Cubic {
                from,
                ctrl1,
                ctrl2,
                to,
            } => {
                if last_point != from {
                    builder.move_to((from.x, from.y));
                }

                builder.curve_to((ctrl1.x, ctrl1.y), (ctrl2.x, ctrl2.y), (to.x, to.y));

                last_point = to;
            }
            lyon_path::Event::End { close, .. } => {
                if close {
                    builder.close_path();
                }
            }
        }
    }

    builder
}

pub fn into_paint(style: Style, _transform: kurbo::Affine) -> vello_cpu::PaintType {
    match style {
        Style::Solid(color) => vello_cpu::PaintType::Solid(crate::into_color(color)),
        Style::Gradient(gradient) => match gradient {
            Gradient::Linear(_linear) => {
                // let stops: Vec<tiny_skia::GradientStop> = linear
                //     .stops
                //     .into_iter()
                //     .flatten()
                //     .map(|stop| {
                //         tiny_skia::GradientStop::new(
                //             stop.offset,
                //             tiny_skia::Color::from_rgba(
                //                 stop.color.b,
                //                 stop.color.g,
                //                 stop.color.r,
                //                 stop.color.a,
                //             )
                //             .expect("Create color"),
                //         )
                //     })
                //     .collect();

                // tiny_skia::LinearGradient::new(
                //     tiny_skia::Point {
                //         x: linear.start.x,
                //         y: linear.start.y,
                //     },
                //     tiny_skia::Point {
                //         x: linear.end.x,
                //         y: linear.end.y,
                //     },
                //     if stops.is_empty() {
                //         vec![tiny_skia::GradientStop::new(0.0, tiny_skia::Color::BLACK)]
                //     } else {
                //         stops
                //     },
                //     tiny_skia::SpreadMode::Pad,
                //     tiny_skia::Transform::identity(),
                // )
                // .expect("Create linear gradient")
                //
                // if let vello_cpu::PaintType::Gradient(gradient) = &mut paint {
                //     gradient.kind = match gradient.kind {
                //         peniko::GradientKind::Linear(position) => {
                //             peniko::GradientKind::Linear(peniko::LinearGradientPosition {
                //                 start: self.transform * position.start,
                //                 end: self.transform * position.end,
                //             })
                //         }
                //         peniko::GradientKind::Radial(position) => {
                //             let scale = self.transform.as_coeffs()[0] as f32;

                //             peniko::GradientKind::Radial(peniko::RadialGradientPosition {
                //                 start_center: self.transform * position.start_center,
                //                 start_radius: scale * position.start_radius,
                //                 end_center: self.transform * position.end_center,
                //                 end_radius: scale * position.end_radius,
                //             })
                //         }
                //         peniko::GradientKind::Sweep(position) => {
                //             peniko::GradientKind::Sweep(peniko::SweepGradientPosition {
                //                 center: self.transform * position.center,
                //                 start_angle: position.start_angle,
                //                 end_angle: position.end_angle,
                //             })
                //         }
                //     };
                // }
                todo!()
            }
        },
    }
}

pub fn into_fill_rule(rule: fill::Rule) -> peniko::Fill {
    match rule {
        fill::Rule::EvenOdd => peniko::Fill::EvenOdd,
        fill::Rule::NonZero => peniko::Fill::NonZero,
    }
}

pub fn into_stroke(stroke: &Stroke<'_>) -> kurbo::Stroke {
    let line_cap = match stroke.line_cap {
        stroke::LineCap::Butt => kurbo::Cap::Butt,
        stroke::LineCap::Square => kurbo::Cap::Square,
        stroke::LineCap::Round => kurbo::Cap::Round,
    };

    kurbo::Stroke {
        width: f64::from(stroke.width),
        start_cap: line_cap,
        join: match stroke.line_join {
            stroke::LineJoin::Miter => kurbo::Join::Miter,
            stroke::LineJoin::Round => kurbo::Join::Round,
            stroke::LineJoin::Bevel => kurbo::Join::Bevel,
        },
        dash_pattern: stroke
            .line_dash
            .segments
            .iter()
            .copied()
            .map(f64::from)
            .collect(),
        dash_offset: stroke.line_dash.offset as f64,
        ..Default::default()
    }
}
