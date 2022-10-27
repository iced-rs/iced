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
    
    /// For iced internal use only.
    /// Not intended for end user.
    /// 
    /// Insert a starting keyframe. Used to allow widgets to animate
    /// from specified widget layout then into animation, without requiring
    /// the end user to manually write out the initial widget layout again.
    pub fn insert(mut self, keyframe: impl Into<Handle>) -> Self {
        self.keyframes.insert(0, keyframe.into());
        self
    }

    /// What the animation should do after it has completed.
    /// Read as a sentance,
    /// "the animation should `play(Again::FromBeginning)`"
    pub fn play(mut self, again: Again) -> Self {
        self.again = again;
        self
    }
    
    fn bounds<'a>(&'a self, now: &'a Duration, i: usize) -> (Option<&'a Handle>, &'a Handle) {
        let mut lower_bound_iter = self.keyframes.iter().filter(|handle| handle.keyframe.modifiers()[i].is_some()).peekable();
        let mut lower_bound = lower_bound_iter.next();
        if let Some(lb) = lower_bound {
            lower_bound = loop {
                if let Some(handle) = lower_bound_iter.next() {
                    if let Some(next_handle) = lower_bound_iter.peek() {
                        if handle.keyframe.after() < *now && next_handle.keyframe.after() > *now {
                            break Some(handle)
                        }
                    }
                } else {break lower_bound}
            };
        }
        println!("lower bound = {:?}", lower_bound);
        
        let upper_bound = match self.keyframes.iter().find(|&handle| handle.keyframe.modifiers()[i].is_some() && handle.keyframe.after() > *now ) {
            Some(handle) => handle,
            None => self.keyframes.last().unwrap(),
        };
        println!("upper bound = {:?}", upper_bound);
        
        (lower_bound, upper_bound)
    }    
    
    fn calc_linear(&self, now: &Duration, lower_bound: &Handle, upper_bound: &Handle, i: usize) -> isize {
        let (lease, lb) = lower_bound.keyframe.modifiers()[i].unwrap();
        let (uease, ub) = upper_bound.keyframe.modifiers()[i].unwrap();

        let percent_done = (*now - lower_bound.keyframe.after()).as_millis() as f64 / ( upper_bound.keyframe.after() - lower_bound.keyframe.after()).as_millis() as f64;
        let delta = (ub - lb) as f64;
        let value = (percent_done * delta + (lb as f64)) as isize;

        if ub > lb {
            ub.min(value.into())
        } else {
            ub.max(value.into())    
        }
    }
    
    /// Interpolate values for animation.
    pub fn interp(&self,_app_start: &Instant, playhead: &mut Handle ) {
        
        let now = Instant::now().duration_since(self.start);
        if playhead.keyframe.after() <= self.keyframes.last().unwrap().keyframe.after() {
            playhead.keyframe.modifiers_mut().iter_mut().enumerate().for_each(| (i, playhead) | {
                if let Some((ease, val)) = playhead {
                    // TODO handle mismatched lower/upper_bounds. If one exists and one not, what do we do?
                    let (lower_bound, upper_bound) = self.bounds(&now, i);                
                    
                    // TODO this needs to be changed up the upper_bound's ease
                    if lower_bound.is_some() /* && upper_bound.is_some() */ {
                        *val = match ease {
                            Ease::Linear => {
                                self.calc_linear(&now, lower_bound.unwrap(), upper_bound, i)
                            }                
                        };
                    }
                }
            });
        }
        playhead.keyframe.set_after(now);
        
        // TODO: Should set playhead.after to `now`
        // or maybe playhead doesn't need to hold now?
    }

}

/// A trait so widgets can easily animate their own values,
/// without having to manage all of the calculations.
pub trait Keyframe: std::fmt::Debug {
    //fn new() -> Self;
    
    /// Get the time the ketframe is pinned to "after" the start of the animation.
    /// At this time it's modifiers will have its requested value.
    fn after(&self) -> Duration;
    
    /// Set new duration.
    fn set_after(&mut self, after: Duration);
    
    /// This is a Vec of modifiers. The data type needs to be as such, so the animation
    /// calculations can be generalized across any widget type.
    ///
    /// TODO is it possible to use arrays for this? Would be nice to store this on stack rather than heap.
    /// The size is known for each widget, but not for the trait.
    //fn modifiers(&self) -> &Vec<Vec<Option<(Ease, isize)>>>;
    fn modifiers(&self) -> &Vec<Option<(Ease, isize)>>;
    
    /// A mutable verseion of `modifiers`
    fn modifiers_mut(&mut self) -> &mut Vec<Option<(Ease, isize)>>;
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
