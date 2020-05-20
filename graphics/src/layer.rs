use crate::image;
use crate::svg;
use crate::triangle;
use crate::{
    Background, Font, HorizontalAlignment, Point, Primitive, Rectangle, Size,
    Vector, VerticalAlignment, Viewport,
};

#[derive(Debug, Clone)]
pub struct Layer<'a> {
    pub bounds: Rectangle,
    pub quads: Vec<Quad>,
    pub meshes: Vec<Mesh<'a>>,
    pub text: Vec<Text<'a>>,
    pub images: Vec<Image>,
}

impl<'a> Layer<'a> {
    pub fn new(bounds: Rectangle) -> Self {
        Self {
            bounds,
            quads: Vec::new(),
            meshes: Vec::new(),
            text: Vec::new(),
            images: Vec::new(),
        }
    }

    pub fn overlay(lines: &'a [impl AsRef<str>], viewport: &Viewport) -> Self {
        let mut overlay =
            Layer::new(Rectangle::with_size(viewport.logical_size()));

        for (i, line) in lines.iter().enumerate() {
            let text = Text {
                content: line.as_ref(),
                bounds: Rectangle::new(
                    Point::new(11.0, 11.0 + 25.0 * i as f32),
                    Size::INFINITY,
                ),
                color: [0.9, 0.9, 0.9, 1.0],
                size: 20.0,
                font: Font::Default,
                horizontal_alignment: HorizontalAlignment::Left,
                vertical_alignment: VerticalAlignment::Top,
            };

            overlay.text.push(text);

            overlay.text.push(Text {
                bounds: text.bounds + Vector::new(-1.0, -1.0),
                color: [0.0, 0.0, 0.0, 1.0],
                ..text
            });
        }

        overlay
    }

    pub fn generate(
        primitive: &'a Primitive,
        viewport: &Viewport,
    ) -> Vec<Self> {
        let first_layer =
            Layer::new(Rectangle::with_size(viewport.logical_size()));

        let mut layers = vec![first_layer];

        Self::process_primitive(&mut layers, Vector::new(0.0, 0.0), primitive);

        layers
    }

    fn process_primitive(
        layers: &mut Vec<Self>,
        translation: Vector,
        primitive: &'a Primitive,
    ) {
        match primitive {
            Primitive::None => {}
            Primitive::Group { primitives } => {
                // TODO: Inspect a bit and regroup (?)
                for primitive in primitives {
                    Self::process_primitive(layers, translation, primitive)
                }
            }
            Primitive::Text {
                content,
                bounds,
                size,
                color,
                font,
                horizontal_alignment,
                vertical_alignment,
            } => {
                let layer = layers.last_mut().unwrap();

                layer.text.push(Text {
                    content,
                    bounds: *bounds + translation,
                    size: *size,
                    color: color.into_linear(),
                    font: *font,
                    horizontal_alignment: *horizontal_alignment,
                    vertical_alignment: *vertical_alignment,
                });
            }
            Primitive::Quad {
                bounds,
                background,
                border_radius,
                border_width,
                border_color,
            } => {
                let layer = layers.last_mut().unwrap();

                // TODO: Move some of these computations to the GPU (?)
                layer.quads.push(Quad {
                    position: [
                        bounds.x + translation.x,
                        bounds.y + translation.y,
                    ],
                    scale: [bounds.width, bounds.height],
                    color: match background {
                        Background::Color(color) => color.into_linear(),
                    },
                    border_radius: *border_radius as f32,
                    border_width: *border_width as f32,
                    border_color: border_color.into_linear(),
                });
            }
            Primitive::Mesh2D { buffers, size } => {
                let layer = layers.last_mut().unwrap();

                let bounds = Rectangle::new(
                    Point::new(translation.x, translation.y),
                    *size,
                );

                // Only draw visible content
                if let Some(clip_bounds) = layer.bounds.intersection(&bounds) {
                    layer.meshes.push(Mesh {
                        origin: Point::new(translation.x, translation.y),
                        buffers,
                        clip_bounds,
                    });
                }
            }
            Primitive::Clip {
                bounds,
                offset,
                content,
            } => {
                let layer = layers.last_mut().unwrap();
                let translated_bounds = *bounds + translation;

                // Only draw visible content
                if let Some(clip_bounds) =
                    layer.bounds.intersection(&translated_bounds)
                {
                    let clip_layer = Layer::new(clip_bounds);
                    let new_layer = Layer::new(layer.bounds);

                    layers.push(clip_layer);
                    Self::process_primitive(
                        layers,
                        translation
                            - Vector::new(offset.x as f32, offset.y as f32),
                        content,
                    );
                    layers.push(new_layer);
                }
            }
            Primitive::Translate {
                translation: new_translation,
                content,
            } => {
                Self::process_primitive(
                    layers,
                    translation + *new_translation,
                    &content,
                );
            }
            Primitive::Cached { cache } => {
                Self::process_primitive(layers, translation, &cache);
            }
            Primitive::Image { handle, bounds } => {
                let layer = layers.last_mut().unwrap();

                layer.images.push(Image::Raster {
                    handle: handle.clone(),
                    bounds: *bounds + translation,
                });
            }
            Primitive::Svg { handle, bounds } => {
                let layer = layers.last_mut().unwrap();

                layer.images.push(Image::Vector {
                    handle: handle.clone(),
                    bounds: *bounds + translation,
                });
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Quad {
    pub position: [f32; 2],
    pub scale: [f32; 2],
    pub color: [f32; 4],
    pub border_color: [f32; 4],
    pub border_radius: f32,
    pub border_width: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Mesh<'a> {
    pub origin: Point,
    pub buffers: &'a triangle::Mesh2D,
    pub clip_bounds: Rectangle<f32>,
}

#[derive(Debug, Clone, Copy)]
pub struct Text<'a> {
    pub content: &'a str,
    pub bounds: Rectangle,
    pub color: [f32; 4],
    pub size: f32,
    pub font: Font,
    pub horizontal_alignment: HorizontalAlignment,
    pub vertical_alignment: VerticalAlignment,
}

#[derive(Debug, Clone)]
pub enum Image {
    Raster {
        handle: image::Handle,
        bounds: Rectangle,
    },
    Vector {
        handle: svg::Handle,
        bounds: Rectangle,
    },
}

unsafe impl bytemuck::Zeroable for Quad {}
unsafe impl bytemuck::Pod for Quad {}
