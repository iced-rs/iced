//! A widget to make animated views.
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

/// The logic of a [`Transition`].
///
/// A [`Program`] can be used to group several [`Animation`]s when used with [`grouped`].
pub trait Program: 'static {
    /// The type of value the [`Program`] transitions its state to.
    type Target: Copy + 'static;

    /// Transitions the [`Program`] from its current state
    /// towards the target value at the given time.
    fn tick(&mut self, target: Self::Target, now: Instant);

    /// Returns true if the [`Program`] is currently in progress.
    fn is_animating(&self, now: Instant) -> bool;
}

impl<I> Program for Animation<I>
where
    I: Float + Clone + Copy + PartialEq + 'static,
{
    type Target = I;

    fn tick(&mut self, target: Self::Target, now: Instant) {
        self.go_mut(target, now);
    }

    fn is_animating(&self, now: Instant) -> bool {
        self.is_animating(now)
    }
}

/// A widget that can be used to animate its contents.
pub struct Transition<'a, Message, Theme, Renderer, P>
where
    P: Program,
{
    init: Box<dyn Fn() -> P + 'a>,
    view: Box<dyn Fn(&P, Instant) -> Element<'a, Message, Theme, Renderer> + 'a>,
    element: Element<'a, Message, Theme, Renderer>,
    width: Length,
    height: Length,
    key: Key,
    id: Option<widget::Id>,
    target_value: P::Target,
}

impl<'a, Message, Theme, Renderer, P> Transition<'a, Message, Theme, Renderer, P>
where
    Renderer: core::Renderer,
    P: Program,
{
    /// Creates a new [`Transition`].
    ///
    /// The `init` closure will be used to initialize an animation.
    ///
    /// The `view` closure will receive the animation and an [`Instant`], which can be used for interpolating values.
    /// This will be called every frame until the given `target_value` is reached.
    pub fn new(
        init: impl Fn() -> P + 'a,
        target_value: P::Target,
        view: impl Fn(&P, Instant) -> Element<'a, Message, Theme, Renderer> + 'a,
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

    /// Sets the [`widget::Id`] of the [`Transition`].
    ///
    /// The [`widget::Id`] can subsequently be used to reset the [`Animation`] via [`reset`].
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

impl<Message, Theme, Renderer, P> Widget<Message, Theme, Renderer>
    for Transition<'_, Message, Theme, Renderer, P>
where
    Renderer: core::Renderer,
    P: Program,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<P>>()
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
        let State::<P> {
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
        let State::<P> {
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
        let State::<P> {
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
            animation.tick(self.target_value, *instant);
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
        let State::<P> { should_reset, .. } = tree.state.downcast_mut();
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

impl<'a, Message, Theme, Renderer, P> From<Transition<'a, Message, Theme, Renderer, P>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: core::Renderer + 'a,
    P: Program,
{
    fn from(transition: Transition<'a, Message, Theme, Renderer, P>) -> Self {
        Self::new(transition)
    }
}

struct State<P: Program> {
    animation: P,
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

/// Reset the [`Animation`] of a [`Transition`].
pub fn reset<Message>(id: impl Into<widget::Id>) -> iced_runtime::Task<Message>
where
    Message: iced_runtime::futures::MaybeSend + 'static,
{
    let id = id.into();
    iced_runtime::task::widget(reset_raw(id)).discard()
}

/// An [`Operation`] to reset the [`Animation`] of a [`Transition`].
pub fn reset_raw(id: impl Into<widget::Id>) -> impl Operation {
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

/// Creates a new [`Transition`] widget with a custom [`Program`].
///
/// This can be useful if you need to use multiple [`Animation`]s without resorting to nesting [`Transition`]s.
pub fn grouped<'a, Message, Theme, Renderer, P>(
    init: impl Fn() -> P + 'a,
    target_value: P::Target,
    view: impl Fn(&P, Instant) -> Element<'a, Message, Theme, Renderer> + 'a,
) -> Transition<'a, Message, Theme, Renderer, P>
where
    Renderer: core::Renderer,
    P: Program,
{
    Transition::new(init, target_value, view)
}
