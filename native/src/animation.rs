//! State management and calculation for widget state animations
use crate::Length;

/// A type for managing animations
///
/// Most animations are only temporary, so when done/idle
/// we can just drop the extra data. This type can also
/// handle the calculations for each animation based on the
/// [`Ease`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Animation {
    /// Used to hold the current animation state at a frame
    /// tells the iced to animate the next frame when calculating layout/
    Working(Working),
    /// Holds current value when animation is idle, or finished.
    /// Iced will skip animation calculations, but can be converted into
    /// [`Animation::Working`] after any `view()`.
    Idle(Idle),
}

impl std::default::Default for Animation {
    fn default() -> Self {
        Animation::Idle(Idle::default())
    }
}

/// The function used to transition between given values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ease {
    /// Animate linearly, animates at the same speed through the whole animation.
    Linear,
    // TODO: in, out, cubic, should also be options
}

/// The animation state at a specific frame
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Working {
    bounds: Bounds,
    at: Length,
    begin: usize,
    runtime: usize,
    ease: Ease,
}

/// The animation state when not running.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Idle {
    at: Length,
}

impl Default for Idle {
    fn default() -> Self {
        Idle { at: Length::Shrink }
    }
}

impl Animation {
    /// Creates an [`Animation`] that is idle for now, for when we want to animate later
    pub fn new_idle(at: Length) -> Animation {
        Animation::Idle(Idle {at})
    }

    /// Creates an animation that will begin to animate immediatly.
    pub fn new(start: Length, end: Length, runtime: usize, ease: Ease) -> Animation {
        let bounds = Bounds::new(start, end);
        Animation::Working(Working {
            bounds,
            at: start,
            begin: 0,
            runtime,
            ease,
        })
    }

    /// A helper function to get state now, whether the animation is [`Working`] or [`Idle`]
    pub fn at(&self) -> Length {
        match self {
            Animation::Working(state) => state.at,
            Animation::Idle(state) => state.at,
        }
    }

    /// Takes the current frame, and the time that the next frame should be rendered, and returns the state at that frame.
    pub fn step(&mut self, now: usize) {
        println!("step");
        match self {
            Animation::Idle(_at) => {},
            Animation::Working(mut state) => {
                match state.ease {
                    Ease::Linear => {
                        let start: f64 = state.bounds.get_start().into();
                        let end: f64 = state.bounds.get_end().into();
                        let slope = (end - start) / state.runtime as f64;
                        let value = start + (now - state.begin / state.runtime) as f64 * slope;

                        if value >= end {
                            state.at = state.bounds.as_length();
                        } else {
                            state.at = Length::Units(value.clamp(u16::MIN.into(), u16::MAX.into()).round() as u16);
                        }
                    }
                }
            }
        }
    }
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
