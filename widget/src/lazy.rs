//! A widget that only rebuilds its contents when necessary.
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::tree::{self, Tree};
use crate::core::widget::{self, Widget};
use crate::core::{self, Event, Length, Rectangle, Shell, Size, Vector};

use rustc_hash::FxHasher;
use std::hash::{Hash, Hasher};

/// A widget that only rebuilds its contents when necessary.
pub struct Lazy<'a, W, Dependency> {
    dependency: Dependency,
    view: Box<dyn Fn(&Dependency) -> W + 'a>,
    size: Size<Length>,
}

impl<'a, W, Dependency> Lazy<'a, W, Dependency>
where
    Dependency: Hash + 'a,
{
    /// Creates a new [`Lazy`] widget with the given data `Dependency` and a
    /// closure that can turn this data into a widget tree.
    pub fn new(dependency: Dependency, view: impl Fn(&Dependency) -> W + 'a) -> Self {
        Self {
            dependency,
            view: Box::new(view),
            size: Size::new(Length::Fit, Length::Fit),
        }
    }
}

struct Internal<W> {
    element: W,
    hash: u64,
}

impl<W, Dependency> widget::Node for Lazy<'_, W, Dependency> {}

impl<'a, W, Message, Theme, Renderer, Dependency> Widget<Message, Theme, Renderer>
    for Lazy<'a, W, Dependency>
where
    W: Widget<Message, Theme, Renderer> + 'static,
    Dependency: Hash + 'a,
    Renderer: core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        struct Tag<T>(T);
        tree::Tag::of::<Tag<W>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Internal {
            element: (self.view)(&self.dependency),
            hash: hash(&self.dependency),
        })
    }

    fn diff(&mut self, tree: &mut Tree) {
        let current = tree.state.downcast_mut::<Internal<W>>();

        let new_hash = hash(&self.dependency);

        if current.hash != new_hash {
            current.hash = new_hash;
            current.element = (self.view)(&self.dependency);
        }

        // The widget value is recreated every frame, so the size hint must be
        // re-derived from the cached element on every diff
        self.size = current.element.size();

        tree::diff_children(
            &mut tree.children,
            std::slice::from_mut(&mut current.element),
        );
    }

    fn size(&self) -> Size<Length> {
        self.size
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) {
        let cached = tree.state.downcast_mut::<Internal<W>>();

        cached
            .element
            .layout(&mut tree.children[0], renderer, limits);

        tree.size = tree.children[0].size;
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout,
        viewport: &Rectangle,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let cached = tree.state.downcast_mut::<Internal<W>>();

        cached
            .element
            .operate(&mut tree.children[0], layout, viewport, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let cached = tree.state.downcast_mut::<Internal<W>>();

        cached.element.update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let cached = tree.state.downcast_ref::<Internal<W>>();

        cached
            .element
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let current = tree.state.downcast_ref::<Internal<W>>();

        current.element.draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
        window: Size,
    ) -> Vec<overlay::Element<'b, Message, Theme, Renderer>> {
        let current = tree.state.downcast_mut::<Internal<W>>();

        current.element.overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
            window,
        )
    }
}

fn hash(data: impl Hash) -> u64 {
    let mut hasher = FxHasher::default();
    data.hash(&mut hasher);
    hasher.finish()
}
