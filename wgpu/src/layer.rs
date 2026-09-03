use cryoglyph::cosmic_text;

use crate::core::{
    self, Background, Color, Point, Rectangle, Size, Svg, Transformation, Vector, renderer,
};
use crate::graphics;
use crate::graphics::Mesh;
use crate::graphics::color;
use crate::graphics::layer;
use crate::graphics::mesh;
use crate::graphics::text::{Editor, Paragraph};
use crate::image::{self, Image};
use crate::primitive::{self, Primitive};
use crate::quad::{self, Quad};
use crate::text::{self, Text};
use crate::triangle;

pub type Stack = layer::Stack<Layer>;

#[derive(Debug)]
struct TextDecoration {
    rect: Rectangle,
    color: Color,
    transformation: Transformation,
}

#[derive(Debug)]
pub struct Layer {
    pub bounds: Rectangle,
    pub quads: quad::Batch,
    pub triangles: triangle::Batch,
    pub primitives: primitive::Batch,
    pub images: image::Batch,
    pub text: text::Batch,
    pending_meshes: Vec<Mesh>,
    pending_text: Vec<Text>,
    pending_decorations: Vec<TextDecoration>,
}

impl Layer {
    pub fn is_empty(&self) -> bool {
        self.quads.is_empty()
            && self.triangles.is_empty()
            && self.primitives.is_empty()
            && self.images.is_empty()
            && self.text.is_empty()
            && self.pending_meshes.is_empty()
            && self.pending_text.is_empty()
            && self.pending_decorations.is_empty()
    }

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
            border_radius: (quad.border.radius * transformation.scale_factor()).into(),
            border_width: quad.border.width * transformation.scale_factor(),
            shadow_color: color::pack(quad.shadow.color),
            shadow_offset: (quad.shadow.offset * transformation.scale_factor()).into(),
            shadow_blur_radius: quad.shadow.blur_radius * transformation.scale_factor(),
            snap: quad.snap as u32,
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
        self.draw_text_decorations(
            editor.buffer(),
            position,
            color,
            clip_bounds,
            transformation,
        );

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
            line_height: text.line_height.to_absolute(text.size) * transformation.scale_factor(),
            font: text.font,
            align_x: text.align_x,
            align_y: text.align_y,
            shaping: text.shaping,
            wrapping: text.wrapping,
            ellipsis: text.ellipsis,
            clip_bounds: clip_bounds * transformation,
        };

        self.pending_text.push(text);
    }

    pub fn draw_text_raw(&mut self, raw: graphics::text::Raw, transformation: Transformation) {
        let raw = Text::Raw {
            raw,
            transformation,
        };

        self.pending_text.push(raw);
    }

    pub fn draw_image(&mut self, image: Image, transformation: Transformation) {
        match image {
            Image::Raster {
                image,
                bounds,
                clip_bounds,
            } => {
                self.draw_raster(image, bounds, clip_bounds, transformation);
            }
            Image::Vector {
                svg,
                bounds,
                clip_bounds,
            } => {
                self.draw_svg(svg, bounds, clip_bounds, transformation);
            }
        }
    }

    pub fn draw_raster(
        &mut self,
        image: core::Image,
        bounds: Rectangle,
        clip_bounds: Rectangle,
        transformation: Transformation,
    ) {
        let image = Image::Raster {
            image: core::Image {
                border_radius: image.border_radius * transformation.scale_factor(),
                ..image
            },
            bounds: bounds * transformation,
            clip_bounds: clip_bounds * transformation,
        };

        self.images.push(image);
    }

    pub fn draw_svg(
        &mut self,
        svg: Svg,
        bounds: Rectangle,
        clip_bounds: Rectangle,
        transformation: Transformation,
    ) {
        let svg = Image::Vector {
            svg,
            bounds: bounds * transformation,
            clip_bounds: clip_bounds * transformation,
        };

        self.images.push(svg);
    }

    pub fn draw_mesh(&mut self, mut mesh: Mesh, transformation: Transformation) {
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

    pub fn draw_mesh_group(&mut self, meshes: Vec<Mesh>, transformation: Transformation) {
        self.flush_meshes();

        self.triangles.push(triangle::Item::Group {
            meshes,
            transformation,
        });
    }

    pub fn draw_mesh_cache(&mut self, cache: mesh::Cache, transformation: Transformation) {
        self.flush_meshes();

        self.triangles.push(triangle::Item::Cached {
            cache,
            transformation,
        });
    }

    pub fn draw_text_group(&mut self, text: Vec<Text>, transformation: Transformation) {
        self.flush_text();

        self.text.push(text::Item::Group {
            text,
            transformation,
        });
    }

    pub fn draw_text_cache(&mut self, cache: text::Cache, transformation: Transformation) {
        self.flush_text();

        self.text.push(text::Item::Cached {
            cache,
            transformation,
        });
    }

    pub fn draw_primitive(
        &mut self,
        bounds: Rectangle,
        primitive: impl Primitive,
        transformation: Transformation,
    ) {
        let bounds = bounds * transformation;

        self.primitives
            .push(primitive::Instance::new(bounds, primitive));
    }

    fn draw_text_decorations(
        &mut self,
        buffer: &cosmic_text::Buffer,
        position: Point,
        default_color: Color,
        clip_bounds: Rectangle,
        transformation: Transformation,
    ) {
        for run in buffer.layout_runs() {
            for span in run.decorations {
                let glyphs = &run.glyphs[span.glyph_range.clone()];
                if glyphs.is_empty() {
                    continue;
                }

                let deco = &span.data;
                let td = &deco.text_decoration;
                let font_size = span.font_size;

                // Compute x extent as min/max over all glyphs, not first/last,
                // because RTL paragraphs store glyphs in right-to-left order.
                let mut x_min = f32::INFINITY;
                let mut x_max = f32::NEG_INFINITY;
                for g in glyphs {
                    x_min = x_min.min(g.x);
                    x_max = x_max.max(g.x + g.w);
                }
                let width = x_max - x_min;
                if width <= 0.0 {
                    continue;
                }

                let mut record_quad = |x: f32, y: f32, w: f32, h: f32, color: Color| {
                    let rect = Rectangle::new(position + Vector::new(x, y), Size::new(w, h));
                    if let Some(clipped_bounds) = rect.intersection(&clip_bounds) {
                        self.pending_decorations.push(TextDecoration {
                            rect: clipped_bounds,
                            color,
                            transformation,
                        });
                    }
                };

                // Underline
                match td.underline {
                    cosmic_text::UnderlineStyle::None => {}
                    cosmic_text::UnderlineStyle::Single => {
                        let color = td
                            .underline_color_opt
                            .or(span.color_opt)
                            .map(from_color)
                            .unwrap_or(default_color);
                        let thickness = (deco.underline_metrics.thickness * font_size)
                            .max(1.0)
                            .ceil();
                        let y = run.line_y - deco.underline_metrics.offset * font_size;
                        record_quad(x_min, y, width, thickness, color);
                    }
                    cosmic_text::UnderlineStyle::Double => {
                        let color = td
                            .underline_color_opt
                            .or(span.color_opt)
                            .map(from_color)
                            .unwrap_or(default_color);
                        let thickness = (deco.underline_metrics.thickness * font_size)
                            .max(1.0)
                            .ceil();
                        let gap = thickness;
                        let y = run.line_y - deco.underline_metrics.offset * font_size;
                        record_quad(x_min, y, width, thickness, color);
                        record_quad(x_min, y + thickness + gap, width, thickness, color);
                    }
                }

                // Strikethrough
                if td.strikethrough {
                    let color = td
                        .strikethrough_color_opt
                        .or(span.color_opt)
                        .map(from_color)
                        .unwrap_or(default_color);
                    let thickness = (deco.strikethrough_metrics.thickness * font_size)
                        .max(1.0)
                        .ceil();
                    let y = run.line_y - deco.strikethrough_metrics.offset * font_size;
                    record_quad(x_min, y, width, thickness, color);
                }

                // Overline
                if td.overline {
                    let color = td
                        .overline_color_opt
                        .or(span.color_opt)
                        .map(from_color)
                        .unwrap_or(default_color);
                    // Reuse underline thickness for overline
                    let thickness = (deco.underline_metrics.thickness * font_size)
                        .max(1.0)
                        .ceil();
                    // clamped so it doesn't go above the line top.
                    let y = (run.line_y - deco.ascent * font_size).max(run.line_top);
                    record_quad(x_min, y, width, thickness, color);
                }
            }
        }
    }

    #[allow(clippy::drain_collect)] // We want to reuse `Layer` capacity
    fn flush_meshes(&mut self) {
        if !self.pending_meshes.is_empty() {
            self.triangles.push(triangle::Item::Group {
                transformation: Transformation::IDENTITY,
                meshes: self.pending_meshes.drain(..).collect(),
            });
        }
    }

    #[allow(clippy::drain_collect)] // We want to reuse `Layer` capacity
    fn flush_text(&mut self) {
        for decoration in self.pending_decorations.drain(..).collect::<Vec<_>>() {
            self.draw_quad(
                renderer::Quad {
                    bounds: decoration.rect,
                    ..renderer::Quad::default()
                },
                Background::Color(decoration.color),
                decoration.transformation,
            );
        }

        if !self.pending_text.is_empty() {
            self.text.push(text::Item::Group {
                transformation: Transformation::IDENTITY,
                text: self.pending_text.drain(..).collect(),
            });
        }
    }
}

fn from_color(color: cosmic_text::Color) -> Color {
    let [r, g, b, a] = color.as_rgba();

    Color::from_rgba8(r, g, b, a as f32 / 255.0)
}

impl graphics::Layer for Layer {
    fn with_bounds(bounds: Rectangle) -> Self {
        Self {
            bounds,
            ..Self::default()
        }
    }

    fn bounds(&self) -> Rectangle {
        self.bounds
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
        self.pending_decorations.clear();
    }

    fn start(&self) -> usize {
        if !self.quads.is_empty() || !self.pending_decorations.is_empty() {
            return 1;
        }

        if !self.triangles.is_empty() {
            return 2;
        }

        if !self.primitives.is_empty() {
            return 3;
        }

        if !self.images.is_empty() {
            return 4;
        }

        if !self.text.is_empty() {
            return 5;
        }

        usize::MAX
    }

    fn end(&self) -> usize {
        if !self.text.is_empty() {
            return 5;
        }

        if !self.images.is_empty() {
            return 4;
        }

        if !self.primitives.is_empty() {
            return 3;
        }

        if !self.triangles.is_empty() {
            return 2;
        }

        if !self.quads.is_empty() || !self.pending_decorations.is_empty() {
            return 1;
        }

        0
    }

    fn merge(&mut self, layer: &mut Self) {
        self.quads.append(&mut layer.quads);
        self.triangles.append(&mut layer.triangles);
        self.primitives.append(&mut layer.primitives);
        self.images.append(&mut layer.images);
        self.text.append(&mut layer.text);
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
            pending_decorations: Vec::new(),
        }
    }
}
