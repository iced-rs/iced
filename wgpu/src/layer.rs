use crate::core::{
    renderer, Background, Color, Point, Radians, Rectangle, Transformation,
};
use crate::graphics;
use crate::graphics::color;
use crate::graphics::layer;
use crate::graphics::text::{Editor, Paragraph};
use crate::graphics::Mesh;
use crate::image::{self, Image};
use crate::primitive::{self, Primitive};
use crate::quad::{self, Quad};
use crate::text::{self, Text};
use crate::triangle;

pub type Stack = layer::Stack<Layer>;

#[derive(Debug)]
pub struct Layer {
    pub bounds: Rectangle,
    pub quads: quad::Batch,
    pub triangles: triangle::Batch,
    pub primitives: primitive::Batch,
    pub text: text::Batch,
    pub images: image::Batch,
    pending_meshes: Vec<Mesh>,
    pending_text: Vec<Text>,
}

impl Layer {
    pub fn draw_quad(
        &mut self,
        quad: renderer::Quad,
        background: Background,
        transformation: Transformation,
    ) {
        let bounds = quad.bounds * transformation;

        let quad = Quad {
            position: [bounds.x, bounds.y],
            size: [bounds.width, bounds.height],
            border_color: color::pack(quad.border.color),
            border_radius: quad.border.radius.into(),
            border_width: quad.border.width,
            shadow_color: color::pack(quad.shadow.color),
            shadow_offset: quad.shadow.offset.into(),
            shadow_blur_radius: quad.shadow.blur_radius,
        };

        self.quads.add(quad, &background);
    }

    pub fn draw_paragraph(
        &mut self,
        paragraph: &Paragraph,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
        transformation: Transformation,
    ) {
        let paragraph = Text::Paragraph {
            paragraph: paragraph.downgrade(),
            position,
            color,
            clip_bounds,
            transformation,
        };

        self.pending_text.push(paragraph);
    }

    pub fn draw_editor(
        &mut self,
        editor: &Editor,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
        transformation: Transformation,
    ) {
        let editor = Text::Editor {
            editor: editor.downgrade(),
            position,
            color,
            clip_bounds,
            transformation,
        };

        self.pending_text.push(editor);
    }

    pub fn draw_text(
        &mut self,
        text: crate::core::Text,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
        transformation: Transformation,
    ) {
        let text = Text::Cached {
            content: text.content,
            bounds: Rectangle::new(position, text.bounds) * transformation,
            color,
            size: text.size * transformation.scale_factor(),
            line_height: text.line_height.to_absolute(text.size)
                * transformation.scale_factor(),
            font: text.font,
            horizontal_alignment: text.horizontal_alignment,
            vertical_alignment: text.vertical_alignment,
            shaping: text.shaping,
            clip_bounds: clip_bounds * transformation,
        };

        self.pending_text.push(text);
    }

    pub fn draw_image(
        &mut self,
        handle: crate::core::image::Handle,
        filter_method: crate::core::image::FilterMethod,
        bounds: Rectangle,
        transformation: Transformation,
        rotation: Radians,
        opacity: f32,
    ) {
        let image = Image::Raster {
            handle,
            filter_method,
            bounds: bounds * transformation,
            rotation,
            opacity,
        };

        self.images.push(image);
    }

    pub fn draw_svg(
        &mut self,
        handle: crate::core::svg::Handle,
        color: Option<Color>,
        bounds: Rectangle,
        transformation: Transformation,
        rotation: Radians,
        opacity: f32,
    ) {
        let svg = Image::Vector {
            handle,
            color,
            bounds: bounds * transformation,
            rotation,
            opacity,
        };

        self.images.push(svg);
    }

    pub fn draw_mesh(
        &mut self,
        mut mesh: Mesh,
        transformation: Transformation,
    ) {
        match &mut mesh {
            Mesh::Solid {
                transformation: local_transformation,
                ..
            }
            | Mesh::Gradient {
                transformation: local_transformation,
                ..
            } => {
                *local_transformation = *local_transformation * transformation;
            }
        }

        self.pending_meshes.push(mesh);
    }

    pub fn draw_mesh_group(
        &mut self,
        meshes: Vec<Mesh>,
        transformation: Transformation,
    ) {
        self.flush_meshes();

        self.triangles.push(triangle::Item::Group {
            meshes,
            transformation,
        });
    }

    pub fn draw_mesh_cache(
        &mut self,
        cache: triangle::Cache,
        transformation: Transformation,
    ) {
        self.flush_meshes();

        self.triangles.push(triangle::Item::Cached {
            cache,
            transformation,
        });
    }

    pub fn draw_text_group(
        &mut self,
        text: Vec<Text>,
        transformation: Transformation,
    ) {
        self.flush_text();

        self.text.push(text::Item::Group {
            text,
            transformation,
        });
    }

    pub fn draw_text_cache(
        &mut self,
        cache: text::Cache,
        transformation: Transformation,
    ) {
        self.flush_text();

        self.text.push(text::Item::Cached {
            cache,
            transformation,
        });
    }

    pub fn draw_primitive(
        &mut self,
        bounds: Rectangle,
        primitive: Box<dyn Primitive>,
        transformation: Transformation,
    ) {
        let bounds = bounds * transformation;

        self.primitives
            .push(primitive::Instance { bounds, primitive });
    }

    fn flush_meshes(&mut self) {
        if !self.pending_meshes.is_empty() {
            self.triangles.push(triangle::Item::Group {
                transformation: Transformation::IDENTITY,
                meshes: self.pending_meshes.drain(..).collect(),
            });
        }
    }

    fn flush_text(&mut self) {
        if !self.pending_text.is_empty() {
            self.text.push(text::Item::Group {
                transformation: Transformation::IDENTITY,
                text: self.pending_text.drain(..).collect(),
            });
        }
    }
}

impl graphics::Layer for Layer {
    fn with_bounds(bounds: Rectangle) -> Self {
        Self {
            bounds,
            ..Self::default()
        }
    }

    fn flush(&mut self) {
        self.flush_meshes();
        self.flush_text();
    }

    fn resize(&mut self, bounds: Rectangle) {
        self.bounds = bounds;
    }

    fn reset(&mut self) {
        self.bounds = Rectangle::INFINITE;

        self.quads.clear();
        self.triangles.clear();
        self.primitives.clear();
        self.text.clear();
        self.images.clear();
        self.pending_meshes.clear();
        self.pending_text.clear();
    }
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            bounds: Rectangle::INFINITE,
            quads: quad::Batch::default(),
            triangles: triangle::Batch::default(),
            primitives: primitive::Batch::default(),
            text: text::Batch::default(),
            images: image::Batch::default(),
            pending_meshes: Vec::new(),
            pending_text: Vec::new(),
        }
    }
}
