//! State management and calculation for widget state animations
use crate::Length;
use crate::Padding;
use crate::widget::Id;

use iced_core::time::{Instant, Duration};

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
    start: Instant,
    keyframes: Vec<Handle>,
    again: Again,
    message: bool //TODO: add a message to be sent on animation completion
}

impl std::default::Default for Animation {
    fn default() -> Self {
        Animation {
            id: Id::unique(),
            start: Instant::now(),
            keyframes: Vec::new(),
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
    pub fn with_keyframes(keyframes: Vec<Handle>) -> Self {
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
    pub fn push(mut self, keyframe: impl Into<Handle>) -> Self {
        self.keyframes.push(keyframe.into());
        self
    }

    /// What the animation should do after it has completed.
    /// Read as a sentance,
    /// "the animation should `play(Again::FromBeginning)`"
    pub fn play(mut self, again: Again) -> Self {
        self.again = again;
        self
    }
    
    /// placeholder for now
    pub fn interp(&self,_app_start: &Instant, _playhead: &mut Handle ) {
        
    }

}

/// A trait so widgets can easily animate their own values,
/// without having to manage all of the calculations.
pub trait Keyframe: std::fmt::Debug {
    //fn new() -> Self;
    
    /// Get the time the ketframe is pinned to "after" the start of the animation.
    /// At this time it's modifiers will have its requested value.
    fn after(&self) -> Duration;
    
    /// This is a Vec of modifiers. The data type needs to be as such, so the animation
    /// calculations can be generalized across any widget type.
    ///
    /// TODO is it possible to use arrays for this? Would be nice to store this on stack rather than heap.
    /// The size is known for each widget, but not for the trait.
    //fn modifiers(&self) -> &Vec<Vec<Option<(Ease, usize)>>>;
    fn modifiers(&self) -> &Vec<Option<(Ease, usize)>>;
}

/// A handle to the Keyframe trait to help make rustc happy.
#[derive(Debug)]
pub struct Handle {
    /// keyframe
    pub keyframe: Box<dyn Keyframe>
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

/// The time that a widget requests to be redrawn.
///
/// Widgets that implement a [`widget::interp`] return this value.
/// It is a signal to the iced runtime when the widget should be redrawn.
/// Iced will only listen to the shortest time returned from all of the widgets
/// in the view. Because the widget will then be able to return its requested time
/// the next time it's [`widget::interp`] is called because of the widget that
/// required rerender sooner.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum Request {
    /// Similar to javascript's requestAnimationFrame, This is a request to render "as soon as possible".
    /// Though for iced's runtime and most custom implementations that will be as soon as the refresh rate
    /// of the monitor.
    AnimationFrame,
    /// Request some time in the future.
    /// For For times shorter than or equal to the user's monitor's refresh rate, it is preferable to use
    /// Request::AnimationFrame.
    /// Widgets are expected to return `Instant::now() + Duration::from_/* arbitrary duration*/` for a
    /// requested time in the future. Widgets may want `app_start + Duration` if they want to animate 
    /// on a consistant multiple like a blinking cursor.
    Timeout(Instant),
    /// The widget doesn't need to reanimate. It is either done animating, or static.
    None,
}
