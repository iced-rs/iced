use crate::{input::mouse, Column, Element, Event, Layout, MouseCursor, Point};

use std::hash::Hasher;
use stretch::result;

pub struct Runtime {
    cache: Cache,
    events: Vec<Event>,
    cursor_position: Point,
}

impl Runtime {
    pub fn new() -> Runtime {
        // We use this as a placeholder to initialize the cache.
        // This way, we can avoid the overhead of using an `Option`
        // in `compute`.
        let root: Element<'_, (), ()> = Column::new().into();

        let hasher = &mut crate::Hasher::default();
        root.hash(hasher);

        Runtime {
            cache: Cache {
                hash: hasher.finish(),
                layout: root.compute_layout(&()),
            },
            events: Vec::new(),
            cursor_position: Point::new(0.0, 0.0),
        }
    }

    pub fn on_event(&mut self, event: Event) {
        match event {
            Event::Mouse(mouse::Event::CursorMoved { x, y }) => {
                self.cursor_position = Point::new(x, y);
            }
            _ => {}
        }

        self.events.push(event);
    }

    pub fn compute<'a, Message, Renderer>(
        &'a mut self,
        root: Element<'a, Message, Renderer>,
        renderer: &Renderer,
    ) -> Interface<'a, Message, Renderer> {
        let hasher = &mut crate::Hasher::default();
        root.hash(hasher);

        let hash = hasher.finish();

        if hash != self.cache.hash {
            self.cache = Cache {
                hash,
                layout: root.compute_layout(renderer),
            };
        }

        Interface {
            root,
            layout: &self.cache.layout,
            events: &mut self.events,
            cursor_position: self.cursor_position,
        }
    }
}

struct Cache {
    hash: u64,
    layout: result::Layout,
}

pub struct Interface<'a, Message, Renderer> {
    root: Element<'a, Message, Renderer>,
    layout: &'a result::Layout,
    events: &'a mut Vec<Event>,
    cursor_position: Point,
}

impl<'a, Message, Renderer> Interface<'a, Message, Renderer> {
    pub fn update(&mut self) -> Vec<Message> {
        let mut messages = Vec::new();

        for event in self.events.drain(..) {
            self.root.widget.on_event(
                event,
                Layout::new(&self.layout),
                self.cursor_position,
                &mut messages,
            );
        }

        messages
    }

    pub fn draw(
        &self,
        renderer: &mut Renderer,
        cursor_position: Point,
    ) -> MouseCursor {
        let cursor = self.root.widget.draw(
            renderer,
            Layout::new(self.layout),
            cursor_position,
        );

        cursor
    }
}
