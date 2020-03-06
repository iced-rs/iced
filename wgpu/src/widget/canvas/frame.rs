use iced_native::{Point, Size, Vector};

use crate::{
    canvas::{Fill, Path, Stroke, TextNode},
    triangle, Primitive,
};

use std::sync::Arc;

/// The frame of a [`Canvas`].
///
/// [`Canvas`]: struct.Canvas.html
#[derive(Debug)]
pub struct Frame {
    width: f32,
    height: f32,
    buffers: lyon::tessellation::VertexBuffers<triangle::Vertex2D, u32>,
    transforms: Transforms,
    texts: Vec<TextNode>,
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
    ///
    /// [`Frame`]: struct.Frame.html
    pub fn new(width: f32, height: f32) -> Frame {
        Frame {
            width,
            height,
            buffers: lyon::tessellation::VertexBuffers::new(),
            texts: Vec::new(),
            transforms: Transforms {
                previous: Vec::new(),
                current: Transform {
                    raw: lyon::math::Transform::identity(),
                    is_identity: true,
                },
            },
        }
    }

    /// Returns the width of the [`Frame`].
    ///
    /// [`Frame`]: struct.Frame.html
    #[inline]
    pub fn width(&self) -> f32 {
        self.width
    }

    /// Returns the width of the [`Frame`].
    ///
    /// [`Frame`]: struct.Frame.html
    #[inline]
    pub fn height(&self) -> f32 {
        self.height
    }

    /// Returns the dimensions of the [`Frame`].
    ///
    /// [`Frame`]: struct.Frame.html
    #[inline]
    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    /// Returns the coordinate of the center of the [`Frame`].
    ///
    /// [`Frame`]: struct.Frame.html
    #[inline]
    pub fn center(&self) -> Point {
        Point::new(self.width / 2.0, self.height / 2.0)
    }

    /// Draws the given [`Path`] on the [`Frame`] by filling it with the
    /// provided style.
    ///
    /// [`Path`]: path/struct.Path.html
    /// [`Frame`]: struct.Frame.html
    pub fn fill(&mut self, path: &Path, fill: Fill) {
        use lyon::tessellation::{
            BuffersBuilder, FillOptions, FillTessellator,
        };

        let mut buffers = BuffersBuilder::new(
            &mut self.buffers,
            FillVertex(match fill {
                Fill::Color(color) => color.into_linear(),
            }),
        );

        let mut tessellator = FillTessellator::new();

        let result = if self.transforms.current.is_identity {
            tessellator.tessellate_path(
                path.raw(),
                &FillOptions::default(),
                &mut buffers,
            )
        } else {
            let path = path.transformed(&self.transforms.current.raw);

            tessellator.tessellate_path(
                path.raw(),
                &FillOptions::default(),
                &mut buffers,
            )
        };

        let _ = result.expect("Tessellate path");
    }

    /// Draws the stroke of the given [`Path`] on the [`Frame`] with the
    /// provided style.
    ///
    /// [`Path`]: path/struct.Path.html
    /// [`Frame`]: struct.Frame.html
    pub fn stroke(&mut self, path: &Path, stroke: Stroke) {
        use lyon::tessellation::{
            BuffersBuilder, StrokeOptions, StrokeTessellator,
        };

        let mut buffers = BuffersBuilder::new(
            &mut self.buffers,
            StrokeVertex(stroke.color.into_linear()),
        );

        let mut tessellator = StrokeTessellator::new();

        let mut options = StrokeOptions::default();
        options.line_width = stroke.width;
        options.start_cap = stroke.line_cap.into();
        options.end_cap = stroke.line_cap.into();
        options.line_join = stroke.line_join.into();

        let result = if self.transforms.current.is_identity {
            tessellator.tessellate_path(path.raw(), &options, &mut buffers)
        } else {
            let path = path.transformed(&self.transforms.current.raw);

            tessellator.tessellate_path(path.raw(), &options, &mut buffers)
        };

        let _ = result.expect("Stroke path");
    }

    /// Draws the text of the given [`TextNode`] on the [`Frame`]
    ///
    /// [`TextNode`]: struct.TextNode.html
    /// [`Frame`]: struct.Frame.html
    pub fn text(&mut self, text: TextNode) {
        self.texts.push(text);
    }

    /// Stores the current transform of the [`Frame`] and executes the given
    /// drawing operations, restoring the transform afterwards.
    ///
    /// This method is useful to compose transforms and perform drawing
    /// operations in different coordinate systems.
    ///
    /// [`Frame`]: struct.Frame.html
    #[inline]
    pub fn with_save(&mut self, f: impl FnOnce(&mut Frame)) {
        self.transforms.previous.push(self.transforms.current);

        f(self);

        self.transforms.current = self.transforms.previous.pop().unwrap();
    }

    /// Applies a translation to the current transform of the [`Frame`].
    ///
    /// [`Frame`]: struct.Frame.html
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
    ///
    /// [`Frame`]: struct.Frame.html
    #[inline]
    pub fn rotate(&mut self, angle: f32) {
        self.transforms.current.raw = self
            .transforms
            .current
            .raw
            .pre_rotate(lyon::math::Angle::radians(-angle));
        self.transforms.current.is_identity = false;
    }

    /// Applies a scaling to the current transform of the [`Frame`].
    ///
    /// [`Frame`]: struct.Frame.html
    #[inline]
    pub fn scale(&mut self, scale: f32) {
        self.transforms.current.raw =
            self.transforms.current.raw.pre_scale(scale, scale);
        self.transforms.current.is_identity = false;
    }

    /// Produces the primitive representing everything drawn on the [`Frame`].
    ///
    /// [`Frame`]: struct.Frame.html
    pub fn into_primitive(self, origin: Point) -> Primitive {
        let mut primitives: Vec<Primitive> = self
            .texts
            .into_iter()
            .map(|mut t| {
                t.bounds.x += origin.x;
                t.bounds.y += origin.y;

                Primitive::Text {
                    content: t.content,
                    bounds: t.bounds,
                    color: t.color,
                    size: t.size,
                    font: t.font,
                    horizontal_alignment: t.horizontal_alignment,
                    vertical_alignment: t.vertical_alignment,
                }
            })
            .collect();

        primitives.push(Primitive::Mesh2D {
            origin,
            buffers: Arc::new(triangle::Mesh2D {
                vertices: self.buffers.vertices,
                indices: self.buffers.indices,
            }),
        });

        Primitive::Group { primitives }
    }
}

struct FillVertex([f32; 4]);

impl lyon::tessellation::FillVertexConstructor<triangle::Vertex2D>
    for FillVertex
{
    fn new_vertex(
        &mut self,
        position: lyon::math::Point,
        _attributes: lyon::tessellation::FillAttributes<'_>,
    ) -> triangle::Vertex2D {
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
        position: lyon::math::Point,
        _attributes: lyon::tessellation::StrokeAttributes<'_, '_>,
    ) -> triangle::Vertex2D {
        triangle::Vertex2D {
            position: [position.x, position.y],
            color: self.0,
        }
    }
}
