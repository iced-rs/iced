use iced_native::{Point, Rectangle, Size, Vector};

use crate::{
    canvas::{Fill, Geometry, Path, Stroke, Text},
    triangle, Primitive,
};

use lyon::tessellation;

/// The frame of a [`Canvas`].
///
/// [`Canvas`]: crate::widget::Canvas
#[allow(missing_debug_implementations)]
pub struct Frame {
    size: Size,
    buffers: lyon::tessellation::VertexBuffers<triangle::Vertex2D, u32>,
    primitives: Vec<Primitive>,
    transforms: Transforms,
    fill_tessellator: tessellation::FillTessellator,
    stroke_tessellator: tessellation::StrokeTessellator,
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

impl Frame {
    /// Creates a new empty [`Frame`] with the given dimensions.
    ///
    /// The default coordinate system of a [`Frame`] has its origin at the
    /// top-left corner of its bounds.
    pub fn new(size: Size) -> Frame {
        Frame {
            size,
            buffers: lyon::tessellation::VertexBuffers::new(),
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
        let Fill { color, rule } = fill.into();

        let mut buffers = tessellation::BuffersBuilder::new(
            &mut self.buffers,
            FillVertex(color.into_linear()),
        );

        let options =
            tessellation::FillOptions::default().with_fill_rule(rule.into());

        let result = if self.transforms.current.is_identity {
            self.fill_tessellator.tessellate_path(
                path.raw(),
                &options,
                &mut buffers,
            )
        } else {
            let path = path.transformed(&self.transforms.current.raw);

            self.fill_tessellator.tessellate_path(
                path.raw(),
                &options,
                &mut buffers,
            )
        };

        let _ = result.expect("Tessellate path");
    }

    /// Draws an axis-aligned rectangle given its top-left corner coordinate and
    /// its `Size` on the [`Frame`] by filling it with the provided style.
    pub fn fill_rectangle(
        &mut self,
        top_left: Point,
        size: Size,
        fill: impl Into<Fill>,
    ) {
        let Fill { color, rule } = fill.into();

        let mut buffers = tessellation::BuffersBuilder::new(
            &mut self.buffers,
            FillVertex(color.into_linear()),
        );

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

        let _ = self
            .fill_tessellator
            .tessellate_rectangle(
                &lyon::math::Rect::new(top_left, size.into()),
                &options,
                &mut buffers,
            )
            .expect("Fill rectangle");
    }

    /// Draws the stroke of the given [`Path`] on the [`Frame`] with the
    /// provided style.
    pub fn stroke(&mut self, path: &Path, stroke: impl Into<Stroke>) {
        let stroke = stroke.into();

        let mut buffers = tessellation::BuffersBuilder::new(
            &mut self.buffers,
            StrokeVertex(stroke.color.into_linear()),
        );

        let mut options = tessellation::StrokeOptions::default();
        options.line_width = stroke.width;
        options.start_cap = stroke.line_cap.into();
        options.end_cap = stroke.line_cap.into();
        options.line_join = stroke.line_join.into();

        let result = if self.transforms.current.is_identity {
            self.stroke_tessellator.tessellate_path(
                path.raw(),
                &options,
                &mut buffers,
            )
        } else {
            let path = path.transformed(&self.transforms.current.raw);

            self.stroke_tessellator.tessellate_path(
                path.raw(),
                &options,
                &mut buffers,
            )
        };

        let _ = result.expect("Stroke path");
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
        use std::f32;

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

    /// Applies a rotation to the current transform of the [`Frame`].
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
    pub fn into_geometry(mut self) -> Geometry {
        if !self.buffers.indices.is_empty() {
            self.primitives.push(Primitive::Mesh2D {
                buffers: triangle::Mesh2D {
                    vertices: self.buffers.vertices,
                    indices: self.buffers.indices,
                },
                size: self.size,
            });
        }

        Geometry::from_primitive(Primitive::Group {
            primitives: self.primitives,
        })
    }
}

struct FillVertex([f32; 4]);

impl lyon::tessellation::FillVertexConstructor<triangle::Vertex2D>
    for FillVertex
{
    fn new_vertex(
        &mut self,
        vertex: lyon::tessellation::FillVertex<'_>,
    ) -> triangle::Vertex2D {
        let position = vertex.position();

        triangle::Vertex2D {
            position: [position.x, position.y],
            color: self.0,
        }
    }
}

struct StrokeVertex([f32; 4]);

impl lyon::tessellation::StrokeVertexConstructor<triangle::Vertex2D>
    for StrokeVertex
{
    fn new_vertex(
        &mut self,
        vertex: lyon::tessellation::StrokeVertex<'_, '_>,
    ) -> triangle::Vertex2D {
        let position = vertex.position();

        triangle::Vertex2D {
            position: [position.x, position.y],
            color: self.0,
        }
    }
}
