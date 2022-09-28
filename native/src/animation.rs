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
/// Each transition, whether that be width, height, padding, etc must reference
/// a keyframe as a trigger to start the transition. This allows for many
/// animations to start at the same time, or have some start at an offset of
/// others.
#[derive(Debug)]
pub struct Animation {
    id: Id,
    keyframes: Vec<Keyframe>,
    transitions: Vec<Transition>,
    loop_type: LoopType,
}

impl Default for Animation {
    fn default() -> Self {
        Animation {
            id: Id::unique(),
            keyframes: Vec::new(),
            transitions: Vec::new(),
            loop_type: LoopType::None,
        }
    }
}

impl Animation {
    /// Create a new animation to be attached to a widget.
    pub fn new(keyframes: Vec<Keyframe>, transitions: Vec<Transition>, loop_type: LoopType) -> Self{
        Animation {
            id: Id::unique(),
            keyframes,
            transitions,
            loop_type,
        }
    }

    /// Create an animation with an Id. This is useful if keyframes or transitions need to be modified,
    /// appended, deleted, etc before the animation is complete.
    pub fn with_id(id: Id, keyframes: Vec<Keyframe>, transitions: Vec<Transition>, loop_type: LoopType) -> Self{
        Animation {
            id,
            keyframes,
            transitions,
            loop_type,
        }
    }
}

/// A point in time that can trigger animation start. The time doesn't have to match exactly.
/// Any time after the keyframe's start will trigger a step in the [`Transition`].
/// start is the time from when the animation is created.
#[derive(Debug)]
pub struct Keyframe {
    id: Id,
    start: Duration,
}

impl Keyframe {
    /// Create a new Keyframe
    pub fn new(start: Duration) -> Self {
        Keyframe {
            id: Id::unique(),
            start,
        }
    }

    /// Create a new keyframe with an Id known before the keyframe is created.
    /// Useful for animations that have keyframes known in advance.
    pub fn with_id(id: Id, start: Duration) -> Self {
        Keyframe {
            id,
            start,
        }
    }
}

/// The data needed for transitioning between two values.
//#[derive(Debug)]
//pub struct Transition {
//    trigger: Id,
//    duration: Duration,
//    at: u16,
//    end: u16,
//    ease: Ease,
//}

type Transition = Box<dyn Animatable>;

impl fmt::Debug for Transition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: make debug printing more verbose
        write!(f, "Debug print of a transition")
    }
}

/// A trait that must be implemented by any value that is animatable.
pub trait Animatable {
    /// The current state of the interpolation.
    fn at<A>(&self) -> A;
}

/// A type to request animating the widget's width
#[derive(Debug)]
pub struct Width {
    trigger: Id,
    duration: Duration,
    at: u16,
    end: u16,
    ease: Ease,
}

impl Animatable for Width {
    fn at(&self) -> Length {
        Length::Units(self.at)
    }
}

/// The action that should be completed if to replay an animation if at all
#[derive(Debug)]
pub enum LoopType {
    /// Jump back to the beginning of the animation and replay
    Jump,
    /// Flip the order of the animation and play in reverse when animation finishes.
    Bounce,
    /// The animation plays once then stays at it's completed possition.
    None,
    // TODO: Should have a loop u16 number of times options like Repeat(LoopType, u16)
}

/// The function used to transition between given values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ease {
    /// Animate linearly, animates at the same speed through the whole animation.
    Linear,
    // TODO: in, out, cubic, should also be options
}

/// A type that forces the start and end to to be of the same length type.
/// Currently only Length::Units and Length::FillPortion are supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Bounds {
    Units(u16, u16),
    FillPortion(u16, u16),
}

impl Bounds {
    fn new(start: Length, end: Length) -> Bounds {
        if let Length::Units(s) = start {
            if let Length::Units(e) = end {
                return Bounds::Units(s, e);
            }
        }
        if let Length::FillPortion(s) = start {
            if let Length::FillPortion(e) = end {
                return Bounds::FillPortion(s, e);
            }
        }
        // TODO: Should be possible to use different types to make this error at compile time, rather than runtime
        panic!("Only Length::Units and Length::FillPortion animatable vaules");
    }

    fn get_start(&self) -> u16 {
        match self {
            Bounds::Units(s, _) => *s,
            Bounds::FillPortion(s, _) => *s,
        }
    }

    fn get_end(&self) -> u16 {
        match self {
            Bounds::Units(_, e) => *e,
            Bounds::FillPortion(_, e) => *e,
        }
    }

    fn as_length(self) -> Length {
        match self {
            Bounds::Units(_, e) => Length::Units(e),
            Bounds::FillPortion(_, e) => Length::FillPortion(e),
        }
    }
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
