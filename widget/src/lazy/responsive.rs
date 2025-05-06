use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget;
use crate::core::widget::tree::{self, Tree};
use crate::core::{
    self, Clipboard, Element, Event, Length, Point, Rectangle, Shell, Size,
    Vector, Widget,
};
use crate::horizontal_space;
use crate::runtime::overlay::Nested;

use ouroboros::self_referencing;
use std::cell::{RefCell, RefMut};
use std::marker::PhantomData;
use std::ops::Deref;

/// A widget that is aware of its dimensions.
///
/// A [`Responsive`] widget will always try to fill all the available space of
/// its parent.
#[cfg(feature = "lazy")]
#[allow(missing_debug_implementations)]
pub struct Responsive<
    'a,
    Message,
    Theme = crate::Theme,
    Renderer = crate::Renderer,
> {
    view: Box<dyn Fn(Size) -> Element<'a, Message, Theme, Renderer> + 'a>,
    content: RefCell<Content<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> Responsive<'a, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    /// Creates a new [`Responsive`] widget with a closure that produces its
    /// contents.
    ///
    /// The `view` closure will be provided with the current [`Size`] of
    /// the [`Responsive`] widget and, therefore, can be used to build the
    /// contents of the widget in a responsive way.
    pub fn new(
        view: impl Fn(Size) -> Element<'a, Message, Theme, Renderer> + 'a,
    ) -> Self {
        Self {
            view: Box::new(view),
            content: RefCell::new(Content {
                size: Size::ZERO,
                layout: None,
                is_layout_invalid: true,
                element: Element::new(horizontal_space().width(0)),
            }),
        }
    }
}

struct Content<'a, Message, Theme, Renderer> {
    size: Size,
    layout: Option<layout::Node>,
    is_layout_invalid: bool,
    element: Element<'a, Message, Theme, Renderer>,
}

impl<'a, Message, Theme, Renderer> Content<'a, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer) {
        if self.layout.is_none() || self.is_layout_invalid {
            self.layout = Some(self.element.as_widget().layout(
                tree,
                renderer,
                &layout::Limits::new(Size::ZERO, self.size),
            ));
            self.is_layout_invalid = false;
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        new_size: Size,
        view: &dyn Fn(Size) -> Element<'a, Message, Theme, Renderer>,
    ) {
        if self.size != new_size {
            self.element = view(new_size);
            self.size = new_size;
            self.layout = None;

            tree.diff(&self.element);
        } else {
            let is_tree_empty =
                tree.tag == tree::Tag::stateless() && tree.children.is_empty();

            if is_tree_empty {
                self.layout = None;
                tree.diff(&self.element);
            }
        }
    }

    fn resolve<R, T>(
        &mut self,
        tree: &mut Tree,
        renderer: R,
        layout: Layout<'_>,
        view: &dyn Fn(Size) -> Element<'a, Message, Theme, Renderer>,
        f: impl FnOnce(
            &mut Tree,
            R,
            Layout<'_>,
            &mut Element<'a, Message, Theme, Renderer>,
        ) -> T,
    ) -> T
    where
        R: Deref<Target = Renderer>,
    {
        self.update(tree, layout.bounds().size(), view);
        self.layout(tree, renderer.deref());

        let content_layout = Layout::with_offset(
            layout.position() - Point::ORIGIN,
            self.layout.as_ref().unwrap(),
        );

        f(tree, renderer, content_layout, &mut self.element)
    }
}

struct State {
    tree: RefCell<Tree>,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Responsive<'_, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            tree: RefCell::new(Tree::empty()),
        })
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.max())
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let mut content = self.content.borrow_mut();

        content.resolve(
            &mut state.tree.borrow_mut(),
            renderer,
            layout,
            &self.view,
            |tree, renderer, layout, element| {
                element
                    .as_widget()
                    .operate(tree, layout, renderer, operation);
            },
        );
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
        let state = tree.state.downcast_mut::<State>();
        let mut content = self.content.borrow_mut();

        let mut local_messages = vec![];
        let mut local_shell = Shell::new(&mut local_messages);

        content.resolve(
            &mut state.tree.borrow_mut(),
            renderer,
            layout,
            &self.view,
            |tree, renderer, layout, element| {
                element.as_widget_mut().update(
                    tree,
                    event,
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    &mut local_shell,
                    viewport,
                );
            },
        );

        if local_shell.is_layout_invalid() {
            content.layout = None;
        }

        shell.merge(local_shell, std::convert::identity);
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
        let state = tree.state.downcast_ref::<State>();
        let mut content = self.content.borrow_mut();

        content.resolve(
            &mut state.tree.borrow_mut(),
            renderer,
            layout,
            &self.view,
            |tree, renderer, layout, element| {
                element.as_widget().draw(
                    tree, renderer, theme, style, layout, cursor, viewport,
                );
            },
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
        let state = tree.state.downcast_ref::<State>();
        let mut content = self.content.borrow_mut();

        content.resolve(
            &mut state.tree.borrow_mut(),
            renderer,
            layout,
            &self.view,
            |tree, renderer, layout, element| {
                element
                    .as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
            },
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        use std::ops::DerefMut;

        let state = tree.state.downcast_ref::<State>();

        let overlay = OverlayBuilder {
            content: self.content.borrow_mut(),
            tree: state.tree.borrow_mut(),
            types: PhantomData,
            overlay_builder: |content: &mut RefMut<
                '_,
                Content<'_, _, _, _>,
            >,
                              tree| {
                content.update(tree, layout.bounds().size(), &self.view);
                content.layout(tree, renderer);

                let Content {
                    element,
                    layout: content_layout_node,
                    is_layout_invalid,
                    ..
                } = content.deref_mut();

                let content_layout = Layout::with_offset(
                    layout.bounds().position() - Point::ORIGIN,
                    content_layout_node.as_ref().unwrap(),
                );

                (
                    element
                        .as_widget_mut()
                        .overlay(
                            tree,
                            content_layout,
                            renderer,
                            viewport,
                            translation,
                        )
                        .map(|overlay| RefCell::new(Nested::new(overlay))),
                    is_layout_invalid,
                )
            },
        }
        .build();

        if overlay.with_overlay(|(overlay, _layout)| overlay.is_some()) {
            Some(overlay::Element::new(Box::new(overlay)))
        } else {
            None
        }
    }
}

impl<'a, Message, Theme, Renderer>
    From<Responsive<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: core::Renderer + 'a,
{
    fn from(responsive: Responsive<'a, Message, Theme, Renderer>) -> Self {
        Self::new(responsive)
    }
}

#[self_referencing]
struct Overlay<'a, 'b, Message, Theme, Renderer> {
    content: RefMut<'a, Content<'b, Message, Theme, Renderer>>,
    tree: RefMut<'a, Tree>,
    types: PhantomData<Message>,

    #[borrows(mut content, mut tree)]
    #[not_covariant]
    overlay: (
        Option<RefCell<Nested<'this, Message, Theme, Renderer>>>,
        &'this mut bool,
    ),
}

impl<Message, Theme, Renderer> Overlay<'_, '_, Message, Theme, Renderer> {
    fn with_overlay_maybe<T>(
        &self,
        f: impl FnOnce(&mut Nested<'_, Message, Theme, Renderer>) -> T,
    ) -> Option<T> {
        self.with_overlay(|(overlay, _layout)| {
            overlay.as_ref().map(|nested| (f)(&mut nested.borrow_mut()))
        })
    }

    fn with_overlay_mut_maybe<T>(
        &mut self,
        f: impl FnOnce(&mut Nested<'_, Message, Theme, Renderer>) -> T,
    ) -> Option<T> {
        self.with_overlay_mut(|(overlay, _layout)| {
            overlay.as_mut().map(|nested| (f)(nested.get_mut()))
        })
    }
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for Overlay<'_, '_, Message, Theme, Renderer>
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
        let mut is_layout_invalid = false;

        let _ = self.with_overlay_mut_maybe(|overlay| {
            overlay.update(event, layout, cursor, renderer, clipboard, shell);

            is_layout_invalid = shell.is_layout_invalid();
        });

        if is_layout_invalid {
            self.with_overlay_mut(|(_overlay, layout)| {
                **layout = true;
            });
        }
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let _ = self.with_overlay_mut_maybe(|overlay| {
            overlay.operate(layout, renderer, operation);
        });
    }
}
