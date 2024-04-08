use crate::core::renderer;
use crate::core::{Background, Color, Point, Rectangle, Transformation};
use crate::graphics::color;
use crate::graphics::text::{Editor, Paragraph};
use crate::graphics::Mesh;
use crate::image::{self, Image};
use crate::primitive::{self, Primitive};
use crate::quad::{self, Quad};
use crate::text::{self, Text};
use crate::triangle;

#[derive(Debug)]
pub struct Layer {
    pub bounds: Rectangle,
    pub quads: quad::Batch,
    pub triangles: triangle::Batch,
    pub primitives: primitive::Batch,
    pub text: text::Batch,
    pub images: image::Batch,
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
        }
    }
}

#[derive(Debug)]
pub struct Stack {
    layers: Vec<Layer>,
    transformations: Vec<Transformation>,
    previous: Vec<usize>,
    pending_meshes: Vec<Vec<Mesh>>,
    pending_text: Vec<Vec<Text>>,
    current: usize,
    active_count: usize,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            layers: vec![Layer::default()],
            transformations: vec![Transformation::IDENTITY],
            previous: vec![],
            pending_meshes: vec![Vec::new()],
            pending_text: vec![Vec::new()],
            current: 0,
            active_count: 1,
        }
    }

    pub fn draw_quad(&mut self, quad: renderer::Quad, background: Background) {
        let transformation = self.transformations.last().unwrap();
        let bounds = quad.bounds * *transformation;

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

        self.layers[self.current].quads.add(quad, &background);
    }

    pub fn draw_paragraph(
        &mut self,
        paragraph: &Paragraph,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        let paragraph = Text::Paragraph {
            paragraph: paragraph.downgrade(),
            position,
            color,
            clip_bounds,
            transformation: self.transformations.last().copied().unwrap(),
        };

        self.pending_text[self.current].push(paragraph);
    }

    pub fn draw_editor(
        &mut self,
        editor: &Editor,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        let editor = Text::Editor {
            editor: editor.downgrade(),
            position,
            color,
            clip_bounds,
            transformation: self.transformation(),
        };

        self.pending_text[self.current].push(editor);
    }

    pub fn draw_text(
        &mut self,
        text: crate::core::Text,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        let transformation = self.transformation();

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

        self.pending_text[self.current].push(text);
    }

    pub fn draw_image(
        &mut self,
        handle: crate::core::image::Handle,
        filter_method: crate::core::image::FilterMethod,
        bounds: Rectangle,
    ) {
        let image = Image::Raster {
            handle,
            filter_method,
            bounds: bounds * self.transformation(),
        };

        self.layers[self.current].images.push(image);
    }

    pub fn draw_svg(
        &mut self,
        handle: crate::core::svg::Handle,
        color: Option<Color>,
        bounds: Rectangle,
    ) {
        let svg = Image::Vector {
            handle,
            color,
            bounds: bounds * self.transformation(),
        };

        self.layers[self.current].images.push(svg);
    }

    pub fn draw_mesh(&mut self, mut mesh: Mesh) {
        match &mut mesh {
            Mesh::Solid { transformation, .. }
            | Mesh::Gradient { transformation, .. } => {
                *transformation = *transformation * self.transformation();
            }
        }

        self.pending_meshes[self.current].push(mesh);
    }

    pub fn draw_mesh_group(&mut self, meshes: Vec<Mesh>) {
        self.flush_pending();

        let transformation = self.transformation();

        self.layers[self.current]
            .triangles
            .push(triangle::Item::Group {
                transformation,
                meshes,
            });
    }

    pub fn draw_mesh_cache(&mut self, cache: triangle::Cache) {
        self.flush_pending();

        let transformation = self.transformation();

        self.layers[self.current]
            .triangles
            .push(triangle::Item::Cached {
                transformation,
                cache,
            });
    }

    pub fn draw_text_group(&mut self, text: Vec<Text>) {
        self.flush_pending();

        let transformation = self.transformation();

        self.layers[self.current].text.push(text::Item::Group {
            transformation,
            text,
        });
    }

    pub fn draw_text_cache(&mut self, cache: text::Cache) {
        self.flush_pending();

        let transformation = self.transformation();

        self.layers[self.current].text.push(text::Item::Cached {
            transformation,
            cache,
        });
    }

    pub fn draw_primitive(
        &mut self,
        bounds: Rectangle,
        primitive: Box<dyn Primitive>,
    ) {
        let bounds = bounds * self.transformation();

        self.layers[self.current]
            .primitives
            .push(primitive::Instance { bounds, primitive });
    }

    pub fn push_clip(&mut self, bounds: Rectangle) {
        self.previous.push(self.current);

        self.current = self.active_count;
        self.active_count += 1;

        let bounds = bounds * self.transformation();

        if self.current == self.layers.len() {
            self.layers.push(Layer {
                bounds,
                ..Layer::default()
            });
            self.pending_meshes.push(Vec::new());
            self.pending_text.push(Vec::new());
        } else {
            self.layers[self.current].bounds = bounds;
        }
    }

    pub fn pop_clip(&mut self) {
        self.flush_pending();

        self.current = self.previous.pop().unwrap();
    }

    pub fn push_transformation(&mut self, transformation: Transformation) {
        self.flush_pending();

        self.transformations
            .push(self.transformation() * transformation);
    }

    pub fn pop_transformation(&mut self) {
        let _ = self.transformations.pop();
    }

    fn transformation(&self) -> Transformation {
        self.transformations.last().copied().unwrap()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Layer> {
        self.flush_pending();

        self.layers[..self.active_count].iter_mut()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Layer> {
        self.layers[..self.active_count].iter()
    }

    pub fn clear(&mut self) {
        for (live, pending_meshes) in self.layers[..self.active_count]
            .iter_mut()
            .zip(self.pending_meshes.iter_mut())
        {
            live.bounds = Rectangle::INFINITE;

            live.quads.clear();
            live.triangles.clear();
            live.primitives.clear();
            live.text.clear();
            live.images.clear();
            pending_meshes.clear();
        }

        self.current = 0;
        self.active_count = 1;
        self.previous.clear();
    }

    // We want to keep the allocated memory
    #[allow(clippy::drain_collect)]
    fn flush_pending(&mut self) {
        let transformation = self.transformation();

        let pending_meshes = &mut self.pending_meshes[self.current];
        if !pending_meshes.is_empty() {
            self.layers[self.current]
                .triangles
                .push(triangle::Item::Group {
                    transformation,
                    meshes: pending_meshes.drain(..).collect(),
                });
        }

        let pending_text = &mut self.pending_text[self.current];
        if !pending_text.is_empty() {
            self.layers[self.current].text.push(text::Item::Group {
                transformation,
                text: pending_text.drain(..).collect(),
            });
        }
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}
