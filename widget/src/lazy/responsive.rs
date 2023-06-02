use crate::core::event::{self, Event};
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget;
use crate::core::widget::tree::{self, Tree};
use crate::core::{
    self, Clipboard, Element, Length, Point, Rectangle, Shell, Size, Widget,
    IME,
};
use crate::horizontal_space;

use ouroboros::self_referencing;
use std::cell::{RefCell, RefMut};
use std::marker::PhantomData;
use std::ops::Deref;

/// A widget that is aware of its dimensions.
///
/// A [`Responsive`] widget will always try to fill all the available space of
/// its parent.
#[allow(missing_debug_implementations)]
pub struct Responsive<'a, Message, Renderer = crate::Renderer> {
    view: Box<dyn Fn(Size) -> Element<'a, Message, Renderer> + 'a>,
    content: RefCell<Content<'a, Message, Renderer>>,
}

impl<'a, Message, Renderer> Responsive<'a, Message, Renderer>
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
        view: impl Fn(Size) -> Element<'a, Message, Renderer> + 'a,
    ) -> Self {
        Self {
            view: Box::new(view),
            content: RefCell::new(Content {
                size: Size::ZERO,
                layout: None,
                element: Element::new(horizontal_space(0)),
            }),
        }
    }
}

struct Content<'a, Message, Renderer> {
    size: Size,
    layout: Option<layout::Node>,
    element: Element<'a, Message, Renderer>,
}

impl<'a, Message, Renderer> Content<'a, Message, Renderer>
where
    Renderer: core::Renderer,
{
    fn layout(&mut self, renderer: &Renderer) {
        if self.layout.is_none() {
            self.layout =
                Some(self.element.as_widget().layout(
                    renderer,
                    &layout::Limits::new(Size::ZERO, self.size),
                ));
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        new_size: Size,
        view: &dyn Fn(Size) -> Element<'a, Message, Renderer>,
    ) {
        if self.size == new_size {
            return;
        }

        self.element = view(new_size);
        self.size = new_size;
        self.layout = None;

        tree.diff(&self.element);
    }

    fn resolve<R, T>(
        &mut self,
        tree: &mut Tree,
        renderer: R,
        layout: Layout<'_>,
        view: &dyn Fn(Size) -> Element<'a, Message, Renderer>,
        f: impl FnOnce(
            &mut Tree,
            R,
            Layout<'_>,
            &mut Element<'a, Message, Renderer>,
        ) -> T,
    ) -> T
    where
        R: Deref<Target = Renderer>,
    {
        self.update(tree, layout.bounds().size(), view);
        self.layout(renderer.deref());

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

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Responsive<'a, Message, Renderer>
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

    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        Length::Fill
    }

    fn layout(
        &self,
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
        operation: &mut dyn widget::Operation<Message>,
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

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        ime: &dyn IME,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let state = tree.state.downcast_mut::<State>();
        let mut content = self.content.borrow_mut();

        let mut local_messages = vec![];
        let mut local_shell = Shell::new(&mut local_messages);

        let status = content.resolve(
            &mut state.tree.borrow_mut(),
            renderer,
            layout,
            &self.view,
            |tree, renderer, layout, element| {
                element.as_widget_mut().on_event(
                    tree,
                    event,
                    layout,
                    cursor_position,
                    renderer,
                    clipboard,
                    ime,
                    &mut local_shell,
                )
            },
        );

        if local_shell.is_layout_invalid() {
            content.layout = None;
        }

        shell.merge(local_shell, std::convert::identity);

        status
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
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
                    tree,
                    renderer,
                    theme,
                    style,
                    layout,
                    cursor_position,
                    viewport,
                )
            },
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
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
                element.as_widget().mouse_interaction(
                    tree,
                    layout,
                    cursor_position,
                    viewport,
                    renderer,
                )
            },
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        use std::ops::DerefMut;

        let state = tree.state.downcast_ref::<State>();

        let overlay = OverlayBuilder {
            content: self.content.borrow_mut(),
            tree: state.tree.borrow_mut(),
            types: PhantomData,
            overlay_builder: |content: &mut RefMut<'_, Content<'_, _, _>>,
                              tree| {
                content.update(tree, layout.bounds().size(), &self.view);
                content.layout(renderer);

                let Content {
                    element,
                    layout: content_layout_node,
                    ..
                } = content.deref_mut();

                let content_layout = Layout::with_offset(
                    layout.bounds().position() - Point::ORIGIN,
                    content_layout_node.as_ref().unwrap(),
                );

                element
                    .as_widget_mut()
                    .overlay(tree, content_layout, renderer)
            },
        }
        .build();

        let has_overlay = overlay.with_overlay(|overlay| {
            overlay.as_ref().map(overlay::Element::position)
        });

        has_overlay
            .map(|position| overlay::Element::new(position, Box::new(overlay)))
    }
}

impl<'a, Message, Renderer> From<Responsive<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: core::Renderer + 'a,
    Message: 'a,
{
    fn from(responsive: Responsive<'a, Message, Renderer>) -> Self {
        Self::new(responsive)
    }
}

#[self_referencing]
struct Overlay<'a, 'b, Message, Renderer> {
    content: RefMut<'a, Content<'b, Message, Renderer>>,
    tree: RefMut<'a, Tree>,
    types: PhantomData<Message>,

    #[borrows(mut content, mut tree)]
    #[covariant]
    overlay: Option<overlay::Element<'this, Message, Renderer>>,
}

impl<'a, 'b, Message, Renderer> Overlay<'a, 'b, Message, Renderer> {
    fn with_overlay_maybe<T>(
        &self,
        f: impl FnOnce(&overlay::Element<'_, Message, Renderer>) -> T,
    ) -> Option<T> {
        self.borrow_overlay().as_ref().map(f)
    }

    fn with_overlay_mut_maybe<T>(
        &mut self,
        f: impl FnOnce(&mut overlay::Element<'_, Message, Renderer>) -> T,
    ) -> Option<T> {
        self.with_overlay_mut(|overlay| overlay.as_mut().map(f))
    }
}

impl<'a, 'b, Message, Renderer> overlay::Overlay<Message, Renderer>
    for Overlay<'a, 'b, Message, Renderer>
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
            let translation = position - overlay.position();

            overlay.layout(renderer, bounds, translation)
        })
        .unwrap_or_default()
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
    ) {
        let _ = self.with_overlay_maybe(|overlay| {
            overlay.draw(renderer, theme, style, layout, cursor_position);
        });
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.with_overlay_maybe(|overlay| {
            overlay.mouse_interaction(
                layout,
                cursor_position,
                viewport,
                renderer,
            )
        })
        .unwrap_or_default()
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        ime: &dyn IME,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.with_overlay_mut_maybe(|overlay| {
            overlay.on_event(
                event,
                layout,
                cursor_position,
                renderer,
                clipboard,
                ime,
                shell,
            )
        })
        .unwrap_or(event::Status::Ignored)
    }

    fn is_over(&self, layout: Layout<'_>, cursor_position: Point) -> bool {
        self.with_overlay_maybe(|overlay| {
            overlay.is_over(layout, cursor_position)
        })
        .unwrap_or_default()
    }
}
