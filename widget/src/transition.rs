//! A widget to make animated views.
//!
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::core::animation::{Animation, Float};
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::time::Instant;
use crate::core::widget::{self, Operation, Tree, tree};
use crate::core::{self, Element, Event, Length, Rectangle, Shell, Size, Vector, Widget};
use crate::space;

/// TODO
pub struct Transition<'a, Message, Theme, Renderer, I>
where
    I: Float + Clone + Copy + PartialEq + 'static,
{
    init: Box<dyn Fn() -> Animation<I> + 'a>,
    view: Box<dyn Fn(&Animation<I>, Instant) -> Element<'a, Message, Theme, Renderer> + 'a>,
    element: Element<'a, Message, Theme, Renderer>,
    width: Length,
    height: Length,
    key: Key,
    id: Option<widget::Id>,
    target_value: I,
}

impl<'a, Message, Theme, Renderer, I> Transition<'a, Message, Theme, Renderer, I>
where
    Renderer: core::Renderer,
    I: Float + Clone + Copy + PartialEq + 'static,
{
    /// TODO
    pub fn new(
        init: impl Fn() -> Animation<I> + 'a,
        target_value: I,
        view: impl Fn(&Animation<I>, Instant) -> Element<'a, Message, Theme, Renderer> + 'a,
    ) -> Self {
        Self {
            init: Box::new(init),
            view: Box::new(view),
            width: Length::Fill,
            height: Length::Fill,
            element: Element::new(space()),
            key: Key::default(),
            id: None,
            target_value,
        }
    }

    /// Sets the width of the [`Transition`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Transition`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the [`Id`] of the [`Transition`].
    ///
    /// The [`Id`] can subsequently be used to reset the [`Animation`] via [`reset`].
    pub fn id(mut self, id: impl Into<widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the key of the [`Transition`] widget for continuity.
    ///
    /// Changing the key will reset the [`Animation`].
    pub fn key(mut self, key: impl Hash) -> Self {
        self.key = Key::new(key);
        self
    }
}

impl<Message, Theme, Renderer, I> Widget<Message, Theme, Renderer>
    for Transition<'_, Message, Theme, Renderer, I>
where
    Renderer: core::Renderer,
    I: Float + Clone + Copy + PartialEq + 'static,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<I>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            animation: (self.init)(),
            instant: Instant::now(),
            key: self.key,
            should_reset: false,
        })
    }

    fn diff(&self, tree: &mut Tree) {
        let State::<I> {
            animation,
            key: old_key,
            should_reset,
            ..
        } = tree.state.downcast_mut();

        if *old_key != self.key {
            *old_key = self.key;
            *animation = (self.init)();
            *should_reset = false;
        }

        // Diff is deferred to layout
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let State::<I> {
            animation, instant, ..
        } = tree.state.downcast_ref();

        self.element = (self.view)(animation, *instant);
        let limits = limits.width(self.width).height(self.height);

        tree.diff_children(std::slice::from_ref(&self.element));

        let node =
            self.element
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &limits.loose());

        let size = limits.resolve(self.width, self.height, node.size());

        layout::Node::with_children(size, vec![node])
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
        let State::<I> {
            animation,
            instant,
            should_reset,
            ..
        } = tree.state.downcast_mut();

        let was_animating = animation.is_animating(*instant);

        if let core::Event::Window(core::window::Event::RedrawRequested(redraw)) = event {
            if *should_reset {
                *animation = (self.init)();
                *should_reset = false;
            }

            *instant = *redraw;
            animation.go_mut(self.target_value, *instant);
        }

        let is_animating = animation.is_animating(*instant);
        let just_finished = was_animating && !is_animating;

        if is_animating || *should_reset {
            shell.invalidate_layout();
            shell.request_redraw();
        } else if just_finished {
            shell.invalidate_layout();
            shell.request_redraw();
        }

        self.element.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            shell,
            viewport,
        );
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
        self.element.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
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
        self.element.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let State::<I> { should_reset, .. } = tree.state.downcast_mut();
        let mut should_reset_new = ShouldReset(*should_reset);

        operation.custom(self.id.as_ref(), layout.bounds(), &mut should_reset_new);
        *should_reset = should_reset_new.0;

        self.element.as_widget_mut().operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout<'a>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        self.element.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer, I> From<Transition<'a, Message, Theme, Renderer, I>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: core::Renderer + 'a,
    I: Float + Clone + Copy + PartialEq + 'static,
{
    fn from(transition: Transition<'a, Message, Theme, Renderer, I>) -> Self {
        Self::new(transition)
    }
}

struct State<I>
where
    I: Float + Clone + Copy + PartialEq + 'static,
{
    animation: Animation<I>,
    instant: Instant,
    key: Key,
    should_reset: bool,
}

#[derive(Default, Clone, Copy, Hash, PartialEq, Eq)]
struct Key(u64);

impl Key {
    fn new(data: impl Hash) -> Self {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        Self(hasher.finish())
    }
}

struct ShouldReset(bool);

/// Reset the [`Animation`] state of the [`Transition`] widget.
pub fn reset(id: impl Into<widget::Id>) -> impl Operation {
    struct Reset(widget::Id);

    impl Operation for Reset {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<()>)) {
            operate(self)
        }

        fn custom(
            &mut self,
            id: Option<&widget::Id>,
            _bounds: Rectangle,
            state: &mut dyn std::any::Any,
        ) {
            if id == Some(&self.0)
                && let Some(ShouldReset(should_reset)) = state.downcast_mut()
            {
                *should_reset = true;
            }
        }
    }

    Reset(id.into())
}
