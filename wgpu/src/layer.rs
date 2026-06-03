use crate::core::{self, Background, Color, Point, Rectangle, Svg, Transformation, renderer};
use crate::graphics;
use crate::graphics::Mesh;
use crate::graphics::color;
use crate::graphics::layer;
use crate::graphics::mesh;
use crate::graphics::text::{Editor, Paragraph, cosmic_text};
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
    pub images: image::Batch,
    pub text: text::Batch,
    /// Quads drawn *after* the text of this layer (e.g. strikethrough), so they
    /// overlay the glyphs instead of being occluded by them.
    pub overlay_quads: quad::Batch,
    pending_meshes: Vec<Mesh>,
    pending_text: Vec<Text>,
}

impl Layer {
    pub fn is_empty(&self) -> bool {
        self.quads.is_empty()
            && self.triangles.is_empty()
            && self.primitives.is_empty()
            && self.images.is_empty()
            && self.text.is_empty()
            && self.overlay_quads.is_empty()
            && self.pending_meshes.is_empty()
            && self.pending_text.is_empty()
    }

    pub fn draw_quad(
        &mut self,
        quad: renderer::Quad,
        background: Background,
        transformation: Transformation,
    ) {
        self.quads
            .add(to_gpu_quad(quad, transformation), &background);
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
        // Quads render before text within a layer, so highlight backgrounds and
        // underlines drawn here sit behind/below the glyphs (correct z-order).
        self.add_text_decorations(editor.buffer(), position, color, transformation);

        let editor = Text::Editor {
            editor: editor.downgrade(),
            position,
            color,
            clip_bounds,
            transformation,
        };

        self.pending_text.push(editor);
    }

    /// Adds highlight-background and underline decoration quads for a laid-out
    /// text `buffer` to this layer's quad batch.
    ///
    /// A layer renders its quads before its text, so backgrounds (behind the
    /// glyphs) and underlines (below the baseline) go in the normal quad batch.
    /// Strikethrough crosses the glyphs, so it must render *after* the text — it
    /// goes in `overlay_quads`, which is rendered as a post-text pass.
    fn add_text_decorations(
        &mut self,
        buffer: &cosmic_text::Buffer,
        position: Point,
        color: Color,
        transformation: Transformation,
    ) {
        let decoration_color = |decoration: cosmic_text::Decoration, glyph_color| match decoration
            .color_opt
            .or(glyph_color)
        {
            Some(c) => {
                let mut fill = from_cosmic_color(c);
                fill.a *= color.a;
                fill
            }
            None => color,
        };

        for run in buffer.layout_runs() {
            // Highlight backgrounds (behind the glyphs).
            for glyph in run.glyphs {
                let Some(background) = glyph.background_opt else {
                    continue;
                };
                let mut fill = from_cosmic_color(background);
                fill.a *= color.a;
                self.fill_decoration(
                    Rectangle {
                        x: position.x + glyph.x,
                        y: position.y + run.line_top,
                        width: glyph.w,
                        height: run.line_height,
                    },
                    fill,
                    transformation,
                    false,
                );
            }

            // Underlines (below the baseline).
            for glyph in run.glyphs {
                let Some(underline) = glyph.underline_opt else {
                    continue;
                };
                let thickness = (glyph.font_size * 0.06).max(1.0);
                self.fill_decoration(
                    Rectangle {
                        x: position.x + glyph.x,
                        y: position.y + run.line_y + glyph.font_size * 0.1,
                        width: glyph.w,
                        height: thickness,
                    },
                    decoration_color(underline, glyph.color_opt),
                    transformation,
                    false,
                );
            }

            // Strikethrough (over the glyphs, via the overlay batch).
            for glyph in run.glyphs {
                let Some(strikethrough) = glyph.strikethrough_opt else {
                    continue;
                };
                let thickness = (glyph.font_size * 0.06).max(1.0);
                self.fill_decoration(
                    Rectangle {
                        x: position.x + glyph.x,
                        y: position.y + run.line_y - glyph.font_size * 0.3,
                        width: glyph.w,
                        height: thickness,
                    },
                    decoration_color(strikethrough, glyph.color_opt),
                    transformation,
                    true,
                );
            }
        }
    }

    fn fill_decoration(
        &mut self,
        bounds: Rectangle,
        fill: Color,
        transformation: Transformation,
        overlay: bool,
    ) {
        if bounds.width <= 0.0 || bounds.height <= 0.0 || fill.a <= 0.0 {
            return;
        }
        let quad = to_gpu_quad(
            renderer::Quad {
                bounds,
                border: core::Border::default(),
                shadow: core::Shadow::default(),
                snap: true,
                border_only: false,
            },
            transformation,
        );
        let background = Background::Color(fill);
        if overlay {
            self.overlay_quads.add(quad, &background);
        } else {
            self.quads.add(quad, &background);
        }
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
            letter_spacing: text.letter_spacing,
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
        self.overlay_quads.clear();
        self.pending_meshes.clear();
        self.pending_text.clear();
    }

    fn start(&self) -> usize {
        if !self.quads.is_empty() {
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

        if !self.overlay_quads.is_empty() {
            return 6;
        }

        usize::MAX
    }

    fn end(&self) -> usize {
        if !self.overlay_quads.is_empty() {
            return 6;
        }

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

        if !self.quads.is_empty() {
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
        self.overlay_quads.append(&mut layer.overlay_quads);
    }
}

impl Default for Layer {
    #[allow(clippy::default_constructed_unit_structs)]
    fn default() -> Self {
        Self {
            bounds: Rectangle::INFINITE,
            quads: quad::Batch::default(),
            triangles: triangle::Batch::default(),
            primitives: primitive::Batch::default(),
            text: text::Batch::default(),
            images: image::Batch::default(),
            overlay_quads: quad::Batch::default(),
            pending_meshes: Vec::new(),
            pending_text: Vec::new(),
        }
    }
}

/// Build a GPU [`Quad`] from a renderer quad under a transformation.
fn to_gpu_quad(quad: renderer::Quad, transformation: Transformation) -> Quad {
    let bounds = quad.bounds * transformation;

    Quad {
        position: [bounds.x, bounds.y],
        size: [bounds.width, bounds.height],
        border_color: color::pack(quad.border.color),
        border_radius: (quad.border.radius * transformation.scale_factor()).into(),
        border_widths: {
            let w = quad.border.widths();
            let s = transformation.scale_factor();
            [w[0] * s, w[1] * s, w[2] * s, w[3] * s]
        },
        shadow_color: color::pack(quad.shadow.color),
        shadow_offset: (quad.shadow.offset * transformation.scale_factor()).into(),
        shadow_blur_radius: quad.shadow.blur_radius * transformation.scale_factor(),
        shadow_inset: quad.shadow.inset as u32,
        shadow_spread_radius: quad.shadow.spread_radius * transformation.scale_factor(),
        snap: quad.snap as u32,
        border_only: quad.border_only as u32,
    }
}

/// Convert a cosmic-text color to an iced [`Color`].
fn from_cosmic_color(color: cosmic_text::Color) -> Color {
    let [r, g, b, a] = color.as_rgba();
    Color::from_rgba8(r, g, b, a as f32 / 255.0)
}
