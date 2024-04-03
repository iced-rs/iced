use crate::core::renderer;
use crate::core::{Background, Color, Point, Rectangle, Transformation};
use crate::graphics::color;
use crate::graphics::text::{Editor, Paragraph};
use crate::graphics::Mesh;
use crate::image::{self, Image};
use crate::quad::{self, Quad};
use crate::text::{self, Text};
use crate::triangle;

use std::cell::{self, RefCell};
use std::rc::Rc;

pub enum Layer<'a> {
    Live(&'a Live),
    Cached(cell::Ref<'a, Cached>),
}

pub enum LayerMut<'a> {
    Live(&'a mut Live),
    Cached(cell::RefMut<'a, Cached>),
}

pub struct Stack {
    live: Vec<Live>,
    cached: Vec<Rc<RefCell<Cached>>>,
    order: Vec<Kind>,
    transformations: Vec<Transformation>,
    previous: Vec<usize>,
    current: usize,
    live_count: usize,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            live: vec![Live::default()],
            cached: Vec::new(),
            order: vec![Kind::Live],
            transformations: vec![Transformation::IDENTITY],
            previous: Vec::new(),
            current: 0,
            live_count: 1,
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

        self.live[self.current].quads.add(quad, &background);
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

        self.live[self.current].text.push(paragraph);
    }

    pub fn draw_editor(
        &mut self,
        editor: &Editor,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        let paragraph = Text::Editor {
            editor: editor.downgrade(),
            position,
            color,
            clip_bounds,
            transformation: self.transformations.last().copied().unwrap(),
        };

        self.live[self.current].text.push(paragraph);
    }

    pub fn draw_text(
        &mut self,
        text: crate::core::Text,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        let transformation = self.transformation();

        let paragraph = Text::Cached {
            content: text.content,
            bounds: Rectangle::new(position, text.bounds) * transformation,
            color,
            size: text.size * transformation.scale_factor(),
            line_height: text.line_height,
            font: text.font,
            horizontal_alignment: text.horizontal_alignment,
            vertical_alignment: text.vertical_alignment,
            shaping: text.shaping,
            clip_bounds: clip_bounds * transformation,
        };

        self.live[self.current].text.push(paragraph);
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

        self.live[self.current].images.push(image);
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

        self.live[self.current].images.push(svg);
    }

    pub fn draw_mesh(&mut self, mut mesh: Mesh) {
        match &mut mesh {
            Mesh::Solid { transformation, .. }
            | Mesh::Gradient { transformation, .. } => {
                *transformation = *transformation * self.transformation();
            }
        }

        self.live[self.current].meshes.push(mesh);
    }

    pub fn draw_layer(&mut self, mut layer: Live) {
        layer.transformation = layer.transformation * self.transformation();

        if self.live_count == self.live.len() {
            self.live.push(layer);
        } else {
            self.live[self.live_count] = layer;
        }

        self.live_count += 1;
        self.order.push(Kind::Live);
    }

    pub fn draw_cached_layer(&mut self, layer: &Rc<RefCell<Cached>>) {
        {
            let mut layer = layer.borrow_mut();
            layer.transformation = self.transformation() * layer.transformation;
        }

        self.cached.push(layer.clone());
        self.order.push(Kind::Cache);
    }

    pub fn push_clip(&mut self, bounds: Option<Rectangle>) {
        self.previous.push(self.current);
        self.order.push(Kind::Live);

        self.current = self.live_count;
        self.live_count += 1;

        let bounds = bounds.map(|bounds| bounds * self.transformation());

        if self.current == self.live.len() {
            self.live.push(Live {
                bounds,
                ..Live::default()
            });
        } else {
            self.live[self.current].bounds = bounds;
        }
    }

    pub fn pop_clip(&mut self) {
        self.current = self.previous.pop().unwrap();
    }

    pub fn push_transformation(&mut self, transformation: Transformation) {
        self.transformations
            .push(self.transformation() * transformation);
    }

    pub fn pop_transformation(&mut self) {
        let _ = self.transformations.pop();
    }

    fn transformation(&self) -> Transformation {
        self.transformations.last().copied().unwrap()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = LayerMut<'_>> {
        let mut live = self.live.iter_mut();
        let mut cached = self.cached.iter_mut();

        self.order.iter().map(move |kind| match kind {
            Kind::Live => LayerMut::Live(live.next().unwrap()),
            Kind::Cache => {
                LayerMut::Cached(cached.next().unwrap().borrow_mut())
            }
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = Layer<'_>> {
        let mut live = self.live.iter();
        let mut cached = self.cached.iter();

        self.order.iter().map(move |kind| match kind {
            Kind::Live => Layer::Live(live.next().unwrap()),
            Kind::Cache => Layer::Cached(cached.next().unwrap().borrow()),
        })
    }

    pub fn clear(&mut self) {
        for live in &mut self.live[..self.live_count] {
            live.bounds = None;
            live.transformation = Transformation::IDENTITY;

            live.quads.clear();
            live.meshes.clear();
            live.text.clear();
            live.images.clear();
        }

        self.current = 0;
        self.live_count = 1;

        self.order.clear();
        self.order.push(Kind::Live);

        self.cached.clear();
        self.previous.clear();
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
pub struct Live {
    pub bounds: Option<Rectangle>,
    pub transformation: Transformation,
    pub quads: quad::Batch,
    pub meshes: triangle::Batch,
    pub text: text::Batch,
    pub images: image::Batch,
}

impl Live {
    pub fn into_cached(self) -> Cached {
        Cached {
            bounds: self.bounds,
            transformation: self.transformation,
            last_transformation: None,
            quads: quad::Cache::Staged(self.quads),
            meshes: triangle::Cache::Staged(self.meshes),
            text: text::Cache::Staged(self.text),
            images: self.images,
        }
    }
}

#[derive(Default)]
pub struct Cached {
    pub bounds: Option<Rectangle>,
    pub transformation: Transformation,
    pub last_transformation: Option<Transformation>,
    pub quads: quad::Cache,
    pub meshes: triangle::Cache,
    pub text: text::Cache,
    pub images: image::Batch,
}

impl Cached {
    pub fn update(&mut self, live: Live) {
        self.bounds = live.bounds;
        self.transformation = live.transformation;

        self.quads.update(live.quads);
        self.meshes.update(live.meshes);
        self.text.update(live.text);
        self.images = live.images;
    }
}

enum Kind {
    Live,
    Cache,
}
