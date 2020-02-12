use iced_native::Point;

use crate::{
    canvas::{Fill, Path, Stroke},
    triangle,
};

#[derive(Debug)]
pub struct Frame {
    width: f32,
    height: f32,
    buffers: lyon::tessellation::VertexBuffers<triangle::Vertex2D, u16>,
}

impl Frame {
    pub fn new(width: f32, height: f32) -> Frame {
        Frame {
            width,
            height,
            buffers: lyon::tessellation::VertexBuffers::new(),
        }
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn center(&self) -> Point {
        Point::new(self.width / 2.0, self.height / 2.0)
    }

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

        let _ = tessellator
            .tessellate_path(path.raw(), &FillOptions::default(), &mut buffers)
            .expect("Tessellate path");
    }

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

        let _ = tessellator
            .tessellate_path(path.raw(), &options, &mut buffers)
            .expect("Stroke path");
    }

    pub fn into_mesh(self) -> triangle::Mesh2D {
        triangle::Mesh2D {
            vertices: self.buffers.vertices,
            indices: self.buffers.indices,
        }
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
