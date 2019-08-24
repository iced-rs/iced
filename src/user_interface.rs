use crate::{input::mouse, Column, Element, Event, Layout, MouseCursor, Point};

use std::hash::Hasher;
use stretch::result;

pub struct UserInterface<'a, Message, Renderer> {
    hash: u64,
    root: Element<'a, Message, Renderer>,
    layout: result::Layout,
    cursor_position: Point,
}

impl<'a, Message, Renderer> UserInterface<'a, Message, Renderer> {
    pub fn build(
        root: Element<'a, Message, Renderer>,
        renderer: &Renderer,
        cache: Cache,
    ) -> Self {
        let hasher = &mut crate::Hasher::default();
        root.hash(hasher);

        let hash = hasher.finish();

        let layout = if hash == cache.hash {
            cache.layout
        } else {
            root.compute_layout(renderer)
        };

        UserInterface {
            hash,
            root,
            layout,
            cursor_position: cache.cursor_position,
        }
    }

    pub fn update(
        &mut self,
        events: impl Iterator<Item = Event>,
    ) -> Vec<Message> {
        let mut messages = Vec::new();

        for event in events {
            match event {
                Event::Mouse(mouse::Event::CursorMoved { x, y }) => {
                    self.cursor_position = Point::new(x, y);
                }
                _ => {}
            }

            self.root.widget.on_event(
                event,
                Layout::new(&self.layout),
                self.cursor_position,
                &mut messages,
            );
        }

        messages
    }

    pub fn draw(&self, renderer: &mut Renderer) -> MouseCursor {
        let cursor = self.root.widget.draw(
            renderer,
            Layout::new(&self.layout),
            self.cursor_position,
        );

        cursor
    }

    pub fn into_cache(self) -> Cache {
        Cache {
            hash: self.hash,
            layout: self.layout,
            cursor_position: self.cursor_position,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cache {
    hash: u64,
    layout: result::Layout,
    cursor_position: Point,
}

impl Cache {
    pub fn new() -> Cache {
        let root: Element<'_, (), ()> = Column::new().into();

        let hasher = &mut crate::Hasher::default();
        root.hash(hasher);

        Cache {
            hash: hasher.finish(),
            layout: root.compute_layout(&()),
            cursor_position: Point::new(0.0, 0.0),
        }
    }
}

impl Default for Cache {
    fn default() -> Cache {
        Cache::new()
    }
}

impl PartialEq for Cache {
    fn eq(&self, other: &Cache) -> bool {
        self.hash == other.hash && self.cursor_position == other.cursor_position
    }
}

impl Eq for Cache {}
