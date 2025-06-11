use crate::Primitive;
use crate::core::text::LineHeight;
use crate::core::{self, Pixels, Point, Radians, Rectangle, Size, Svg, Vector};
use crate::graphics::cache::{self, Cached};
use crate::graphics::geometry::fill::{self, Fill};
use crate::graphics::geometry::stroke::{self, Stroke};
use crate::graphics::geometry::{self, Path, Style};
use crate::graphics::{self, Gradient, Image, Text};

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
    transform: tiny_skia::Transform,
    stack: Vec<tiny_skia::Transform>,
    primitives: Vec<Primitive>,
    images: Vec<graphics::Image>,
    text: Vec<Text>,
}

impl Frame {
    pub fn new(size: Size) -> Self {
        Self::with_clip(Rectangle::with_size(size))
    }

    pub fn with_clip(clip_bounds: Rectangle) -> Self {
        Self {
            clip_bounds,
            stack: Vec::new(),
            primitives: Vec::new(),
            images: Vec::new(),
            text: Vec::new(),
            transform: tiny_skia::Transform::from_translate(
                clip_bounds.x,
                clip_bounds.y,
            ),
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
        let Some(path) =
            convert_path(path).and_then(|path| path.transform(self.transform))
        else {
            return;
        };

        let fill = fill.into();

        let mut paint = into_paint(fill.style);
        paint.shader.transform(self.transform);

        self.primitives.push(Primitive::Fill {
            path,
            paint,
            rule: into_fill_rule(fill.rule),
        });
    }

    fn fill_rectangle(
        &mut self,
        top_left: Point,
        size: Size,
        fill: impl Into<Fill>,
    ) {
        let Some(path) = convert_path(&Path::rectangle(top_left, size))
            .and_then(|path| path.transform(self.transform))
        else {
            return;
        };

        let fill = fill.into();

        let mut paint = tiny_skia::Paint {
            anti_alias: false,
            ..into_paint(fill.style)
        };
        paint.shader.transform(self.transform);

        self.primitives.push(Primitive::Fill {
            path,
            paint,
            rule: into_fill_rule(fill.rule),
        });
    }

    fn stroke<'a>(&mut self, path: &Path, stroke: impl Into<Stroke<'a>>) {
        let Some(path) =
            convert_path(path).and_then(|path| path.transform(self.transform))
        else {
            return;
        };

        let stroke = stroke.into();
        let skia_stroke = into_stroke(&stroke);

        let mut paint = into_paint(stroke.style);
        paint.shader.transform(self.transform);

        self.primitives.push(Primitive::Stroke {
            path,
            paint,
            stroke: skia_stroke,
        });
    }

    fn stroke_rectangle<'a>(
        &mut self,
        top_left: Point,
        size: Size,
        stroke: impl Into<Stroke<'a>>,
    ) {
        self.stroke(&Path::rectangle(top_left, size), stroke);
    }

    fn fill_text(&mut self, text: impl Into<geometry::Text>) {
        let text = text.into();

        let (scale_x, scale_y) = self.transform.get_scale();

        if !self.transform.has_skew()
            && scale_x == scale_y
            && scale_x > 0.0
            && scale_y > 0.0
        {
            let (bounds, size, line_height) = if self.transform.is_identity() {
                (
                    Rectangle::new(
                        text.position,
                        Size::new(text.max_width, f32::INFINITY),
                    ),
                    text.size,
                    text.line_height,
                )
            } else {
                let mut position = [tiny_skia::Point {
                    x: text.position.x,
                    y: text.position.y,
                }];

                self.transform.map_points(&mut position);

                let size = text.size.0 * scale_y;

                let line_height = match text.line_height {
                    LineHeight::Absolute(size) => {
                        LineHeight::Absolute(Pixels(size.0 * scale_y))
                    }
                    LineHeight::Relative(factor) => {
                        LineHeight::Relative(factor)
                    }
                };

                (
                    Rectangle {
                        x: position[0].x,
                        y: position[0].y,
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
                clip_bounds: Rectangle::with_size(Size::INFINITY),
            });
        } else {
            text.draw_with(|path, color| self.fill(&path, color));
        }
    }

    fn stroke_text<'a>(
        &mut self,
        text: impl Into<geometry::Text>,
        stroke: impl Into<Stroke<'a>>,
    ) {
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
        Self::with_clip(clip_bounds)
    }

    fn paste(&mut self, frame: Self) {
        self.primitives.extend(frame.primitives);
        self.text.extend(frame.text);
        self.images.extend(frame.images);
    }

    fn translate(&mut self, translation: Vector) {
        self.transform =
            self.transform.pre_translate(translation.x, translation.y);
    }

    fn rotate(&mut self, angle: impl Into<Radians>) {
        self.transform = self.transform.pre_concat(
            tiny_skia::Transform::from_rotate(angle.into().0.to_degrees()),
        );
    }

    fn scale(&mut self, scale: impl Into<f32>) {
        let scale = scale.into();

        self.scale_nonuniform(Vector { x: scale, y: scale });
    }

    fn scale_nonuniform(&mut self, scale: impl Into<Vector>) {
        let scale = scale.into();

        self.transform = self.transform.pre_scale(scale.x, scale.y);
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

        let (bounds, external_rotation) =
            transform_rectangle(bounds, self.transform);

        image.rotation += external_rotation;

        self.images.push(graphics::Image::Raster(image, bounds));
    }

    fn draw_svg(&mut self, bounds: Rectangle, svg: impl Into<Svg>) {
        let mut svg = svg.into();

        let (bounds, external_rotation) =
            transform_rectangle(bounds, self.transform);

        svg.rotation += external_rotation;

        self.images.push(Image::Vector(svg, bounds));
    }
}

fn transform_rectangle(
    rectangle: Rectangle,
    transform: tiny_skia::Transform,
) -> (Rectangle, Radians) {
    let mut top_left = tiny_skia::Point {
        x: rectangle.x,
        y: rectangle.y,
    };

    let mut top_right = tiny_skia::Point {
        x: rectangle.x + rectangle.width,
        y: rectangle.y,
    };

    let mut bottom_left = tiny_skia::Point {
        x: rectangle.x,
        y: rectangle.y + rectangle.height,
    };

    transform.map_point(&mut top_left);
    transform.map_point(&mut top_right);
    transform.map_point(&mut bottom_left);

    Rectangle::with_vertices(
        Point::new(top_left.x, top_left.y),
        Point::new(top_right.x, top_right.y),
        Point::new(bottom_left.x, bottom_left.y),
    )
}

fn convert_path(path: &Path) -> Option<tiny_skia::Path> {
    use iced_graphics::geometry::path::lyon_path;

    let mut builder = tiny_skia::PathBuilder::new();
    let mut last_point = lyon_path::math::Point::default();

    for event in path.raw() {
        match event {
            lyon_path::Event::Begin { at } => {
                builder.move_to(at.x, at.y);

                last_point = at;
            }
            lyon_path::Event::Line { from, to } => {
                if last_point != from {
                    builder.move_to(from.x, from.y);
                }

                builder.line_to(to.x, to.y);

                last_point = to;
            }
            lyon_path::Event::Quadratic { from, ctrl, to } => {
                if last_point != from {
                    builder.move_to(from.x, from.y);
                }

                builder.quad_to(ctrl.x, ctrl.y, to.x, to.y);

                last_point = to;
            }
            lyon_path::Event::Cubic {
                from,
                ctrl1,
                ctrl2,
                to,
            } => {
                if last_point != from {
                    builder.move_to(from.x, from.y);
                }

                builder
                    .cubic_to(ctrl1.x, ctrl1.y, ctrl2.x, ctrl2.y, to.x, to.y);

                last_point = to;
            }
            lyon_path::Event::End { close, .. } => {
                if close {
                    builder.close();
                }
            }
        }
    }

    let result = builder.finish();

    #[cfg(debug_assertions)]
    if result.is_none() {
        log::warn!("Invalid path: {:?}", path.raw());
    }

    result
}

pub fn into_paint(style: Style) -> tiny_skia::Paint<'static> {
    tiny_skia::Paint {
        shader: match style {
            Style::Solid(color) => tiny_skia::Shader::SolidColor(
                tiny_skia::Color::from_rgba(color.b, color.g, color.r, color.a)
                    .expect("Create color"),
            ),
            Style::Gradient(gradient) => match gradient {
                Gradient::Linear(linear) => {
                    let stops: Vec<tiny_skia::GradientStop> = linear
                        .stops
                        .into_iter()
                        .flatten()
                        .map(|stop| {
                            tiny_skia::GradientStop::new(
                                stop.offset,
                                tiny_skia::Color::from_rgba(
                                    stop.color.b,
                                    stop.color.g,
                                    stop.color.r,
                                    stop.color.a,
                                )
                                .expect("Create color"),
                            )
                        })
                        .collect();

                    tiny_skia::LinearGradient::new(
                        tiny_skia::Point {
                            x: linear.start.x,
                            y: linear.start.y,
                        },
                        tiny_skia::Point {
                            x: linear.end.x,
                            y: linear.end.y,
                        },
                        if stops.is_empty() {
                            vec![tiny_skia::GradientStop::new(
                                0.0,
                                tiny_skia::Color::BLACK,
                            )]
                        } else {
                            stops
                        },
                        tiny_skia::SpreadMode::Pad,
                        tiny_skia::Transform::identity(),
                    )
                    .expect("Create linear gradient")
                }
            },
        },
        anti_alias: true,
        ..Default::default()
    }
}

pub fn into_fill_rule(rule: fill::Rule) -> tiny_skia::FillRule {
    match rule {
        fill::Rule::EvenOdd => tiny_skia::FillRule::EvenOdd,
        fill::Rule::NonZero => tiny_skia::FillRule::Winding,
    }
}

pub fn into_stroke(stroke: &Stroke<'_>) -> tiny_skia::Stroke {
    tiny_skia::Stroke {
        width: stroke.width,
        line_cap: match stroke.line_cap {
            stroke::LineCap::Butt => tiny_skia::LineCap::Butt,
            stroke::LineCap::Square => tiny_skia::LineCap::Square,
            stroke::LineCap::Round => tiny_skia::LineCap::Round,
        },
        line_join: match stroke.line_join {
            stroke::LineJoin::Miter => tiny_skia::LineJoin::Miter,
            stroke::LineJoin::Round => tiny_skia::LineJoin::Round,
            stroke::LineJoin::Bevel => tiny_skia::LineJoin::Bevel,
        },
        dash: if stroke.line_dash.segments.is_empty() {
            None
        } else {
            tiny_skia::StrokeDash::new(
                stroke.line_dash.segments.into(),
                stroke.line_dash.offset as f32,
            )
        },
        ..Default::default()
    }
}
