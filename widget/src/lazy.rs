#![allow(clippy::await_holding_refcell_ref, clippy::type_complexity)]
pub(crate) mod helpers;

pub mod component;
pub mod responsive;

pub use component::Component;
pub use responsive::Responsive;

mod cache;

use crate::core::event::{self, Event};
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::tree::{self, Tree};
use crate::core::widget::{self, Widget};
use crate::core::Element;
use crate::core::{
    self, Clipboard, Hasher, Length, Point, Rectangle, Shell, Size,
};
use crate::runtime::overlay::Nested;

use ouroboros::self_referencing;
use std::cell::RefCell;
use std::hash::{Hash, Hasher as H};
use std::rc::Rc;

/// A widget that only rebuilds its contents when necessary.
#[allow(missing_debug_implementations)]
pub struct Lazy<'a, Message, Renderer, Dependency, View> {
    dependency: Dependency,
    view: Box<dyn Fn(&Dependency) -> View + 'a>,
    element: RefCell<
        Option<Rc<RefCell<Option<Element<'static, Message, Renderer>>>>>,
    >,
}

impl<'a, Message, Renderer, Dependency, View>
    Lazy<'a, Message, Renderer, Dependency, View>
where
    Dependency: Hash + 'a,
    View: Into<Element<'static, Message, Renderer>>,
{
    /// Creates a new [`Lazy`] widget with the given data `Dependency` and a
    /// closure that can turn this data into a widget tree.
    pub fn new(
        dependency: Dependency,
        view: impl Fn(&Dependency) -> View + 'a,
    ) -> Self {
        Self {
            dependency,
            view: Box::new(view),
            element: RefCell::new(None),
        }
    }

    fn with_element<T>(
        &self,
        f: impl FnOnce(&Element<'_, Message, Renderer>) -> T,
    ) -> T {
        f(self
            .element
            .borrow()
            .as_ref()
            .unwrap()
            .borrow()
            .as_ref()
            .unwrap())
    }

    fn with_element_mut<T>(
        &self,
        f: impl FnOnce(&mut Element<'_, Message, Renderer>) -> T,
    ) -> T {
        f(self
            .element
            .borrow()
            .as_ref()
            .unwrap()
            .borrow_mut()
            .as_mut()
            .unwrap())
    }
}

struct Internal<Message, Renderer> {
    element: Rc<RefCell<Option<Element<'static, Message, Renderer>>>>,
    hash: u64,
}

impl<'a, Message, Renderer, Dependency, View> Widget<Message, Renderer>
    for Lazy<'a, Message, Renderer, Dependency, View>
where
    View: Into<Element<'static, Message, Renderer>> + 'static,
    Dependency: Hash + 'a,
    Message: 'static,
    Renderer: core::Renderer + 'static,
{
    fn tag(&self) -> tree::Tag {
        struct Tag<T>(T);
        tree::Tag::of::<Tag<View>>()
    }

    fn state(&self) -> tree::State {
        let mut hasher = Hasher::default();
        self.dependency.hash(&mut hasher);
        let hash = hasher.finish();

        let element =
            Rc::new(RefCell::new(Some((self.view)(&self.dependency).into())));

        (*self.element.borrow_mut()) = Some(element.clone());

        tree::State::new(Internal { element, hash })
    }

    fn children(&self) -> Vec<Tree> {
        self.with_element(|element| vec![Tree::new(element.as_widget())])
    }

    fn diff(&self, tree: &mut Tree) {
        let current = tree.state.downcast_mut::<Internal<Message, Renderer>>();

        let mut hasher = Hasher::default();
        self.dependency.hash(&mut hasher);
        let new_hash = hasher.finish();

        if current.hash != new_hash {
            current.hash = new_hash;

            let element = (self.view)(&self.dependency).into();
            current.element = Rc::new(RefCell::new(Some(element)));

            (*self.element.borrow_mut()) = Some(current.element.clone());
            self.with_element(|element| {
                tree.diff_children(std::slice::from_ref(&element.as_widget()))
            });
        } else {
            (*self.element.borrow_mut()) = Some(current.element.clone());
        }
    }

    fn width(&self) -> Length {
        self.with_element(|element| element.as_widget().width())
    }

    fn height(&self) -> Length {
        self.with_element(|element| element.as_widget().height())
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.with_element(|element| {
            element.as_widget().layout(renderer, limits)
        })
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        self.with_element(|element| {
            element.as_widget().operate(
                &mut tree.children[0],
                layout,
                renderer,
                operation,
            );
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        self.with_element_mut(|element| {
            element.as_widget_mut().on_event(
                &mut tree.children[0],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            )
        })
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.with_element(|element| {
            element.as_widget().mouse_interaction(
                &tree.children[0],
                layout,
                cursor,
                viewport,
                renderer,
            )
        })
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.with_element(|element| {
            element.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            )
        })
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        let overlay = Overlay(Some(
            InnerBuilder {
                cell: self.element.borrow().as_ref().unwrap().clone(),
                element: self
                    .element
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .borrow_mut()
                    .take()
                    .unwrap(),
                tree: &mut tree.children[0],
                overlay_builder: |element, tree| {
                    element
                        .as_widget_mut()
                        .overlay(tree, layout, renderer)
                        .map(|overlay| RefCell::new(Nested::new(overlay)))
                },
            }
            .build(),
        ));

        let has_overlay =
            overlay.with_overlay_maybe(|overlay| overlay.position());

        has_overlay
            .map(|position| overlay::Element::new(position, Box::new(overlay)))
    }
}

#[self_referencing]
struct Inner<'a, Message: 'a, Renderer: 'a> {
    cell: Rc<RefCell<Option<Element<'static, Message, Renderer>>>>,
    element: Element<'static, Message, Renderer>,
    tree: &'a mut Tree,

    #[borrows(mut element, mut tree)]
    #[not_covariant]
    overlay: Option<RefCell<Nested<'this, Message, Renderer>>>,
}

struct Overlay<'a, Message, Renderer>(Option<Inner<'a, Message, Renderer>>);

impl<'a, Message, Renderer> Drop for Overlay<'a, Message, Renderer> {
    fn drop(&mut self) {
        let heads = self.0.take().unwrap().into_heads();
        (*heads.cell.borrow_mut()) = Some(heads.element);
    }
}

impl<'a, Message, Renderer> Overlay<'a, Message, Renderer> {
    fn with_overlay_maybe<T>(
        &self,
        f: impl FnOnce(&mut Nested<'_, Message, Renderer>) -> T,
    ) -> Option<T> {
        self.0.as_ref().unwrap().with_overlay(|overlay| {
            overlay.as_ref().map(|nested| (f)(&mut nested.borrow_mut()))
        })
    }

    fn with_overlay_mut_maybe<T>(
        &mut self,
        f: impl FnOnce(&mut Nested<'_, Message, Renderer>) -> T,
    ) -> Option<T> {
        self.0.as_mut().unwrap().with_overlay_mut(|overlay| {
            overlay.as_mut().map(|nested| (f)(nested.get_mut()))
        })
    }
}

impl<'a, Message, Renderer> overlay::Overlay<Message, Renderer>
    for Overlay<'a, Message, Renderer>
where
    Renderer: core::Renderer,
{
    fn layout(
        &self,
        renderer: &Renderer,
        bounds: Size,
        position: Point,
    ) -> layout::Node {
        self.with_overlay_maybe(|overlay| {
            overlay.layout(renderer, bounds, position)
        })
        .unwrap_or_default()
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let _ = self.with_overlay_maybe(|overlay| {
            overlay.draw(renderer, theme, style, layout, cursor);
        });
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.with_overlay_maybe(|overlay| {
            overlay.mouse_interaction(layout, cursor, viewport, renderer)
        })
        .unwrap_or_default()
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.with_overlay_mut_maybe(|overlay| {
            overlay.on_event(event, layout, cursor, renderer, clipboard, shell)
        })
        .unwrap_or(event::Status::Ignored)
    }

    fn is_over(
        &self,
        layout: Layout<'_>,
        renderer: &Renderer,
        cursor_position: Point,
    ) -> bool {
        self.with_overlay_maybe(|overlay| {
            overlay.is_over(layout, renderer, cursor_position)
        })
        .unwrap_or_default()
    }
}

impl<'a, Message, Renderer, Dependency, View>
    From<Lazy<'a, Message, Renderer, Dependency, View>>
    for Element<'a, Message, Renderer>
where
    View: Into<Element<'static, Message, Renderer>> + 'static,
    Renderer: core::Renderer + 'static,
    Message: 'static,
    Dependency: Hash + 'a,
{
    fn from(lazy: Lazy<'a, Message, Renderer, Dependency, View>) -> Self {
        Self::new(lazy)
    }
}
