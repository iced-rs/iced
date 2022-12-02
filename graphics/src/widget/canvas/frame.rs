use crate::gradient::Gradient;
use crate::triangle;
use crate::widget::canvas::{path, Fill, Geometry, Path, Stroke, Style, Text};
use crate::Primitive;

use iced_native::{Point, Rectangle, Size, Vector};

use lyon::geom::euclid;
use lyon::tessellation;
use std::borrow::Cow;

/// The frame of a [`Canvas`].
///
/// [`Canvas`]: crate::widget::Canvas
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
    Solid(tessellation::VertexBuffers<triangle::ColoredVertex2D, u32>),
    Gradient(
        tessellation::VertexBuffers<triangle::Vertex2D, u32>,
        Gradient,
    ),
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
            Style::Gradient(gradient) => match self.stack.last() {
                Some(Buffer::Gradient(_, last)) if gradient == last => {}
                _ => {
                    self.stack.push(Buffer::Gradient(
                        tessellation::VertexBuffers::new(),
                        gradient.clone(),
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
                    TriangleVertex2DBuilder(color.into_linear()),
                ))
            }
            (Style::Gradient(_), Buffer::Gradient(buffer, _)) => Box::new(
                tessellation::BuffersBuilder::new(buffer, Vertex2DBuilder),
            ),
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
                    TriangleVertex2DBuilder(color.into_linear()),
                ))
            }
            (Style::Gradient(_), Buffer::Gradient(buffer, _)) => Box::new(
                tessellation::BuffersBuilder::new(buffer, Vertex2DBuilder),
            ),
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
struct Transform {
    raw: lyon::math::Transform,
    is_identity: bool,
}

impl Transform {
    /// Transforms the given [Point] by the transformation matrix.
    fn transform_point(&self, point: &mut Point) {
        let transformed = self
            .raw
            .transform_point(euclid::Point2D::new(point.x, point.y));
        point.x = transformed.x;
        point.y = transformed.y;
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
        let (start, end) = match &mut gradient {
            Gradient::Linear(linear) => (&mut linear.start, &mut linear.end),
        };
        self.transform_point(start);
        self.transform_point(end);
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
                current: Transform {
                    raw: lyon::math::Transform::identity(),
                    is_identity: true,
                },
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

        let options =
            tessellation::FillOptions::default().with_fill_rule(rule.into());

        if self.transforms.current.is_identity {
            self.fill_tessellator.tessellate_path(
                path.raw(),
                &options,
                buffer.as_mut(),
            )
        } else {
            let path = path.transformed(&self.transforms.current.raw);

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

        let top_left =
            self.transforms.current.raw.transform_point(
                lyon::math::Point::new(top_left.x, top_left.y),
            );

        let size =
            self.transforms.current.raw.transform_vector(
                lyon::math::Vector::new(size.width, size.height),
            );

        let options =
            tessellation::FillOptions::default().with_fill_rule(rule.into());

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
        options.start_cap = stroke.line_cap.into();
        options.end_cap = stroke.line_cap.into();
        options.line_join = stroke.line_join.into();

        let path = if stroke.line_dash.segments.is_empty() {
            Cow::Borrowed(path)
        } else {
            Cow::Owned(path::dashed(path, stroke.line_dash))
        };

        if self.transforms.current.is_identity {
            self.stroke_tessellator.tessellate_path(
                path.raw(),
                &options,
                buffer.as_mut(),
            )
        } else {
            let path = path.transformed(&self.transforms.current.raw);

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
    /// a [`Canvas`]. Therefore, it is currently only meant to be used for
    /// overlays, which is the most common use case.
    ///
    /// Support for vectorial text is planned, and should address all these
    /// limitations.
    ///
    /// [`Canvas`]: crate::widget::Canvas
    pub fn fill_text(&mut self, text: impl Into<Text>) {
        let text = text.into();

        let position = if self.transforms.current.is_identity {
            text.position
        } else {
            let transformed = self.transforms.current.raw.transform_point(
                lyon::math::Point::new(text.position.x, text.position.y),
            );

            Point::new(transformed.x, transformed.y)
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
            font: text.font,
            horizontal_alignment: text.horizontal_alignment,
            vertical_alignment: text.vertical_alignment,
        });
    }

    /// Stores the current transform of the [`Frame`] and executes the given
    /// drawing operations, restoring the transform afterwards.
    ///
    /// This method is useful to compose transforms and perform drawing
    /// operations in different coordinate systems.
    #[inline]
    pub fn with_save(&mut self, f: impl FnOnce(&mut Frame)) {
        self.transforms.previous.push(self.transforms.current);

        f(self);

        self.transforms.current = self.transforms.previous.pop().unwrap();
    }

    /// Executes the given drawing operations within a [`Rectangle`] region,
    /// clipping any geometry that overflows its bounds. Any transformations
    /// performed are local to the provided closure.
    ///
    /// This method is useful to perform drawing operations that need to be
    /// clipped.
    #[inline]
    pub fn with_clip(&mut self, region: Rectangle, f: impl FnOnce(&mut Frame)) {
        let mut frame = Frame::new(region.size());

        f(&mut frame);

        let primitives = frame.into_primitives();

        let (text, meshes) = primitives
            .into_iter()
            .partition(|primitive| matches!(primitive, Primitive::Text { .. }));

        let translation = Vector::new(region.x, region.y);

        self.primitives.push(Primitive::Group {
            primitives: vec![
                Primitive::Translate {
                    translation,
                    content: Box::new(Primitive::Group { primitives: meshes }),
                },
                Primitive::Translate {
                    translation,
                    content: Box::new(Primitive::Clip {
                        bounds: Rectangle::with_size(region.size()),
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
        self.transforms.current.raw = self
            .transforms
            .current
            .raw
            .pre_translate(lyon::math::Vector::new(
                translation.x,
                translation.y,
            ));
        self.transforms.current.is_identity = false;
    }

    /// Applies a rotation in radians to the current transform of the [`Frame`].
    #[inline]
    pub fn rotate(&mut self, angle: f32) {
        self.transforms.current.raw = self
            .transforms
            .current
            .raw
            .pre_rotate(lyon::math::Angle::radians(angle));
        self.transforms.current.is_identity = false;
    }

    /// Applies a scaling to the current transform of the [`Frame`].
    #[inline]
    pub fn scale(&mut self, scale: f32) {
        self.transforms.current.raw =
            self.transforms.current.raw.pre_scale(scale, scale);
        self.transforms.current.is_identity = false;
    }

    /// Produces the [`Geometry`] representing everything drawn on the [`Frame`].
    pub fn into_geometry(self) -> Geometry {
        Geometry::from_primitive(Primitive::Group {
            primitives: self.into_primitives(),
        })
    }

    fn into_primitives(mut self) -> Vec<Primitive> {
        for buffer in self.buffers.stack {
            match buffer {
                Buffer::Solid(buffer) => {
                    if !buffer.indices.is_empty() {
                        self.primitives.push(Primitive::SolidMesh {
                            buffers: triangle::Mesh2D {
                                vertices: buffer.vertices,
                                indices: buffer.indices,
                            },
                            size: self.size,
                        })
                    }
                }
                Buffer::Gradient(buffer, gradient) => {
                    if !buffer.indices.is_empty() {
                        self.primitives.push(Primitive::GradientMesh {
                            buffers: triangle::Mesh2D {
                                vertices: buffer.vertices,
                                indices: buffer.indices,
                            },
                            size: self.size,
                            gradient,
                        })
                    }
                }
            }
        }

        self.primitives
    }
}

struct Vertex2DBuilder;

impl tessellation::FillVertexConstructor<triangle::Vertex2D>
    for Vertex2DBuilder
{
    fn new_vertex(
        &mut self,
        vertex: tessellation::FillVertex<'_>,
    ) -> triangle::Vertex2D {
        let position = vertex.position();

        triangle::Vertex2D {
            position: [position.x, position.y],
        }
    }
}

impl tessellation::StrokeVertexConstructor<triangle::Vertex2D>
    for Vertex2DBuilder
{
    fn new_vertex(
        &mut self,
        vertex: tessellation::StrokeVertex<'_, '_>,
    ) -> triangle::Vertex2D {
        let position = vertex.position();

        triangle::Vertex2D {
            position: [position.x, position.y],
        }
    }
}

struct TriangleVertex2DBuilder([f32; 4]);

impl tessellation::FillVertexConstructor<triangle::ColoredVertex2D>
    for TriangleVertex2DBuilder
{
    fn new_vertex(
        &mut self,
        vertex: tessellation::FillVertex<'_>,
    ) -> triangle::ColoredVertex2D {
        let position = vertex.position();

        triangle::ColoredVertex2D {
            position: [position.x, position.y],
            color: self.0,
        }
    }
}

impl tessellation::StrokeVertexConstructor<triangle::ColoredVertex2D>
    for TriangleVertex2DBuilder
{
    fn new_vertex(
        &mut self,
        vertex: tessellation::StrokeVertex<'_, '_>,
    ) -> triangle::ColoredVertex2D {
        let position = vertex.position();

        triangle::ColoredVertex2D {
            position: [position.x, position.y],
            color: self.0,
        }
    }
}
