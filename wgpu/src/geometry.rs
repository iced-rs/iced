//! Build and draw geometry.
use crate::core::text::LineHeight;
use crate::core::{
    self, Pixels, Point, Radians, Rectangle, Size, Svg, Transformation, Vector,
};
use crate::graphics::cache::{self, Cached};
use crate::graphics::color;
use crate::graphics::geometry::fill::{self, Fill};
use crate::graphics::geometry::{
    self, LineCap, LineDash, LineJoin, Path, Stroke, Style,
};
use crate::graphics::gradient::{self, Gradient};
use crate::graphics::mesh::{self, Mesh};
use crate::graphics::{Image, Text};
use crate::text;
use crate::triangle;

use lyon::geom::euclid;
use lyon::tessellation;

use std::borrow::Cow;
use std::sync::Arc;

#[derive(Debug)]
pub enum Geometry {
    Live {
        meshes: Vec<Mesh>,
        images: Vec<Image>,
        text: Vec<Text>,
    },
    Cached(Cache),
}

#[derive(Debug, Clone, Default)]
pub struct Cache {
    pub meshes: Option<triangle::Cache>,
    pub images: Option<Arc<[Image]>>,
    pub text: Option<text::Cache>,
}

impl Cached for Geometry {
    type Cache = Cache;

    fn load(cache: &Self::Cache) -> Self {
        Geometry::Cached(cache.clone())
    }

    fn cache(
        self,
        group: cache::Group,
        previous: Option<Self::Cache>,
    ) -> Self::Cache {
        match self {
            Self::Live {
                meshes,
                images,
                text,
            } => {
                let images = if images.is_empty() {
                    None
                } else {
                    Some(Arc::from(images))
                };

                if let Some(mut previous) = previous {
                    if let Some(cache) = &mut previous.meshes {
                        cache.update(meshes);
                    } else {
                        previous.meshes = triangle::Cache::new(meshes);
                    }

                    if let Some(cache) = &mut previous.text {
                        cache.update(text);
                    } else {
                        previous.text = text::Cache::new(group, text);
                    }

                    previous.images = images;

                    previous
                } else {
                    Cache {
                        meshes: triangle::Cache::new(meshes),
                        images,
                        text: text::Cache::new(group, text),
                    }
                }
            }
            Self::Cached(cache) => cache,
        }
    }
}

/// A frame for drawing some geometry.
#[allow(missing_debug_implementations)]
pub struct Frame {
    clip_bounds: Rectangle,
    buffers: BufferStack,
    meshes: Vec<Mesh>,
    images: Vec<Image>,
    text: Vec<Text>,
    transforms: Transforms,
    fill_tessellator: tessellation::FillTessellator,
    stroke_tessellator: tessellation::StrokeTessellator,
}

impl Frame {
    /// Creates a new [`Frame`] with the given [`Size`].
    pub fn new(size: Size) -> Frame {
        Self::with_clip(Rectangle::with_size(size))
    }

    /// Creates a new [`Frame`] with the given clip bounds.
    pub fn with_clip(bounds: Rectangle) -> Frame {
        Frame {
            clip_bounds: bounds,
            buffers: BufferStack::new(),
            meshes: Vec::new(),
            images: Vec::new(),
            text: Vec::new(),
            transforms: Transforms {
                previous: Vec::new(),
                current: Transform(lyon::math::Transform::translation(
                    bounds.x, bounds.y,
                )),
            },
            fill_tessellator: tessellation::FillTessellator::new(),
            stroke_tessellator: tessellation::StrokeTessellator::new(),
        }
    }
}

impl geometry::frame::Backend for Frame {
    type Geometry = Geometry;

    #[inline]
    fn width(&self) -> f32 {
        self.clip_bounds.width
    }

    #[inline]
    fn height(&self) -> f32 {
        self.clip_bounds.height
    }

    #[inline]
    fn size(&self) -> Size {
        self.clip_bounds.size()
    }

    #[inline]
    fn center(&self) -> Point {
        Point::new(self.clip_bounds.width / 2.0, self.clip_bounds.height / 2.0)
    }

    fn fill(&mut self, path: &Path, fill: impl Into<Fill>) {
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

    fn fill_rectangle(
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

    fn stroke<'a>(&mut self, path: &Path, stroke: impl Into<Stroke<'a>>) {
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

    fn stroke_rectangle<'a>(
        &mut self,
        top_left: Point,
        size: Size,
        stroke: impl Into<Stroke<'a>>,
    ) {
        let stroke = stroke.into();

        let mut buffer = self
            .buffers
            .get_stroke(&self.transforms.current.transform_style(stroke.style));

        let top_left = self
            .transforms
            .current
            .0
            .transform_point(lyon::math::Point::new(top_left.x, top_left.y));

        let size =
            self.transforms.current.0.transform_vector(
                lyon::math::Vector::new(size.width, size.height),
            );

        let mut options = tessellation::StrokeOptions::default();
        options.line_width = stroke.width;
        options.start_cap = into_line_cap(stroke.line_cap);
        options.end_cap = into_line_cap(stroke.line_cap);
        options.line_join = into_line_join(stroke.line_join);

        self.stroke_tessellator
            .tessellate_rectangle(
                &lyon::math::Box2D::new(top_left, top_left + size),
                &options,
                buffer.as_mut(),
            )
            .expect("Stroke rectangle");
    }

    fn stroke_text<'a>(
        &mut self,
        text: impl Into<geometry::Text>,
        stroke: impl Into<Stroke<'a>>,
    ) {
        let text = text.into();
        let stroke = stroke.into();

        text.draw_with(|glyph, _color| self.stroke(&glyph, stroke));
    }

    fn fill_text(&mut self, text: impl Into<geometry::Text>) {
        let text = text.into();

        let (scale_x, scale_y) = self.transforms.current.scale();

        if self.transforms.current.is_scale_translation()
            && scale_x == scale_y
            && scale_x > 0.0
            && scale_y > 0.0
        {
            let (bounds, size, line_height) =
                if self.transforms.current.is_identity() {
                    (
                        Rectangle::new(
                            text.position,
                            Size::new(text.max_width, f32::INFINITY),
                        ),
                        text.size,
                        text.line_height,
                    )
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

                    (
                        Rectangle::new(
                            position,
                            Size::new(text.max_width, f32::INFINITY),
                        ),
                        size,
                        line_height,
                    )
                };

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
                clip_bounds: self.clip_bounds,
            });
        } else {
            text.draw_with(|path, color| self.fill(&path, color));
        }
    }

    #[inline]
    fn translate(&mut self, translation: Vector) {
        self.transforms.current.0 =
            self.transforms
                .current
                .0
                .pre_translate(lyon::math::Vector::new(
                    translation.x,
                    translation.y,
                ));
    }

    #[inline]
    fn rotate(&mut self, angle: impl Into<Radians>) {
        self.transforms.current.0 = self
            .transforms
            .current
            .0
            .pre_rotate(lyon::math::Angle::radians(angle.into().0));
    }

    #[inline]
    fn scale(&mut self, scale: impl Into<f32>) {
        let scale = scale.into();

        self.scale_nonuniform(Vector { x: scale, y: scale });
    }

    #[inline]
    fn scale_nonuniform(&mut self, scale: impl Into<Vector>) {
        let scale = scale.into();

        self.transforms.current.0 =
            self.transforms.current.0.pre_scale(scale.x, scale.y);
    }

    fn push_transform(&mut self) {
        self.transforms.previous.push(self.transforms.current);
    }

    fn pop_transform(&mut self) {
        self.transforms.current = self.transforms.previous.pop().unwrap();
    }

    fn draft(&mut self, clip_bounds: Rectangle) -> Frame {
        Frame::with_clip(clip_bounds)
    }

    fn paste(&mut self, frame: Frame) {
        self.meshes.extend(frame.meshes);
        self.meshes
            .extend(frame.buffers.into_meshes(frame.clip_bounds));

        self.images.extend(frame.images);
        self.text.extend(frame.text);
    }

    fn into_geometry(mut self) -> Self::Geometry {
        self.meshes
            .extend(self.buffers.into_meshes(self.clip_bounds));

        Geometry::Live {
            meshes: self.meshes,
            images: self.images,
            text: self.text,
        }
    }

    fn draw_image(&mut self, bounds: Rectangle, image: impl Into<core::Image>) {
        let mut image = image.into();

        let (bounds, external_rotation) =
            self.transforms.current.transform_rectangle(bounds);

        image.rotation += external_rotation;

        self.images.push(Image::Raster(image, bounds));
    }

    fn draw_svg(&mut self, bounds: Rectangle, svg: impl Into<Svg>) {
        let mut svg = svg.into();

        let (bounds, external_rotation) =
            self.transforms.current.transform_rectangle(bounds);

        svg.rotation += external_rotation;

        self.images.push(Image::Vector(svg, bounds));
    }
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

    fn into_meshes(self, clip_bounds: Rectangle) -> impl Iterator<Item = Mesh> {
        self.stack
            .into_iter()
            .filter_map(move |buffer| match buffer {
                Buffer::Solid(buffer) if !buffer.indices.is_empty() => {
                    Some(Mesh::Solid {
                        buffers: mesh::Indexed {
                            vertices: buffer.vertices,
                            indices: buffer.indices,
                        },
                        clip_bounds,
                        transformation: Transformation::IDENTITY,
                    })
                }
                Buffer::Gradient(buffer) if !buffer.indices.is_empty() => {
                    Some(Mesh::Gradient {
                        buffers: mesh::Indexed {
                            vertices: buffer.vertices,
                            indices: buffer.indices,
                        },
                        clip_bounds,
                        transformation: Transformation::IDENTITY,
                    })
                }
                _ => None,
            })
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

    fn transform_rectangle(
        &self,
        rectangle: Rectangle,
    ) -> (Rectangle, Radians) {
        let top_left = self.transform_point(rectangle.position());
        let top_right = self.transform_point(
            rectangle.position() + Vector::new(rectangle.width, 0.0),
        );
        let bottom_left = self.transform_point(
            rectangle.position() + Vector::new(0.0, rectangle.height),
        );

        Rectangle::with_vertices(top_left, top_right, bottom_left)
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
        RepeatedPattern, WalkerEvent, walk_along_path,
    };
    use lyon::path::iterator::PathIterator;

    Path::new(|builder| {
        let segments_odd = (line_dash.segments.len() % 2 == 1)
            .then(|| [line_dash.segments, line_dash.segments].concat());

        let mut draw_line = false;

        walk_along_path(
            path.raw().iter().flattened(
                lyon::tessellation::StrokeOptions::DEFAULT_TOLERANCE,
            ),
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
