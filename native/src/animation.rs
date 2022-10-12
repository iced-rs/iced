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
#[derive(Debug, Clone)]
pub struct Animation {
    id: Id,
    start: Instant,
    keyframes: Vec<Keyframe>,
    playhead: Option<Keyframe>,
    again: Again,
    message: bool //TODO: add a message to be sent on animation completion
}

impl std::default::Default for Animation {
    fn default() -> Self {
        Animation {
            id: Id::unique(),
            start: Instant::now(),
            keyframes: Vec::new(),
            playhead: None,
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

    /// Get the current width of the animated widget.
    pub fn width(&self) -> Option<Length> {
        self.playhead.as_ref().and_then(|p| p.width)
    }

    /// Get the current height of the animated widget
    pub fn height(&self) -> Option<Length> {
        self.playhead.as_ref().and_then(|p| p.height)
    }

    /// Get the current padding of the animated widget
    pub fn padding(&self) -> Option<Padding> {
        self.playhead.as_ref().and_then(|p| p.padding)
    }

    /// Get the current spacing of the animated widget
    pub fn spacing(&self) -> Option<u16> {
        self.playhead.as_ref().and_then(|p| p.spacing)
    }

    /// Generate a new frame given the keyframes, requested [`Again`] type, and set the playhead
    /// to the newly generated value.
    /// Default are the default values that a widget may have. Each widget can choose what it's
    /// default state should be. The default value should then be modified by the the widget
    /// for the given start values of a widget. For example the [`row::width`] method on an
    /// unanimated [`row`] is also the default width that should be passed here.
    /// The default should be `None` for values that are never animatable for the widget.
    pub fn interp(&mut self, app_start: &Instant, mut default: Keyframe) -> Request {
        if let Some(playhead) = &mut self.playhead {
            playhead.after = Instant::now().duration_since(self.start);
            //println!("not adding playhead");

            if let Some(width) = &playhead.width {
                let mut lower_bound_iter = self.keyframes.iter().peekable();
                let lower_bound = loop {
                    if let Some(keyframe) = lower_bound_iter.next() {
                        if let Some(next_keyframe) = lower_bound_iter.peek() {
                            if keyframe.after < playhead.after && next_keyframe.after > playhead.after {
                                break keyframe.after
                            }
                        }
                    } else {break Duration::ZERO}
                };
                //println!("lower bound at {:?}, playhead at {:?}", lower_bound, playhead.after);
                // THIS IS BAD
                //let upper_bound = match self.keyframes.iter().find(|&&keyframe| keyframe.width.is_some() /* && keyframe.after > app_start */) {
                //    Some(keyframe) => *keyframe,
                //    None => match self.keyframes.last() {
                //        Some(frame) => *frame,
                //        None => default
                //    }
                //}.after;
                match playhead.ease {
                    Ease::Linear => {
                        //println!("calculate linear animation");
                    }
                }
            }

            if let Some(height) = playhead.height {
                // TODO
            }

            if let Some(padding) = playhead.padding {
                // TODO
            }

            if let Some(spacing) = playhead.spacing {
                // TODO
            }

        } else {
            // This is the first interp on the animation. Set the playhead at the beginning.
            //println!("setting playhead as default");
            default.after = Instant::now().duration_since(self.start);
            self.playhead = Some(default);
        }
        Request::Timeout(Instant::now() + Duration::from_secs(3))
        //Request::None
    }

}

/// The requested values to a widget to have `after`
/// the given [`Duration`] is equal to the duration since
/// the animation was created.
/// A keyframe can animate many different widget types.
/// Widgets are not guarenteed to use all values. Extra
/// assignemnts will be ignored when animating.
#[derive(Debug, Clone)]
pub struct Keyframe {
    id: Id,
    after: Duration,
    width: Option<Length>,
    height: Option<Length>,
    spacing: Option<u16>,
    padding: Option<Padding>,
    ease: Ease,
}

impl std::default::Default for Keyframe {
    fn default() -> Self {
        Keyframe {
            id: Id::unique(),
            after: Duration::ZERO,
            width: None,
            height: None,
            spacing: Some(0),
            padding: Some(0.into()),
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

    /// Set the desired padding by the time the keyframe's delay.
    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = Some(spacing);
        self
    }

    /// Set the desired padding by the time the keyframe's delay.
    pub fn padding(mut self, padding: Padding) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Set the easing algorithm for the values changed by this keyframe.
    pub fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }

    /// Set the the time after animation creation that the widget animation
    /// will have values set in the keyframe.
    pub fn after(mut self, after: Duration) -> Self {
        self.after = after;
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
