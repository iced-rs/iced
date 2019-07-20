use std::hash::Hasher;
use stretch::result;

use crate::{Element, Event, Layout, MouseCursor, Point};

pub struct Interface<'a, Message, Renderer> {
    hash: u64,
    root: Element<'a, Message, Renderer>,
    layout: result::Layout,
}

pub struct Cache {
    hash: u64,
    layout: result::Layout,
}

impl<'a, Message, Renderer> Interface<'a, Message, Renderer> {
    pub fn compute(
        root: Element<'a, Message, Renderer>,
        renderer: &Renderer,
    ) -> Interface<'a, Message, Renderer> {
        let hasher = &mut crate::Hasher::default();
        root.hash(hasher);

        let hash = hasher.finish();
        let layout = root.compute_layout(renderer);

        Interface { hash, root, layout }
    }

    pub fn compute_with_cache(
        root: Element<'a, Message, Renderer>,
        renderer: &Renderer,
        cache: Cache,
    ) -> Interface<'a, Message, Renderer> {
        let hasher = &mut crate::Hasher::default();
        root.hash(hasher);

        let hash = hasher.finish();

        let layout = if hash == cache.hash {
            cache.layout
        } else {
            root.compute_layout(renderer)
        };

        Interface { hash, root, layout }
    }

    pub fn on_event(&mut self, event: Event, cursor_position: Point, messages: &mut Vec<Message>) {
        let Interface { root, layout, .. } = self;

        root.widget
            .on_event(event, Self::layout(layout), cursor_position, messages);
    }

    pub fn draw(&self, renderer: &mut Renderer, cursor_position: Point) -> MouseCursor {
        let Interface { root, layout, .. } = self;

        let cursor = root
            .widget
            .draw(renderer, Self::layout(layout), cursor_position);

        cursor
    }

    pub fn cache(self) -> Cache {
        Cache {
            hash: self.hash,
            layout: self.layout,
        }
    }

    fn layout(layout: &result::Layout) -> Layout<'_> {
        Layout::new(layout, Point::new(0.0, 0.0))
    }
}
