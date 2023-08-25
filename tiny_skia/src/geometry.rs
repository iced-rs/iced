use crate::core::{Point, Rectangle, Size, Vector};
use crate::graphics::geometry::fill::{self, Fill};
use crate::graphics::geometry::stroke::{self, Stroke};
use crate::graphics::geometry::{Path, Style, Text};
use crate::graphics::Gradient;
use crate::primitive::{self, Primitive};

pub struct Frame {
    size: Size,
    transform: tiny_skia::Transform,
    stack: Vec<tiny_skia::Transform>,
    primitives: Vec<Primitive>,
}

impl Frame {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            transform: tiny_skia::Transform::identity(),
            stack: Vec::new(),
            primitives: Vec::new(),
        }
    }

    pub fn width(&self) -> f32 {
        self.size.width
    }

    pub fn height(&self) -> f32 {
        self.size.height
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn center(&self) -> Point {
        Point::new(self.size.width / 2.0, self.size.height / 2.0)
    }

    pub fn fill(&mut self, path: &Path, fill: impl Into<Fill>) {
        let Some(path) = convert_path(path) else {
            return;
        };
        let fill = fill.into();

        self.primitives
            .push(Primitive::Custom(primitive::Custom::Fill {
                path,
                paint: into_paint(fill.style),
                rule: into_fill_rule(fill.rule),
                transform: self.transform,
            }));
    }

    pub fn fill_rectangle(
        &mut self,
        top_left: Point,
        size: Size,
        fill: impl Into<Fill>,
    ) {
        let Some(path) = convert_path(&Path::rectangle(top_left, size)) else {
            return;
        };
        let fill = fill.into();

        self.primitives
            .push(Primitive::Custom(primitive::Custom::Fill {
                path,
                paint: tiny_skia::Paint {
                    anti_alias: false,
                    ..into_paint(fill.style)
                },
                rule: into_fill_rule(fill.rule),
                transform: self.transform,
            }));
    }

    pub fn stroke<'a>(&mut self, path: &Path, stroke: impl Into<Stroke<'a>>) {
        let Some(path) = convert_path(path) else {
            return;
        };

        let stroke = stroke.into();
        let skia_stroke = into_stroke(&stroke);

        self.primitives
            .push(Primitive::Custom(primitive::Custom::Stroke {
                path,
                paint: into_paint(stroke.style),
                stroke: skia_stroke,
                transform: self.transform,
            }));
    }

    pub fn fill_text(&mut self, text: impl Into<Text>) {
        let text = text.into();

        let position = if self.transform.is_identity() {
            text.position
        } else {
            let mut transformed = [tiny_skia::Point {
                x: text.position.x,
                y: text.position.y,
            }];

            self.transform.map_points(&mut transformed);

            Point::new(transformed[0].x, transformed[0].y)
        };

        // TODO: Use vectorial text instead of primitive
        self.primitives.push(Primitive::Text {
            content: text.content,
            bounds: Rectangle {
                x: position.x,
                y: position.y,
                width: f32::INFINITY,
                height: f32::INFINITY,
            },
            color: text.color,
            size: text.size,
            line_height: text.line_height,
            font: text.font,
            horizontal_alignment: text.horizontal_alignment,
            vertical_alignment: text.vertical_alignment,
            shaping: text.shaping,
        });
    }

    pub fn push_transform(&mut self) {
        self.stack.push(self.transform);
    }

    pub fn pop_transform(&mut self) {
        self.transform = self.stack.pop().expect("Pop transform");
    }

    pub fn clip(&mut self, frame: Self, at: Point) {
        self.primitives.push(Primitive::Translate {
            translation: Vector::new(at.x, at.y),
            content: Box::new(frame.into_primitive()),
        });
    }

    pub fn translate(&mut self, translation: Vector) {
        self.transform =
            self.transform.pre_translate(translation.x, translation.y);
    }

    pub fn rotate(&mut self, angle: f32) {
        self.transform = self
            .transform
            .pre_concat(tiny_skia::Transform::from_rotate(angle.to_degrees()));
    }

    pub fn scale(&mut self, scale: f32) {
        self.transform = self.transform.pre_scale(scale, scale);
    }

    pub fn into_primitive(self) -> Primitive {
        Primitive::Clip {
            bounds: Rectangle::new(Point::ORIGIN, self.size),
            content: Box::new(Primitive::Group {
                primitives: self.primitives,
            }),
        }
    }
}

fn convert_path(path: &Path) -> Option<tiny_skia::Path> {
    use iced_graphics::geometry::path::lyon_path;

    let mut builder = tiny_skia::PathBuilder::new();
    let mut last_point = Default::default();

    for event in path.raw().iter() {
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

pub fn into_stroke(stroke: &Stroke) -> tiny_skia::Stroke {
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
