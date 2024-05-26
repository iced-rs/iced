//! Helpers for internal animations for a [`Widget`], mainly for style transitions.

/// The duration and enabled state of an animation.
#[derive(Debug, Clone)]
pub struct AnimationDuration {
    /// Duration, in milliseconds, for the animation.
    duration_ms: f32,
    /// If disabled, transition is instantaneous.
    enabled: bool,
}

impl AnimationDuration {
    /// Create a [`AnimationDuration`] with the given duration in milliseconds.
    pub fn new(duration_ms: f32) -> Self {
        Self {
            duration_ms,
            enabled: true,
        }
    }

    /// Get the duration, in milliseconds, of the animation.
    /// If `enabled` is set to `false`, duration is 0.0.
    pub fn get(&self) -> f32 {
        if self.enabled {
            self.duration_ms
        } else {
            0.
        }
    }

    /// Set the duration, in milliseconds, of the animation.
    pub fn set(&mut self, duration_ms: f32) {
        self.duration_ms = duration_ms;
    }

    /// Disable the animation, making it instantaneous.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Enable the animation.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Is the animation enabled
    pub fn enabled(&self) -> bool {
        self.enabled
    }
}
