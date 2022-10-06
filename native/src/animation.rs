//! State management and calculation for widget state animations
use crate::Length;
use crate::widget::Id;

use iced_core::time::{Instant, Duration};
use std::fmt;

/// A type for managing animations
///
/// The id is used to allow for data to be extended to an animation, or notify
/// iced that a new animation should be added without considering the previous
/// animation.
/// A keyframe is a descriptor of what the widget dimensions should be at some
/// point in time. The time is relative to the first time the animation is
/// rendered. If you would like to animate again in some length of time, it is
/// recommended to subscribe to [`time::every`] and use that message to update
/// your view.
/// A keyframe is also used to describe the current state of the widget. This
/// is to guarentee that the curret tracked state is the same as what
/// animatable traits are available via the iced API.
#[derive(Debug)]
pub struct Animation {
    id: Id,
    keyframes: Vec<Keyframe>,
    at: Option<Keyframe>,
    again: Again,
    message: bool //TODO: add a message to be sent on animation completion
}

impl std::default::Default for Animation {
    fn default() -> Self {
        Animation {
            id: Id::unique(),
            keyframes: Vec::new(),
            at: None,
            again: Again::Never,
            message: false,
        }
    }
}

impl Animation {
    /// Create a new animation to be attached to a widget.
    pub fn new() -> Self{
        Animation::default()
    }

    /// Create a new animation to be attached to a widget.
    /// This is an optimization if you know your keyframes
    /// in advance.
    pub fn with_keyframes(keyframes: Vec<Keyframe>) -> Self {
        Animation {
            keyframes,
            ..Animation::default()
        }
    }

    /// Create an animation with an Id. This is useful if keyframes or transitions need to be modified,
    /// appended, deleted, etc before the animation is complete.
    pub fn with_id(id: Id) -> Self{
        Animation {
            id,
            ..Animation::default()
        }
    }

    /// Add a keyframe to an animation.
    pub fn push(mut self, keyframe: Keyframe) -> Self {
        self.keyframes.push(keyframe);
        self
    }

    /// What the animation should do after it has completed.
    /// Read as a sentance,
    /// "the animation should `play(Again::FromBeginning)`"
    pub fn play(mut self, again: Again) -> Self {
        self.again = again;
        self
    }

}

/// A point in time that can trigger animation start. The time doesn't have to match exactly.
/// Any time after the keyframe's start will trigger a step in the [`Transition`].
/// start is the time from when the animation is created.
#[derive(Debug)]
pub struct Keyframe {
    id: Id,
    delay: Duration,
    width: Option<Length>,
    height: Option<Length>,
    ease: Ease,
}

impl std::default::Default for Keyframe {
    fn default() -> Self {
        Keyframe {
            id: Id::unique(),
            delay: Duration::ZERO,
            width: None,
            height: None,
            ease: Ease::Linear,
        }
    }
}

impl Keyframe {
    /// Create a new Keyframe
    pub fn new() -> Self {
        Keyframe::default()
    }

    /// Create a new keyframe with an Id known before the keyframe is created.
    /// Useful for animations that have keyframes known in advance.
    pub fn with_id(id: Id) -> Self {
        Keyframe {
            id,
            ..Keyframe::default()
        }
    }

    /// Set the desired width by the time the keyframe's delay.
    pub fn width(mut self, width: Length) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the desired height by the time the keyframe's delay.
    pub fn height(mut self, height: Length) -> Self {
        self.height = Some(height);
        self
    }

    /// Set the easing algorithm for the values changed by this keyframe.
    pub fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }

    /// Set the the time after animation creation that the widget animation
    /// will have values set in the keyframe.
    pub fn after(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }
}

/// The function used to transition between given values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ease {
    /// Animate linearly, animates at the same speed through the whole animation.
    Linear,
    // TODO: in, out, cubic, should also be options
}

/// What the animation should do after it has completed.
/// Assigned via `play()`, read as a sentance,
/// "the animation should `play(Again::FromBeginning)`"
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Again {
    /// After the animation has finished, sit idle at the completed state.
    Never,
    /// After the animation has finished, jump back to its initial state and play again.
    FromBeginning,
    /// After the animation had finished, play the animation in reverse. Then forwards,
    /// then reverse again, repeating forever.
    Bounce,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
/// The time that a widget requests to be redrawn.
///
/// Widgets that implement a [`widget::step_state`] return this value.
/// It is a signal to the iced runtime when the widget should be redrawn.
/// Iced will only listen to the shortest time returned from all of the widgets
/// in the view. Because the widget will then be able to return its requested time
/// the next time its [`widget::step_state`] is called because of the widget that
/// required rerender sooner.
pub enum Request {
    /// Similar to javascript's requestAnimationFrame, This is a request to render "as soon as possible".
    /// Though for iced's runtime and most custom implementations that will be as soon as the refresh rate
    /// of the monitor.
    AnimationFrame,
    /// Request some time in the future. That isn't tied to any other value.
    Timeout(Instant),
    /// The widget doesn't need to reanimate. It is either done animating, or static.
    None,
}
