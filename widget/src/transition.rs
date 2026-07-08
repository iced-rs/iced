//! A widget to make animated views.
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::core::animation::{Animation, Float};
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::shell;
use crate::core::time::Instant;
use crate::core::widget::{self, Operation, Tree, tree};
use crate::core::{self, Element, Event, Length, Point, Rectangle, Shell, Size, Vector, Widget};
use crate::space;

/// The logic of a [`Transition`].
pub trait Program: 'static {
    /// The type of value that the [`Program`] animates.
    type Value: Copy + 'static;

    /// Transitions the [`Program`] from its current state towards the given value at
    /// the given time.
    fn go(&mut self, value: Self::Value, now: Instant);

    /// Returns true if the [`Program`] is currently in progress.
    fn is_animating(&self, now: Instant) -> bool;
}

impl<I> Program for Animation<I>
where
    I: Float + Clone + Copy + PartialEq + 'static,
{
    type Value = I;

    fn go(&mut self, value: Self::Value, now: Instant) {
        self.go_mut(value, now);
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
    on_finish: Option<Box<dyn Fn() -> Message + 'a>>,
    element: Element<'a, Message, Theme, Renderer>,
    next_element: Option<Element<'a, Message, Theme, Renderer>>,
    last_limits: layout::Limits,
    new_layout: Option<layout::Node>,
    key: Key,
    id: Option<widget::Id>,
    value: P::Value,
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
    /// This will be called every frame until the given `value` is reached.
    pub fn new<E>(
        init: impl Fn() -> P + 'a,
        value: P::Value,
        view: impl Fn(&P, Instant) -> E + 'a,
    ) -> Self
    where
        E: Into<Element<'a, Message, Theme, Renderer>>,
    {
        Self {
            init: Box::new(init),
            view: Box::new(move |program, at| view(program, at).into()),
            on_finish: None,
            element: Element::new(space()),
            next_element: None,
            new_layout: None,
            last_limits: layout::Limits::new(Size::ZERO, Size::ZERO),
            key: Key::default(),
            id: None,
            value,
        }
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

    /// Sets the message that will be produced when the [`Transition`] has finished animating.
    ///
    /// Note that if an [`Animation`] is set to loop forever, the message will never be produced!
    pub fn on_finish(self, on_finish: Message) -> Self
    where
        Message: Clone + 'a,
    {
        self.on_finish_maybe(on_finish)
    }

    /// Sets the message that will be produced when the [`Transition`] has finished animating, if [`Some`].
    ///
    /// Note that if an [`Animation`] is set to loop forever, the message will never be produced!
    pub fn on_finish_maybe(mut self, on_finish: impl Into<Option<Message>>) -> Self
    where
        Message: Clone + 'a,
    {
        self.on_finish = on_finish
            .into()
            .map(|on_finish| Box::new(move || on_finish.clone()) as _);

        self
    }

    /// Sets the message that will be produced when the [`Transition`] has finished animating.
    ///
    /// This is analogous to [`Transition::on_finish`], but using a closure to produce
    /// the message.
    ///
    /// This closure will only be called when the [`Transition`] has actually finished animating and,
    /// therefore, this method is useful to reduce overhead if creating the resulting
    /// message is slow.
    ///
    /// Note that if an [`Animation`] is set to loop forever, the message will never be produced!
    pub fn on_finish_with(mut self, on_finish: impl Fn() -> Message + 'a) -> Self {
        self.on_finish = Some(Box::new(on_finish));
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
        self.element.as_widget().size()
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
            size: None,
        })
    }

    fn diff(&mut self, tree: &mut Tree) {
        let State::<P> {
            animation,
            key: old_key,
            should_reset,
            instant,
            ..
        } = tree.state.downcast_mut();

        if *old_key != self.key {
            *old_key = self.key;
            *animation = (self.init)();
            *should_reset = false;
        }

        if let Some(next_element) = self.next_element.take() {
            self.element = next_element;
        } else {
            self.element = (self.view)(animation, *instant);
        }

        tree.diff_children(std::slice::from_mut(&mut self.element));
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.last_limits = *limits;
        self.new_layout = None;

        self.element
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, &limits.loose())
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
        if let core::Event::Window(core::window::Event::RedrawRequested(redraw)) = event {
            let State::<P> {
                animation,
                instant,
                should_reset,
                size,
                ..
            } = tree.state.downcast_mut();

            if instant == redraw {
                shell.request_redraw();
            } else {
                let was_animating = animation.is_animating(*instant);

                if *should_reset {
                    *animation = (self.init)();
                    *should_reset = false;
                }

                *instant = *redraw;
                animation.go(self.value, *instant);

                let is_animating = animation.is_animating(*instant);
                let just_finished = was_animating && !is_animating;

                if is_animating || just_finished {
                    let size = *size;

                    let mut new = (self.view)(animation, *instant);
                    tree.diff_children(&mut [new.as_widget_mut()]);

                    let new_size = new.as_widget().size();

                    if size != Some(new_size) {
                        self.next_element = Some(new);
                        shell.invalidate_layout_with(shell::Diff::Perform);

                        let state = tree.state.downcast_mut::<State<P>>();
                        state.size = Some(new_size);
                    } else {
                        self.element = new;
                        self.new_layout = Some(self.element.as_widget_mut().layout(
                            &mut tree.children[0],
                            renderer,
                            &self.last_limits,
                        ));
                    }

                    shell.request_redraw();
                }

                if just_finished && let Some(on_finish) = &self.on_finish {
                    shell.publish(on_finish());
                }
            }
        }

        self.element.as_widget_mut().update(
            &mut tree.children[0],
            event,
            self::layout(layout, &self.new_layout),
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
            self::layout(layout, &self.new_layout),
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
            self::layout(layout, &self.new_layout),
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
        let layout = self::layout(layout, &self.new_layout);

        let mut should_reset = ShouldReset(false);
        operation.custom(self.id.as_ref(), layout.bounds(), &mut should_reset);

        if should_reset.0 {
            tree.state.downcast_mut::<State<P>>().should_reset = true;
        }

        self.element
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
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
            self::layout(layout, &self.new_layout),
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
    size: Option<Size<Length>>,
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
            operate(self);
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

fn layout<'a>(current: Layout<'a>, animated: &'a Option<layout::Node>) -> Layout<'a> {
    animated
        .as_ref()
        .map(|new_layout| Layout::with_offset(current.position() - Point::ORIGIN, new_layout))
        .unwrap_or(current)
}
