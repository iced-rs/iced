#![allow(clippy::await_holding_refcell_ref, clippy::type_complexity)]
pub(crate) mod helpers;

pub mod component;
pub mod responsive;

#[allow(deprecated)]
pub use component::Component;
pub use responsive::Responsive;

mod cache;

use crate::core::Element;
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::tree::{self, Tree};
use crate::core::widget::{self, Widget};
use crate::core::{
    self, Clipboard, Event, Length, Rectangle, Shell, Size, Vector,
};
use crate::runtime::overlay::Nested;

use ouroboros::self_referencing;
use rustc_hash::FxHasher;
use std::cell::RefCell;
use std::hash::{Hash, Hasher as H};
use std::rc::Rc;

/// A widget that only rebuilds its contents when necessary.
#[cfg(feature = "lazy")]
#[allow(missing_debug_implementations)]
pub struct Lazy<'a, Message, Theme, Renderer, Dependency, View> {
    dependency: Dependency,
    view: Box<dyn Fn(&Dependency) -> View + 'a>,
    element: RefCell<
        Option<Rc<RefCell<Option<Element<'static, Message, Theme, Renderer>>>>>,
    >,
}

impl<'a, Message, Theme, Renderer, Dependency, View>
    Lazy<'a, Message, Theme, Renderer, Dependency, View>
where
    Dependency: Hash + 'a,
    View: Into<Element<'static, Message, Theme, Renderer>>,
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
        f: impl FnOnce(&Element<'_, Message, Theme, Renderer>) -> T,
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
        f: impl FnOnce(&mut Element<'_, Message, Theme, Renderer>) -> T,
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

struct Internal<Message, Theme, Renderer> {
    element: Rc<RefCell<Option<Element<'static, Message, Theme, Renderer>>>>,
    hash: u64,
}

impl<'a, Message, Theme, Renderer, Dependency, View>
    Widget<Message, Theme, Renderer>
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
        let hash = {
            let mut hasher = FxHasher::default();
            self.dependency.hash(&mut hasher);

            hasher.finish()
        };

        let element =
            Rc::new(RefCell::new(Some((self.view)(&self.dependency).into())));

        (*self.element.borrow_mut()) = Some(element.clone());

        tree::State::new(Internal { element, hash })
    }

    fn children(&self) -> Vec<Tree> {
        self.with_element(|element| vec![Tree::new(element.as_widget())])
    }

    fn diff(&self, tree: &mut Tree) {
        let current = tree
            .state
            .downcast_mut::<Internal<Message, Theme, Renderer>>();

        let new_hash = {
            let mut hasher = FxHasher::default();
            self.dependency.hash(&mut hasher);

            hasher.finish()
        };

        if current.hash != new_hash {
            current.hash = new_hash;

            let element = (self.view)(&self.dependency).into();
            current.element = Rc::new(RefCell::new(Some(element)));

            (*self.element.borrow_mut()) = Some(current.element.clone());
            self.with_element(|element| {
                tree.diff_children(std::slice::from_ref(&element.as_widget()));
            });
        } else {
            (*self.element.borrow_mut()) = Some(current.element.clone());
        }
    }

    fn size(&self) -> Size<Length> {
        self.with_element(|element| element.as_widget().size())
    }

    fn size_hint(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.with_element(|element| {
            element
                .as_widget()
                .layout(&mut tree.children[0], renderer, limits)
        })
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
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

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.with_element_mut(|element| {
            element.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        });
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
        theme: &Theme,
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
            );
        });
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let overlay = InnerBuilder {
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
            layout,
            overlay_builder: |element, tree, layout| {
                element
                    .as_widget_mut()
                    .overlay(tree, *layout, renderer, viewport, translation)
                    .map(|overlay| RefCell::new(Nested::new(overlay)))
            },
        }
        .build();

        #[allow(clippy::redundant_closure_for_method_calls)]
        if overlay.with_overlay(|overlay| overlay.is_some()) {
            Some(overlay::Element::new(Box::new(Overlay(Some(overlay)))))
        } else {
            let heads = overlay.into_heads();

            // - You may not like it, but this is what peak performance looks like
            // - TODO: Get rid of ouroboros, for good
            // - What?!
            *self.element.borrow().as_ref().unwrap().borrow_mut() =
                Some(heads.element);

            None
        }
    }
}

#[self_referencing]
struct Inner<'a, Message: 'a, Theme: 'a, Renderer: 'a> {
    cell: Rc<RefCell<Option<Element<'static, Message, Theme, Renderer>>>>,
    element: Element<'static, Message, Theme, Renderer>,
    tree: &'a mut Tree,
    layout: Layout<'a>,

    #[borrows(mut element, mut tree, layout)]
    #[not_covariant]
    overlay: Option<RefCell<Nested<'this, Message, Theme, Renderer>>>,
}

struct Overlay<'a, Message, Theme, Renderer>(
    Option<Inner<'a, Message, Theme, Renderer>>,
);

impl<Message, Theme, Renderer> Drop for Overlay<'_, Message, Theme, Renderer> {
    fn drop(&mut self) {
        let heads = self.0.take().unwrap().into_heads();
        (*heads.cell.borrow_mut()) = Some(heads.element);
    }
}

impl<Message, Theme, Renderer> Overlay<'_, Message, Theme, Renderer> {
    fn with_overlay_maybe<T>(
        &self,
        f: impl FnOnce(&mut Nested<'_, Message, Theme, Renderer>) -> T,
    ) -> Option<T> {
        self.0.as_ref().unwrap().with_overlay(|overlay| {
            overlay.as_ref().map(|nested| (f)(&mut nested.borrow_mut()))
        })
    }

    fn with_overlay_mut_maybe<T>(
        &mut self,
        f: impl FnOnce(&mut Nested<'_, Message, Theme, Renderer>) -> T,
    ) -> Option<T> {
        self.0.as_mut().unwrap().with_overlay_mut(|overlay| {
            overlay.as_mut().map(|nested| (f)(nested.get_mut()))
        })
    }
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for Overlay<'_, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        self.with_overlay_maybe(|overlay| overlay.layout(renderer, bounds))
            .unwrap_or_default()
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
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
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.with_overlay_maybe(|overlay| {
            overlay.mouse_interaction(layout, cursor, renderer)
        })
        .unwrap_or_default()
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let _ = self.with_overlay_mut_maybe(|overlay| {
            overlay.update(event, layout, cursor, renderer, clipboard, shell);
        });
    }
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
    fn from(
        lazy: Lazy<'a, Message, Theme, Renderer, Dependency, View>,
    ) -> Self {
        Self::new(lazy)
    }
}
