#![allow(clippy::await_holding_refcell_ref, clippy::type_complexity)]

mod cache;

use crate::core::Element;
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::tree::{self, Tree};
use crate::core::widget::{self, Widget};
use crate::core::{self, Event, Length, Rectangle, Shell, Size, Vector};

use rustc_hash::FxHasher;
use std::hash::{Hash, Hasher as H};
#[cfg(feature = "lazy")]
use std::marker::PhantomData;

/// Creates a new [`Lazy`] widget with the given data `Dependency` and a
/// closure that can turn this data into a widget tree.
#[cfg(feature = "lazy")]
pub fn lazy<'a, Message, Theme, Renderer, Dependency, View>(
    dependency: Dependency,
    view: impl Fn(&Dependency) -> View + 'a,
) -> Lazy<'a, Message, Theme, Renderer, Dependency, View>
where
    Dependency: Hash + 'a,
    View: Into<Element<'static, Message, Theme, Renderer>>,
{
    Lazy::new(dependency, view)
}

/// A widget that only rebuilds its contents when necessary.
#[cfg(feature = "lazy")]
pub struct Lazy<'a, Message, Theme, Renderer, Dependency, View> {
    dependency: Dependency,
    view: Box<dyn Fn(&Dependency) -> View + 'a>,
    phantom: PhantomData<(Message, Theme, Renderer)>,
    size: Size<Length>,
}

impl<'a, Message, Theme, Renderer, Dependency, View>
    Lazy<'a, Message, Theme, Renderer, Dependency, View>
where
    Dependency: Hash + 'a,
    View: Into<Element<'static, Message, Theme, Renderer>>,
{
    /// Creates a new [`Lazy`] widget with the given data `Dependency` and a
    /// closure that can turn this data into a widget tree.
    pub fn new(dependency: Dependency, view: impl Fn(&Dependency) -> View + 'a) -> Self {
        Self {
            dependency,
            view: Box::new(view),
            phantom: PhantomData,
            size: Size::new(Length::Fit, Length::Fit),
        }
    }
}

struct Internal<Message, Theme, Renderer> {
    // TODO: store in widget impl instead?
    element: Element<'static, Message, Theme, Renderer>,
    hash: u64,
}

impl<'a, Message, Theme, Renderer, Dependency, View> Widget<Message, Theme, Renderer>
    for Lazy<'a, Message, Theme, Renderer, Dependency, View>
where
    View: Into<Element<'static, Message, Theme, Renderer>> + 'static,
    Dependency: Hash + 'a,
    Message: 'static,
    Theme: 'static,
    Renderer: core::Renderer + 'static,
{
    fn tag(&self) -> tree::Tag {
        struct Tag<T>(T);
        tree::Tag::of::<Tag<View>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Internal {
            element: (self.view)(&self.dependency).into(),
            hash: hash(&self.dependency),
        })
    }

    fn diff(&mut self, tree: &mut Tree) {
        let Tree {
            state, children, ..
        } = tree;

        let current = state.downcast_mut::<Internal<Message, Theme, Renderer>>();

        let new_hash = hash(&self.dependency);

        if current.hash != new_hash {
            current.hash = new_hash;
            current.element = (self.view)(&self.dependency).into();

            self.size = current.element.as_widget().size();
        }

        tree::diff_children(
            children,
            std::slice::from_mut(&mut current.element.as_widget_mut()),
        );
    }

    // TODO: Pass tree?
    fn size(&self) -> Size<Length> {
        self.size
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let cached = tree
            .state
            .downcast_mut::<Internal<Message, Theme, Renderer>>();

        cached
            .element
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let cached = tree
            .state
            .downcast_mut::<Internal<Message, Theme, Renderer>>();

        cached
            .element
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let cached = tree
            .state
            .downcast_mut::<Internal<Message, Theme, Renderer>>();

        cached.element.as_widget_mut().update(
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
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let cached = tree
            .state
            .downcast_ref::<Internal<Message, Theme, Renderer>>();

        cached.element.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let current = tree
            .state
            .downcast_ref::<Internal<Message, Theme, Renderer>>();

        current.element.as_widget().draw(
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
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let current = tree
            .state
            .downcast_mut::<Internal<Message, Theme, Renderer>>();

        current.element.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

fn hash(data: impl Hash) -> u64 {
    let mut hasher = FxHasher::default();
    data.hash(&mut hasher);
    hasher.finish()
}

impl<'a, Message, Theme, Renderer, Dependency, View>
    From<Lazy<'a, Message, Theme, Renderer, Dependency, View>>
    for Element<'a, Message, Theme, Renderer>
where
    View: Into<Element<'static, Message, Theme, Renderer>> + 'static,
    Renderer: core::Renderer + 'static,
    Message: 'static,
    Theme: 'static,
    Dependency: Hash + 'a,
{
    fn from(lazy: Lazy<'a, Message, Theme, Renderer, Dependency, View>) -> Self {
        Self::new(lazy)
    }
}
