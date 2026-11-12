use crate::core::renderer::Quad;
use crate::core::{self, Background, Color, Point, Rectangle, Svg, Transformation};
use crate::graphics::damage;
use crate::graphics::layer;
use crate::graphics::text::{Editor, Paragraph, Text};
use crate::graphics::{self, Image};

#[cfg(feature = "geometry")]
use crate::geometry;

use std::sync::Arc;

pub type Stack = layer::Stack<Layer>;

#[derive(Debug, Clone)]
pub struct Layer {
    pub bounds: Rectangle,
    pub quads: Vec<(Quad, Background)>,
    #[cfg(feature = "geometry")]
    pub primitives: Vec<Item<geometry::Primitive>>,
    pub images: Vec<Image>,
    pub text: Vec<Item<Text>>,
}

impl Layer {
    pub fn draw_quad(
        &mut self,
        mut quad: Quad,
        background: Background,
        transformation: Transformation,
    ) {
        quad.bounds = quad.bounds * transformation;
        self.quads.push((quad, background));
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

        self.text.push(Item::Live(paragraph));
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

        self.text.push(Item::Live(editor));
    }

    pub fn draw_text(
        &mut self,
        text: core::Text,
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

        self.text.push(Item::Live(text));
    }

    pub fn draw_text_raw(&mut self, raw: graphics::text::Raw, transformation: Transformation) {
        let raw = Text::Raw {
            raw,
            transformation,
        };

        self.text.push(Item::Live(raw));
    }

    pub fn draw_text_group(
        &mut self,
        text: Vec<Text>,
        clip_bounds: Rectangle,
        transformation: Transformation,
    ) {
        self.text
            .push(Item::Group(text, clip_bounds, transformation));
    }

    pub fn draw_text_cache(
        &mut self,
        text: Arc<[Text]>,
        clip_bounds: Rectangle,
        transformation: Transformation,
    ) {
        self.text
            .push(Item::Cached(text, clip_bounds, transformation));
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

    #[cfg(feature = "geometry")]
    pub fn draw_primitive_group(
        &mut self,
        primitives: Vec<geometry::Primitive>,
        clip_bounds: Rectangle,
        transformation: Transformation,
    ) {
        self.primitives.push(Item::Group(
            primitives,
            clip_bounds * transformation,
            transformation,
        ));
    }

    #[cfg(feature = "geometry")]
    pub fn draw_primitive_cache(
        &mut self,
        primitives: Arc<[geometry::Primitive]>,
        clip_bounds: Rectangle,
        transformation: Transformation,
    ) {
        self.primitives.push(Item::Cached(
            primitives,
            clip_bounds * transformation,
            transformation,
        ));
    }

    pub fn damage(previous: &Self, current: &Self) -> Vec<Rectangle> {
        if previous.bounds != current.bounds {
            return vec![previous.bounds, current.bounds];
        }

        let layer_bounds = current.bounds.expand(1.0);

        let mut damage = damage::list(
            &previous.quads,
            &current.quads,
            |(quad, _)| {
                let Some(bounds) = quad.bounds.expand(1.0).intersection(&layer_bounds) else {
                    return vec![];
                };

                vec![if quad.shadow.color.a > 0.0 {
                    bounds.expand(
                        quad.shadow.offset.x.abs().max(quad.shadow.offset.y.abs())
                            + quad.shadow.blur_radius,
                    )
                } else {
                    bounds
                }]
            },
            |(quad_a, background_a), (quad_b, background_b)| {
                quad_a == quad_b && background_a == background_b
            },
        );

        let text = damage::diff(
            &previous.text,
            &current.text,
            |item| {
                item.as_slice()
                    .iter()
                    .filter_map(Text::visible_bounds)
                    .map(|bounds| bounds * item.transformation())
                    .collect()
            },
            |text_a, text_b| {
                damage::list(
                    text_a.as_slice(),
                    text_b.as_slice(),
                    |text| {
                        text.visible_bounds()
                            .into_iter()
                            .map(|bounds| bounds * text_a.transformation())
                            .collect()
                    },
                    |text_a, text_b| text_a == text_b,
                )
            },
        );

        #[cfg(feature = "geometry")]
        let primitives = damage::list(
            &previous.primitives,
            &current.primitives,
            |item| match item {
                Item::Live(primitive) => vec![primitive.visible_bounds()],
                Item::Group(primitives, group_bounds, transformation) => primitives
                    .as_slice()
                    .iter()
                    .map(geometry::Primitive::visible_bounds)
                    .map(|bounds| bounds * *transformation)
                    .filter_map(|bounds| bounds.intersection(group_bounds))
                    .collect(),
                Item::Cached(_primitives, bounds, _transformation) => {
                    vec![*bounds]
                }
            },
            |primitive_a, primitive_b| match (primitive_a, primitive_b) {
                (
                    Item::Cached(cache_a, bounds_a, transformation_a),
                    Item::Cached(cache_b, bounds_b, transformation_b),
                ) => {
                    Arc::ptr_eq(cache_a, cache_b)
                        && bounds_a == bounds_b
                        && transformation_a == transformation_b
                }
                _ => false,
            },
        );

        let images = damage::list(
            &previous.images,
            &current.images,
            |image| vec![image.bounds().expand(1.0)],
            Image::eq,
        );

        damage.extend(text);
        #[cfg(feature = "geometry")]
        damage.extend(primitives);
        damage.extend(images);
        damage
    }
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            bounds: Rectangle::INFINITE,
            quads: Vec::new(),
            #[cfg(feature = "geometry")]
            primitives: Vec::new(),
            text: Vec::new(),
            images: Vec::new(),
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

    fn flush(&mut self) {}

    fn resize(&mut self, bounds: Rectangle) {
        self.bounds = bounds;
    }

    fn reset(&mut self) {
        self.bounds = Rectangle::INFINITE;

        self.quads.clear();
        #[cfg(feature = "geometry")]
        self.primitives.clear();
        self.text.clear();
        self.images.clear();
    }

    fn start(&self) -> usize {
        if !self.quads.is_empty() {
            return 1;
        }

        #[cfg(feature = "geometry")]
        if !self.primitives.is_empty() {
            return 2;
        }

        if !self.images.is_empty() {
            return 3;
        }

        if !self.text.is_empty() {
            return 4;
        }

        usize::MAX
    }

    fn end(&self) -> usize {
        if !self.text.is_empty() {
            return 4;
        }

        if !self.images.is_empty() {
            return 3;
        }

        #[cfg(feature = "geometry")]
        if !self.primitives.is_empty() {
            return 2;
        }

        if !self.quads.is_empty() {
            return 1;
        }

        0
    }

    fn merge(&mut self, layer: &mut Self) {
        self.quads.append(&mut layer.quads);
        #[cfg(feature = "geometry")]
        self.primitives.append(&mut layer.primitives);
        self.text.append(&mut layer.text);
        self.images.append(&mut layer.images);
    }
}

#[derive(Debug, Clone)]
pub enum Item<T> {
    Live(T),
    Group(Vec<T>, Rectangle, Transformation),
    Cached(Arc<[T]>, Rectangle, Transformation),
}

impl<T> Item<T> {
    pub fn transformation(&self) -> Transformation {
        match self {
            Item::Live(_) => Transformation::IDENTITY,
            Item::Group(_, _, transformation) | Item::Cached(_, _, transformation) => {
                *transformation
            }
        }
    }

    pub fn clip_bounds(&self) -> Rectangle {
        match self {
            Item::Live(_) => Rectangle::INFINITE,
            Item::Group(_, clip_bounds, _) | Item::Cached(_, clip_bounds, _) => *clip_bounds,
        }
    }

    pub fn as_slice(&self) -> &[T] {
        match self {
            Item::Live(item) => std::slice::from_ref(item),
            Item::Group(group, _, _) => group.as_slice(),
            Item::Cached(cache, _, _) => cache,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Shadow, Vector};

    fn quad(bounds: Rectangle) -> Quad {
        Quad {
            bounds,
            ..Default::default()
        }
    }

    fn background() -> Background {
        Background::Color(Color::from_rgb(1.0, 0.0, 0.0))
    }

    fn layer(bounds: Rectangle) -> Layer {
        Layer {
            bounds,
            ..Default::default()
        }
    }

    #[test]
    fn identical_layers_have_no_damage() {
        let mut layer = layer(Rectangle {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        });
        layer.quads.push((
            quad(Rectangle {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            }),
            background(),
        ));

        assert!(Layer::damage(&layer, &layer).is_empty());
    }

    #[test]
    fn bounds_change_has_both_bounds_as_damage() {
        let previous = layer(Rectangle {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        });
        let current = layer(Rectangle {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 200.0,
        });

        assert_eq!(
            Layer::damage(&previous, &current),
            vec![previous.bounds, current.bounds],
        );
    }

    #[test]
    fn moved_quad_has_old_and_new_bounds_as_damage() {
        let bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };

        let mut previous = layer(bounds);
        previous.quads.push((
            quad(Rectangle {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            }),
            background(),
        ));

        let mut current = layer(bounds);
        current.quads.push((
            quad(Rectangle {
                x: 20.0,
                y: 20.0,
                width: 10.0,
                height: 10.0,
            }),
            background(),
        ));

        assert_eq!(
            Layer::damage(&previous, &current),
            vec![
                Rectangle {
                    x: -1.0,
                    y: -1.0,
                    width: 12.0,
                    height: 12.0
                },
                Rectangle {
                    x: 19.0,
                    y: 19.0,
                    width: 12.0,
                    height: 12.0
                },
            ],
        );
    }

    #[test]
    fn added_quad_has_bounds_as_damage() {
        let bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };

        let previous = layer(bounds);

        let mut current = layer(bounds);
        current.quads.push((
            quad(Rectangle {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            }),
            background(),
        ));

        assert_eq!(
            Layer::damage(&previous, &current),
            vec![Rectangle {
                x: -1.0,
                y: -1.0,
                width: 12.0,
                height: 12.0
            }],
        );
    }

    #[test]
    fn shadow_change_expands_damage() {
        let bounds = Rectangle {
            x: -10.0,
            y: -10.0,
            width: 120.0,
            height: 120.0,
        };
        let quad_bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        };

        let mut previous = layer(bounds);
        previous.quads.push((quad(quad_bounds), background()));

        let mut current = layer(bounds);
        let mut shadowed = quad(quad_bounds);
        shadowed.shadow = Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            offset: Vector::new(5.0, 0.0),
            blur_radius: 2.0,
        };
        current.quads.push((shadowed, background()));

        assert_eq!(
            Layer::damage(&previous, &current),
            vec![
                Rectangle {
                    x: -1.0,
                    y: -1.0,
                    width: 12.0,
                    height: 12.0
                },
                Rectangle {
                    x: -8.0,
                    y: -8.0,
                    width: 26.0,
                    height: 26.0
                },
            ],
        );
    }

    #[test]
    fn changed_image_has_bounds_as_damage() {
        let bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };

        let image_bounds = Rectangle {
            x: 10.0,
            y: 10.0,
            width: 20.0,
            height: 20.0,
        };

        let make_image = |handle: u32| {
            Image::Raster {
                image: crate::core::image::Image::new(crate::core::image::Handle::from_bytes(
                    vec![handle as u8; 4],
                )),
                bounds: image_bounds,
                clip_bounds: bounds,
            }
        };

        let mut previous = layer(bounds);
        previous.images.push(make_image(0));

        let mut current = layer(bounds);
        current.images.push(make_image(1));

        assert_eq!(
            Layer::damage(&previous, &current),
            vec![
                Rectangle {
                    x: 9.0,
                    y: 9.0,
                    width: 22.0,
                    height: 22.0
                },
                Rectangle {
                    x: 9.0,
                    y: 9.0,
                    width: 22.0,
                    height: 22.0
                },
            ],
        );
    }

    #[cfg(feature = "geometry")]
    fn rect_path(x0: f64, y0: f64, x1: f64, y1: f64) -> vello_cpu::kurbo::BezPath {
        use vello_cpu::kurbo::Shape;

        vello_cpu::kurbo::Rect {
            x0, y0, x1, y1,
        }
        .to_path(0.0)
    }

    #[cfg(feature = "geometry")]
    fn fill_primitive() -> geometry::Primitive {
        geometry::Primitive::Fill {
            path: rect_path(0.0, 0.0, 100.0, 100.0),
            paint: vello_cpu::PaintType::Solid(vello_cpu::color::AlphaColor::new([
                0.0, 0.0, 0.0, 1.0,
            ])),
            rule: vello_cpu::peniko::Fill::NonZero,
        }
    }

    #[cfg(feature = "geometry")]
    #[test]
    fn cached_primitives_compare_by_pointer() {
        let bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };

        let cache: Arc<[geometry::Primitive]> = Arc::from(vec![fill_primitive()]);

        let with_cache = |cache: &Arc<[geometry::Primitive]>| Layer {
            bounds,
            primitives: vec![Item::Cached(
                Arc::clone(cache),
                bounds,
                Transformation::IDENTITY,
            )],
            ..Default::default()
        };

        // Same pointer, same bounds and transformation: no damage.
        assert!(Layer::damage(&with_cache(&cache), &with_cache(&cache)).is_empty());

        // Same content but a different pointer: damage.
        let other: Arc<[geometry::Primitive]> = Arc::from(vec![fill_primitive()]);

        assert!(!Layer::damage(&with_cache(&cache), &with_cache(&other)).is_empty());
    }
}
