use crate::Primitive;
use crate::core::renderer::Quad;
use crate::core::{
    self, Background, Color, Point, Rectangle, Svg, Transformation,
};
use crate::graphics::damage;
use crate::graphics::layer;
use crate::graphics::text::{Editor, Paragraph, Text};
use crate::graphics::{self, Image};

use std::sync::Arc;

pub type Stack = layer::Stack<Layer>;

#[derive(Debug, Clone)]
pub struct Layer {
    pub bounds: Rectangle,
    pub quads: Vec<(Quad, Background)>,
    pub primitives: Vec<Item<Primitive>>,
    pub text: Vec<Item<Text>>,
    pub images: Vec<Image>,
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
            line_height: text.line_height.to_absolute(text.size)
                * transformation.scale_factor(),
            font: text.font,
            align_x: text.align_x,
            align_y: text.align_y,
            shaping: text.shaping,
            clip_bounds: clip_bounds * transformation,
        };

        self.text.push(Item::Live(text));
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
            Image::Raster(raster, bounds) => {
                self.draw_raster(raster, bounds, transformation);
            }
            Image::Vector(svg, bounds) => {
                self.draw_svg(svg, bounds, transformation);
            }
        }
    }

    pub fn draw_raster(
        &mut self,
        image: core::Image,
        bounds: Rectangle,
        transformation: Transformation,
    ) {
        let image = Image::Raster(image, bounds * transformation);

        self.images.push(image);
    }

    pub fn draw_svg(
        &mut self,
        svg: Svg,
        bounds: Rectangle,
        transformation: Transformation,
    ) {
        let svg = Image::Vector(svg, bounds * transformation);

        self.images.push(svg);
    }

    pub fn draw_primitive_group(
        &mut self,
        primitives: Vec<Primitive>,
        clip_bounds: Rectangle,
        transformation: Transformation,
    ) {
        self.primitives.push(Item::Group(
            primitives,
            clip_bounds,
            transformation,
        ));
    }

    pub fn draw_primitive_cache(
        &mut self,
        primitives: Arc<[Primitive]>,
        clip_bounds: Rectangle,
        transformation: Transformation,
    ) {
        self.primitives.push(Item::Cached(
            primitives,
            clip_bounds,
            transformation,
        ));
    }

    pub fn damage(previous: &Self, current: &Self) -> Vec<Rectangle> {
        if previous.bounds != current.bounds {
            return vec![previous.bounds, current.bounds];
        }

        let mut damage = damage::list(
            &previous.quads,
            &current.quads,
            |(quad, _)| {
                quad.bounds
                    .expand(1.0)
                    .intersection(&current.bounds)
                    .into_iter()
                    .collect()
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

        let primitives = damage::list(
            &previous.primitives,
            &current.primitives,
            |item| match item {
                Item::Live(primitive) => vec![primitive.visible_bounds()],
                Item::Group(primitives, group_bounds, transformation) => {
                    primitives
                        .as_slice()
                        .iter()
                        .map(Primitive::visible_bounds)
                        .map(|bounds| bounds * *transformation)
                        .filter_map(|bounds| bounds.intersection(group_bounds))
                        .collect()
                }
                Item::Cached(_, bounds, _) => {
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

    fn flush(&mut self) {}

    fn resize(&mut self, bounds: Rectangle) {
        self.bounds = bounds;
    }

    fn reset(&mut self) {
        self.bounds = Rectangle::INFINITE;

        self.quads.clear();
        self.primitives.clear();
        self.text.clear();
        self.images.clear();
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
            Item::Group(_, _, transformation)
            | Item::Cached(_, _, transformation) => *transformation,
        }
    }

    pub fn clip_bounds(&self) -> Rectangle {
        match self {
            Item::Live(_) => Rectangle::INFINITE,
            Item::Group(_, clip_bounds, _)
            | Item::Cached(_, clip_bounds, _) => *clip_bounds,
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
