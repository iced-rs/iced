//! Build and draw geometry.
use crate::core::text::LineHeight;
use crate::core::{Pixels, Point, Rectangle, Size, Transformation, Vector};
use crate::graphics::color;
use crate::graphics::geometry::fill::{self, Fill};
use crate::graphics::geometry::{
    LineCap, LineDash, LineJoin, Path, Stroke, Style, Text,
};
use crate::graphics::gradient::{self, Gradient};
use crate::graphics::mesh::{self, Mesh};
use crate::primitive::{self, Primitive};

use lyon::geom::euclid;
use lyon::tessellation;
use std::borrow::Cow;

/// A frame for drawing some geometry.
#[allow(missing_debug_implementations)]
pub struct Frame {
    size: Size,
    buffers: BufferStack,
    primitives: Vec<Primitive>,
    transforms: Transforms,
    fill_tessellator: tessellation::FillTessellator,
    stroke_tessellator: tessellation::StrokeTessellator,
}

enum Buffer {
    Solid(tessellation::VertexBuffers<mesh::SolidVertex2D, u32>),
    Gradient(tessellation::VertexBuffers<mesh::GradientVertex2D, u32>),
}

struct BufferStack {
    stack: Vec<Buffer>,
}

impl BufferStack {
    fn new() -> Self {
        Self { stack: Vec::new() }
    }

    fn get_mut(&mut self, style: &Style) -> &mut Buffer {
        match style {
            Style::Solid(_) => match self.stack.last() {
                Some(Buffer::Solid(_)) => {}
                _ => {
                    self.stack.push(Buffer::Solid(
                        tessellation::VertexBuffers::new(),
                    ));
                }
            },
            Style::Gradient(_) => match self.stack.last() {
                Some(Buffer::Gradient(_)) => {}
                _ => {
                    self.stack.push(Buffer::Gradient(
                        tessellation::VertexBuffers::new(),
                    ));
                }
            },
        }

        self.stack.last_mut().unwrap()
    }

    fn get_fill<'a>(
        &'a mut self,
        style: &Style,
    ) -> Box<dyn tessellation::FillGeometryBuilder + 'a> {
        match (style, self.get_mut(style)) {
            (Style::Solid(color), Buffer::Solid(buffer)) => {
                Box::new(tessellation::BuffersBuilder::new(
                    buffer,
                    TriangleVertex2DBuilder(color::pack(*color)),
                ))
            }
            (Style::Gradient(gradient), Buffer::Gradient(buffer)) => {
                Box::new(tessellation::BuffersBuilder::new(
                    buffer,
                    GradientVertex2DBuilder {
                        gradient: gradient.pack(),
                    },
                ))
            }
            _ => unreachable!(),
        }
    }

    fn get_stroke<'a>(
        &'a mut self,
        style: &Style,
    ) -> Box<dyn tessellation::StrokeGeometryBuilder + 'a> {
        match (style, self.get_mut(style)) {
            (Style::Solid(color), Buffer::Solid(buffer)) => {
                Box::new(tessellation::BuffersBuilder::new(
                    buffer,
                    TriangleVertex2DBuilder(color::pack(*color)),
                ))
            }
            (Style::Gradient(gradient), Buffer::Gradient(buffer)) => {
                Box::new(tessellation::BuffersBuilder::new(
                    buffer,
                    GradientVertex2DBuilder {
                        gradient: gradient.pack(),
                    },
                ))
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
struct Transforms {
    previous: Vec<Transform>,
    current: Transform,
}

#[derive(Debug, Clone, Copy)]
struct Transform(lyon::math::Transform);

impl Transform {
    fn is_identity(&self) -> bool {
        self.0 == lyon::math::Transform::identity()
    }

    fn is_scale_translation(&self) -> bool {
        self.0.m12.abs() < 2.0 * f32::EPSILON
            && self.0.m21.abs() < 2.0 * f32::EPSILON
    }

    fn scale(&self) -> (f32, f32) {
        (self.0.m11, self.0.m22)
    }

    fn transform_point(&self, point: Point) -> Point {
        let transformed = self
            .0
            .transform_point(euclid::Point2D::new(point.x, point.y));

        Point {
            x: transformed.x,
            y: transformed.y,
        }
    }

    fn transform_style(&self, style: Style) -> Style {
        match style {
            Style::Solid(color) => Style::Solid(color),
            Style::Gradient(gradient) => {
                Style::Gradient(self.transform_gradient(gradient))
            }
        }
    }

    fn transform_gradient(&self, mut gradient: Gradient) -> Gradient {
        match &mut gradient {
            Gradient::Linear(linear) => {
                linear.start = self.transform_point(linear.start);
                linear.end = self.transform_point(linear.end);
            }
        }

        gradient
    }
}

impl Frame {
    /// Creates a new empty [`Frame`] with the given dimensions.
    ///
    /// The default coordinate system of a [`Frame`] has its origin at the
    /// top-left corner of its bounds.
    pub fn new(size: Size) -> Frame {
        Frame {
            size,
            buffers: BufferStack::new(),
            primitives: Vec::new(),
            transforms: Transforms {
                previous: Vec::new(),
                current: Transform(lyon::math::Transform::identity()),
            },
            fill_tessellator: tessellation::FillTessellator::new(),
            stroke_tessellator: tessellation::StrokeTessellator::new(),
        }
    }

    /// Returns the width of the [`Frame`].
    #[inline]
    pub fn width(&self) -> f32 {
        self.size.width
    }

    /// Returns the height of the [`Frame`].
    #[inline]
    pub fn height(&self) -> f32 {
        self.size.height
    }

    /// Returns the dimensions of the [`Frame`].
    #[inline]
    pub fn size(&self) -> Size {
        self.size
    }

    /// Returns the coordinate of the center of the [`Frame`].
    #[inline]
    pub fn center(&self) -> Point {
        Point::new(self.size.width / 2.0, self.size.height / 2.0)
    }

    /// Draws the given [`Path`] on the [`Frame`] by filling it with the
    /// provided style.
    pub fn fill(&mut self, path: &Path, fill: impl Into<Fill>) {
        let Fill { style, rule } = fill.into();

        let mut buffer = self
            .buffers
            .get_fill(&self.transforms.current.transform_style(style));

        let options = tessellation::FillOptions::default()
            .with_fill_rule(into_fill_rule(rule));

        if self.transforms.current.is_identity() {
            self.fill_tessellator.tessellate_path(
                path.raw(),
                &options,
                buffer.as_mut(),
            )
        } else {
            let path = path.transform(&self.transforms.current.0);

            self.fill_tessellator.tessellate_path(
                path.raw(),
                &options,
                buffer.as_mut(),
            )
        }
        .expect("Tessellate path.");
    }

    /// Draws an axis-aligned rectangle given its top-left corner coordinate and
    /// its `Size` on the [`Frame`] by filling it with the provided style.
    pub fn fill_rectangle(
        &mut self,
        top_left: Point,
        size: Size,
        fill: impl Into<Fill>,
    ) {
        let Fill { style, rule } = fill.into();

        let mut buffer = self
            .buffers
            .get_fill(&self.transforms.current.transform_style(style));

        let top_left = self
            .transforms
            .current
            .0
            .transform_point(lyon::math::Point::new(top_left.x, top_left.y));

        let size =
            self.transforms.current.0.transform_vector(
                lyon::math::Vector::new(size.width, size.height),
            );

        let options = tessellation::FillOptions::default()
            .with_fill_rule(into_fill_rule(rule));

        self.fill_tessellator
            .tessellate_rectangle(
                &lyon::math::Box2D::new(top_left, top_left + size),
                &options,
                buffer.as_mut(),
            )
            .expect("Fill rectangle");
    }

    /// Draws the stroke of the given [`Path`] on the [`Frame`] with the
    /// provided style.
    pub fn stroke<'a>(&mut self, path: &Path, stroke: impl Into<Stroke<'a>>) {
        let stroke = stroke.into();

        let mut buffer = self
            .buffers
            .get_stroke(&self.transforms.current.transform_style(stroke.style));

        let mut options = tessellation::StrokeOptions::default();
        options.line_width = stroke.width;
        options.start_cap = into_line_cap(stroke.line_cap);
        options.end_cap = into_line_cap(stroke.line_cap);
        options.line_join = into_line_join(stroke.line_join);

        let path = if stroke.line_dash.segments.is_empty() {
            Cow::Borrowed(path)
        } else {
            Cow::Owned(dashed(path, stroke.line_dash))
        };

        if self.transforms.current.is_identity() {
            self.stroke_tessellator.tessellate_path(
                path.raw(),
                &options,
                buffer.as_mut(),
            )
        } else {
            let path = path.transform(&self.transforms.current.0);

            self.stroke_tessellator.tessellate_path(
                path.raw(),
                &options,
                buffer.as_mut(),
            )
        }
        .expect("Stroke path");
    }

    /// Draws the characters of the given [`Text`] on the [`Frame`], filling
    /// them with the given color.
    ///
    /// __Warning:__ Text currently does not work well with rotations and scale
    /// transforms! The position will be correctly transformed, but the
    /// resulting glyphs will not be rotated or scaled properly.
    ///
    /// Additionally, all text will be rendered on top of all the layers of
    /// a `Canvas`. Therefore, it is currently only meant to be used for
    /// overlays, which is the most common use case.
    ///
    /// Support for vectorial text is planned, and should address all these
    /// limitations.
    pub fn fill_text(&mut self, text: impl Into<Text>) {
        let text = text.into();

        let (scale_x, scale_y) = self.transforms.current.scale();

        if self.transforms.current.is_scale_translation()
            && scale_x == scale_y
            && scale_x > 0.0
            && scale_y > 0.0
        {
            let (position, size, line_height) =
                if self.transforms.current.is_identity() {
                    (text.position, text.size, text.line_height)
                } else {
                    let position =
                        self.transforms.current.transform_point(text.position);

                    let size = Pixels(text.size.0 * scale_y);

                    let line_height = match text.line_height {
                        LineHeight::Absolute(size) => {
                            LineHeight::Absolute(Pixels(size.0 * scale_y))
                        }
                        LineHeight::Relative(factor) => {
                            LineHeight::Relative(factor)
                        }
                    };

                    (position, size, line_height)
                };

            let bounds = Rectangle {
                x: position.x,
                y: position.y,
                width: f32::INFINITY,
                height: f32::INFINITY,
            };

            // TODO: Honor layering!
            self.primitives.push(Primitive::Text {
                content: text.content,
                bounds,
                color: text.color,
                size,
                line_height,
                font: text.font,
                horizontal_alignment: text.horizontal_alignment,
                vertical_alignment: text.vertical_alignment,
                shaping: text.shaping,
                clip_bounds: Rectangle::with_size(Size::INFINITY),
            });
        } else {
            text.draw_with(|path, color| self.fill(&path, color));
        }
    }

    /// Stores the current transform of the [`Frame`] and executes the given
    /// drawing operations, restoring the transform afterwards.
    ///
    /// This method is useful to compose transforms and perform drawing
    /// operations in different coordinate systems.
    #[inline]
    pub fn with_save<R>(&mut self, f: impl FnOnce(&mut Frame) -> R) -> R {
        self.push_transform();

        let result = f(self);

        self.pop_transform();

        result
    }

    /// Pushes the current transform in the transform stack.
    pub fn push_transform(&mut self) {
        self.transforms.previous.push(self.transforms.current);
    }

    /// Pops a transform from the transform stack and sets it as the current transform.
    pub fn pop_transform(&mut self) {
        self.transforms.current = self.transforms.previous.pop().unwrap();
    }

    /// Executes the given drawing operations within a [`Rectangle`] region,
    /// clipping any geometry that overflows its bounds. Any transformations
    /// performed are local to the provided closure.
    ///
    /// This method is useful to perform drawing operations that need to be
    /// clipped.
    #[inline]
    pub fn with_clip<R>(
        &mut self,
        region: Rectangle,
        f: impl FnOnce(&mut Frame) -> R,
    ) -> R {
        let mut frame = Frame::new(region.size());

        let result = f(&mut frame);

        let origin = Point::new(region.x, region.y);

        self.clip(frame, origin);

        result
    }

    /// Draws the clipped contents of the given [`Frame`] with origin at the given [`Point`].
    pub fn clip(&mut self, frame: Frame, at: Point) {
        let size = frame.size();
        let primitives = frame.into_primitives();
        let transformation = Transformation::translate(at.x, at.y);

        let (text, meshes) = primitives
            .into_iter()
            .partition(|primitive| matches!(primitive, Primitive::Text { .. }));

        self.primitives.push(Primitive::Group {
            primitives: vec![
                Primitive::Transform {
                    transformation,
                    content: Box::new(Primitive::Group { primitives: meshes }),
                },
                Primitive::Transform {
                    transformation,
                    content: Box::new(Primitive::Clip {
                        bounds: Rectangle::with_size(size),
                        content: Box::new(Primitive::Group {
                            primitives: text,
                        }),
                    }),
                },
            ],
        });
    }

    /// Applies a translation to the current transform of the [`Frame`].
    #[inline]
    pub fn translate(&mut self, translation: Vector) {
        self.transforms.current.0 =
            self.transforms
                .current
                .0
                .pre_translate(lyon::math::Vector::new(
                    translation.x,
                    translation.y,
                ));
    }

    /// Applies a rotation in radians to the current transform of the [`Frame`].
    #[inline]
    pub fn rotate(&mut self, angle: f32) {
        self.transforms.current.0 = self
            .transforms
            .current
            .0
            .pre_rotate(lyon::math::Angle::radians(angle));
    }

    /// Applies a uniform scaling to the current transform of the [`Frame`].
    #[inline]
    pub fn scale(&mut self, scale: impl Into<f32>) {
        let scale = scale.into();

        self.scale_nonuniform(Vector { x: scale, y: scale });
    }

    /// Applies a non-uniform scaling to the current transform of the [`Frame`].
    #[inline]
    pub fn scale_nonuniform(&mut self, scale: impl Into<Vector>) {
        let scale = scale.into();

        self.transforms.current.0 =
            self.transforms.current.0.pre_scale(scale.x, scale.y);
    }

    /// Produces the [`Primitive`] representing everything drawn on the [`Frame`].
    pub fn into_primitive(self) -> Primitive {
        Primitive::Group {
            primitives: self.into_primitives(),
        }
    }

    fn into_primitives(mut self) -> Vec<Primitive> {
        for buffer in self.buffers.stack {
            match buffer {
                Buffer::Solid(buffer) => {
                    if !buffer.indices.is_empty() {
                        self.primitives.push(Primitive::Custom(
                            primitive::Custom::Mesh(Mesh::Solid {
                                buffers: mesh::Indexed {
                                    vertices: buffer.vertices,
                                    indices: buffer.indices,
                                },
                                size: self.size,
                            }),
                        ));
                    }
                }
                Buffer::Gradient(buffer) => {
                    if !buffer.indices.is_empty() {
                        self.primitives.push(Primitive::Custom(
                            primitive::Custom::Mesh(Mesh::Gradient {
                                buffers: mesh::Indexed {
                                    vertices: buffer.vertices,
                                    indices: buffer.indices,
                                },
                                size: self.size,
                            }),
                        ));
                    }
                }
            }
        }

        self.primitives
    }
}

struct GradientVertex2DBuilder {
    gradient: gradient::Packed,
}

impl tessellation::FillVertexConstructor<mesh::GradientVertex2D>
    for GradientVertex2DBuilder
{
    fn new_vertex(
        &mut self,
        vertex: tessellation::FillVertex<'_>,
    ) -> mesh::GradientVertex2D {
        let position = vertex.position();

        mesh::GradientVertex2D {
            position: [position.x, position.y],
            gradient: self.gradient,
        }
    }
}

impl tessellation::StrokeVertexConstructor<mesh::GradientVertex2D>
    for GradientVertex2DBuilder
{
    fn new_vertex(
        &mut self,
        vertex: tessellation::StrokeVertex<'_, '_>,
    ) -> mesh::GradientVertex2D {
        let position = vertex.position();

        mesh::GradientVertex2D {
            position: [position.x, position.y],
            gradient: self.gradient,
        }
    }
}

struct TriangleVertex2DBuilder(color::Packed);

impl tessellation::FillVertexConstructor<mesh::SolidVertex2D>
    for TriangleVertex2DBuilder
{
    fn new_vertex(
        &mut self,
        vertex: tessellation::FillVertex<'_>,
    ) -> mesh::SolidVertex2D {
        let position = vertex.position();

        mesh::SolidVertex2D {
            position: [position.x, position.y],
            color: self.0,
        }
    }
}

impl tessellation::StrokeVertexConstructor<mesh::SolidVertex2D>
    for TriangleVertex2DBuilder
{
    fn new_vertex(
        &mut self,
        vertex: tessellation::StrokeVertex<'_, '_>,
    ) -> mesh::SolidVertex2D {
        let position = vertex.position();

        mesh::SolidVertex2D {
            position: [position.x, position.y],
            color: self.0,
        }
    }
}

fn into_line_join(line_join: LineJoin) -> lyon::tessellation::LineJoin {
    match line_join {
        LineJoin::Miter => lyon::tessellation::LineJoin::Miter,
        LineJoin::Round => lyon::tessellation::LineJoin::Round,
        LineJoin::Bevel => lyon::tessellation::LineJoin::Bevel,
    }
}

fn into_line_cap(line_cap: LineCap) -> lyon::tessellation::LineCap {
    match line_cap {
        LineCap::Butt => lyon::tessellation::LineCap::Butt,
        LineCap::Square => lyon::tessellation::LineCap::Square,
        LineCap::Round => lyon::tessellation::LineCap::Round,
    }
}

fn into_fill_rule(rule: fill::Rule) -> lyon::tessellation::FillRule {
    match rule {
        fill::Rule::NonZero => lyon::tessellation::FillRule::NonZero,
        fill::Rule::EvenOdd => lyon::tessellation::FillRule::EvenOdd,
    }
}

pub(super) fn dashed(path: &Path, line_dash: LineDash<'_>) -> Path {
    use lyon::algorithms::walk::{
        walk_along_path, RepeatedPattern, WalkerEvent,
    };
    use lyon::path::iterator::PathIterator;

    Path::new(|builder| {
        let segments_odd = (line_dash.segments.len() % 2 == 1)
            .then(|| [line_dash.segments, line_dash.segments].concat());

        let mut draw_line = false;

        walk_along_path(
            path.raw().iter().flattened(0.01),
            0.0,
            lyon::tessellation::StrokeOptions::DEFAULT_TOLERANCE,
            &mut RepeatedPattern {
                callback: |event: WalkerEvent<'_>| {
                    let point = Point {
                        x: event.position.x,
                        y: event.position.y,
                    };

                    if draw_line {
                        builder.line_to(point);
                    } else {
                        builder.move_to(point);
                    }

                    draw_line = !draw_line;

                    true
                },
                index: line_dash.offset,
                intervals: segments_odd
                    .as_deref()
                    .unwrap_or(line_dash.segments),
            },
        );
    })
}
