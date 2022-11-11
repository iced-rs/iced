//! Store internal widget state in a state tree to ensure continuity.
use crate::animation;
use crate::Widget;

use iced_core::time::{Instant, Duration};
use std::any::{self, Any};
use std::borrow::{Borrow, BorrowMut};
use std::fmt;

/// A persistent state widget tree.
///
/// A [`Tree`] is normally associated with a specific widget in the widget tree.
#[derive(Debug)]
pub struct Tree {
    /// The tag of the [`Tree`].
    pub tag: Tag,

    /// The [`State`] of the [`Tree`].
    pub state: State,

    /// The children of the root widget of the [`Tree`].
    pub children: Vec<Tree>,
}

impl Tree {
    /// Creates an empty, stateless [`Tree`] with no children.
    pub fn empty() -> Self {
        Self {
            tag: Tag::stateless(),
            state: State::None,
            children: Vec::new(),
        }
    }

    /// Creates a new [`Tree`] for the provided [`Widget`].
    pub fn new<'a, Message, Renderer>(
        widget: impl Borrow<dyn Widget<Message, Renderer> + 'a>,
    ) -> Self
    where
        Renderer: crate::Renderer,
    {
        let widget = widget.borrow();

        Self {
            tag: widget.tag(),
            state: widget.state(),
            children: widget.children(),
        }
    }

    /// Reconciliates the current tree with the provided [`Widget`].
    ///
    /// If the tag of the [`Widget`] matches the tag of the [`Tree`], then the
    /// [`Widget`] proceeds with the reconciliation (i.e. [`Widget::diff`] is called).
    ///
    /// Otherwise, the whole [`Tree`] is recreated.
    ///
    /// [`Widget::diff`]: crate::Widget::diff
    pub fn diff<'a, Message, Renderer>(
        &mut self,
        new: impl Borrow<dyn Widget<Message, Renderer> + 'a>,
    ) where
        Renderer: crate::Renderer,
    {
        if self.tag == new.borrow().tag() {
            if self.state.is_interp() {
                let _ = new.borrow().interp(&mut self.state);
            }
            new.borrow().diff(self)
        } else {
            *self = Self::new(new);
        }
    }

    /// Reconciliates the children of the tree with the provided list of widgets.
    /// See [`diff`], this is similar, but uses a mutable reference to
    /// the tree, which allows us to step animations in [`user_interface::build`]
    /// Also takes an accumulator to track when the next redraw is necessary, if at all.
    pub fn diff_mut<'a, Message, Renderer>(
        &mut self,
        mut acc: animation::Request,
        mut new: impl BorrowMut<dyn Widget<Message, Renderer> + 'a>,
        app_start: &Instant,
    ) -> animation::Request
    where
        Renderer: crate::Renderer,
    {
        if self.tag == new.borrow_mut().tag() {
            acc = acc.min(new.borrow_mut().interp(&mut self.state, app_start));
            new.borrow_mut().diff_mut(acc, self, app_start)
        } else {
            *self = Self::new(new);
            acc
        }
    }

    /// Reconciliates the children of the tree with the provided list of [`Element`].
    pub fn diff_children<'a, Message, Renderer>(
        &mut self,
        new_children: &[impl Borrow<dyn Widget<Message, Renderer> + 'a>],
    ) where
        Renderer: crate::Renderer,
    {
        self.diff_children_custom(
            new_children,
            |tree, widget| tree.diff(widget.borrow()),
            |widget| Self::new(widget.borrow()),
        )
    }

    /// Reconciliates the children of the tree with the provided list of widgets using custom
    /// mutable child diff
    pub fn diff_children_mut<'a, Message, Renderer>(
        &mut self,
        acc: animation::Request,
        new_children: &mut [impl BorrowMut<dyn Widget<Message, Renderer> + 'a>],
        app_start: &Instant,
    ) -> animation::Request
    where
        Renderer: crate::Renderer,
    {
        self.diff_children_custom_mut(
            acc,
            new_children,
            app_start,
            |tree, widget| tree.diff_mut(acc, widget.borrow_mut(), app_start),
            |widget| Self::new(widget.borrow_mut()),
        )
    }

    /// Reconciliates the children of the tree with the provided list of [`Element`] using custom
    /// logic both for diffing and creating new widget state.
    pub fn diff_children_custom<T>(
        &mut self,
        new_children: &[T],
        diff: impl Fn(&mut Tree, &T),
        new_state: impl Fn(&T) -> Self,
    ) {
        if self.children.len() > new_children.len() {
            self.children.truncate(new_children.len());
        }

        for (child_state, new) in
            self.children.iter_mut().zip(new_children.iter())
        {
            diff(child_state, new);
        }

        if self.children.len() < new_children.len() {
            self.children.extend(
                new_children[self.children.len()..].iter().map(new_state),
            );
        }
    }
}

/// The identifier of some widget state.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Tag(any::TypeId);

impl Tag {
    /// Creates a [`Tag`] for a state of type `T`.
    pub fn of<T>() -> Self
    where
        T: 'static,
    {
        Self(any::TypeId::of::<T>())
    }

    /// Creates a [`Tag`] for a stateless widget.
    pub fn stateless() -> Self {
        Self::of::<()>()
    }
}

/// The internal [`State`] of a widget.
pub enum State {
    /// No meaningful internal state.
    ///
    /// Retuning a state of this type will *not* cause `interp` to be called on the widget the
    /// following render
    None,

    /// Some meaningful internal state.
    ///
    /// Retuning a state of this type will *not* cause `interp` to be called on the widget the
    /// following render
    Some(Box<dyn Any>),
    
    /// A `Some` state, but the state will be reused the
    /// next render to to allow for values to be interpolated between redraws
    /// `AnimationFrame` means the widget is requesting a redraw as soon as reasonably
    /// possible (most likely at the time of the next monitor refresh).
    ///
    /// Retuning a state of this type will cause `interp` to be called on the widget the
    /// following render
    AnimationFrame(AnimationState, Box<dyn Any>),
    
    /// A `Some` state, but the state will be reused the
    /// next render to to allow for values to be interpolated between redraws
    /// `Timeout` means the widget is requesting a redraw in the given `Duration`
    /// from widget interpolation.
    ///
    /// Retuning a state of this type will cause `interp` to be called on the widget the
    /// following render
    Timeout(AnimationState, Duration, Box<dyn Any>),
    
    /// A `Some` state, but the state will be reused the
    /// next render to to allow for values to be cached for optimizations. This will not cause
    /// iced to rerender, it just makes the data available the next render, that must be triggered
    /// by some other event.
    ///
    /// Retuning a state of this type will will cause `interp` to be called on the widget the
    /// following render
    Anytime(Box<dyn Any>)
}

impl State {
    /// Creates a new [`State`].
    pub fn new<T>(state: T) -> Self
    where
        T: 'static,
    {
        State::Some(Box::new(state))
    }
    
    /// Creates a new [`State`].
    pub fn newAnimationFrame<T>(hash:u64, state: T) -> Self
    where
        T: 'static,
    {
        State::AnimationFrame(AnimationState::new(hash), Box::new(state))
    }

    /// Downcasts the [`State`] to `T` and returns a reference to it.
    ///
    /// # Panics
    /// This method will panic if the downcast fails or the [`State`] is [`State::None`].
    pub fn downcast_ref<T>(&self) -> &T
    where
        T: 'static,
    {
        match self {
            State::None => panic!("Downcast on stateless state"),
            State::Some(state) => {
                state.downcast_ref().expect("Downcast widget state")
            }
            State::Anytime(state) => {
                state.downcast_ref().expect("Downcast widget state")
            }
            State::AnimationFrame(animation_state, widget_state) => {
                widget_state.downcast_ref().expect("Downcast widget state")
            }
            State::Timeout(animation_state, timeout, widget_state) => {
                widget_state.downcast_ref().expect("Downcast widget state")
            }
        }
    }

    /// Downcasts the [`State`] to `T` and returns a mutable reference to it.
    ///
    /// # Panics
    /// This method will panic if the downcast fails or the [`State`] is [`State::None`].
    pub fn downcast_mut<T>(&mut self) -> &mut T
    where
        T: 'static,
    {
        match self {
            State::None => panic!("Downcast on stateless state"),
            State::Some(state) => {
                state.downcast_mut().expect("Downcast widget state")
            }
            State::Anytime(state) => {
                state.downcast_mut().expect("Downcast widget state")
            }
            State::AnimationFrame(animation_state, widget_state) => {
                widget_state.downcast_mut().expect("Downcast widget state")
            }
            State::Timeout(animation_state, timeout, widget_state) => {
                widget_state.downcast_mut().expect("Downcast widget state")
            }
        }
    }
    
    fn is_interp(&self) -> bool {
        match self {
            State::AnimationFrame(_, _) => true,
            State::Timeout(_, _, _) => true,
            _ => false
        }
    }
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "State::None"),
            Self::Some(_) => write!(f, "State::Some"),
            Self::Anytime(_) => write!(f, "State::Anytime"),
            Self::AnimationFrame(_, _) => write!(f, "State::AnimationFrame"),
            Self::Timeout(_, timeout, _) => write!(f, "State::timeout {:?}", timeout),
        }
    }
}

/// Animation State
///
/// This is the data that is required to be held in a widget's state
/// to be animatable. This is the data required to be held in
/// tree::State::{AnimationFrame, Timeout}. When the hashs don't match
/// it means that view() returned a different animation on the widget,
/// so we need to start the new animation from the beginning, which is
/// the same as setting the start time to the new animation's start time
#[derive(Clone, Copy, Debug)]
pub struct AnimationState {
    /// The start time of the animation
    pub start: Instant,
    /// The hash of the animation. Used to check if the animation
    /// has changed since the previous animation started.
    pub hash: u64
}

impl AnimationState {
    fn new(hash: u64) -> Self {
        AnimationState {
            start: Instant::now(),
            hash,
        }
    }
}
