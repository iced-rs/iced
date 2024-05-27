//! Add internal animations a [`Widget`], mainly for style transitions.

/// Animation timeline for the [`Widget`] to show animations on state changes.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AnimationTimeline {
    /// Animation direction, which depends on when the animation is started by the widget.
    pub direction: AnimationDirection,
    /// The instant the animation was started at (`None` if it is not running).
    pub started_at: Option<std::time::Instant>,
    /// The progress of the animation, between 0.0 and 1.0.
    pub progress: f32,
    /// The progress the animation has been started at.
    pub initial_progress: f32,
    /// The type of effect for the animation.
    pub effect: AnimationEffect,
}

/// The type of effect for the [`AnimationTimeline`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AnimationEffect {
    /// Transition is linear.
    Linear,
    /// Transition is a cubic ease out.
    #[default]
    EaseOut,
}

/// The duration of an [`AnimationTimeline`].
#[derive(Debug, Clone)]
pub struct AnimationDuration {
    /// Duration, in milliseconds, for the [`AnimationTimeline`].
    pub duration_ms: u16,
    /// If `true`, the animation is disabled
    /// and the transition is instantaneous.
    pub enabled: bool,
}

impl AnimationDuration {
    /// Create a [`AnimationDuration`] with the given duration in milliseconds.
    pub fn new(duration_ms: u16) -> Self {
        Self {
            duration_ms,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Direction of the [`AnimationTimeline`].
pub enum AnimationDirection {
    #[default]
    /// The [`AnimationTimeline`] goes forward.
    ///
    /// For example with a [`Button`], forward means it goes from non-hovered to hovered state, or from a non-pressed state to a pressed state.
    Forward,
    /// The [`AnimationTimeline`] goes backward.
    ///
    /// For example with a [`Button`], forward means it goes back from hovered to non-hovered state, or from a pressed state to a non-pressed state.
    Backward,
}

impl AnimationTimeline {
    /// Create a [`AnimationTimeline`] with the given effect.
    pub fn new(effect: AnimationEffect) -> Self {
        Self {
            effect,
            ..Default::default()
        }
    }

    /// Check if the [`AnimationTimeline`] is running.
    pub fn is_running(&self) -> bool {
        self.started_at.is_some()
    }

    /// Reset the [`AnimationTimeline`].
    pub fn reset(&mut self) {
        self.direction = AnimationDirection::Forward;
        self.started_at = None;
        self.progress = 0.0;
        self.initial_progress = 0.0;
    }

    /// Update the [`AnimationTimeline`] progress, if necessary, and return the need to request a redraw.
    ///
    /// Bidirectional animations are, for instance, used by buttons to go from a hovered to a non-hovered state,
    /// whereas scrollables use monodirectional animations to go from offset A to B.
    pub fn on_redraw_request_update(
        &mut self,
        forward_duration: &AnimationDuration,
        backward_duration: &AnimationDuration,
        now: std::time::Instant,
        bidirectional: bool,
    ) -> bool {
        // Is the animation running ?
        if let Some(started_at) = self.started_at {
            // Do not ask for redraw if the animation is done.
            if self.progress == 1.0
                && AnimationDirection::Forward == self.direction
            {
                // Only reset animations when they are not meant to go fully forward then backward again.
                if !bidirectional {
                    self.started_at = None;
                }
                return false;
            }

            // Instanteanously finish the animation when disabled.
            if !forward_duration.enabled
                && self.direction == AnimationDirection::Forward
            {
                self.progress = 1.0;
                self.started_at = None;
                return false;
            } else if !backward_duration.enabled
                && self.direction == AnimationDirection::Backward
            {
                self.progress = 0.0;
                self.started_at = None;
                return false;
            }

            // Reset the animation once it has gone forward and now fully backward,
            // and do not request a redraw.
            if self.progress == 0.0
                && self.direction == AnimationDirection::Backward
            {
                self.started_at = None;
                return false;
            } else {
                // Evaluate new progress.
                match &mut self.effect {
                    AnimationEffect::Linear => match self.direction {
                        AnimationDirection::Forward => {
                            self.progress = (self.initial_progress
                                + (((now - started_at).as_millis() as f64)
                                    / (forward_duration.duration_ms as f64))
                                    as f32)
                                .clamp(0.0, 1.0);
                        }
                        AnimationDirection::Backward => {
                            self.progress = (self.initial_progress
                                - (((now - started_at).as_millis() as f64)
                                    / (backward_duration.duration_ms as f64))
                                    as f32)
                                .clamp(0.0, 1.0);
                        }
                    },
                    AnimationEffect::EaseOut => match self.direction {
                        AnimationDirection::Forward => {
                            self.progress = (self.initial_progress
                                + ease_out_cubic(
                                    ((now - started_at).as_millis() as f32)
                                        / (forward_duration.duration_ms as f32),
                                ))
                            .clamp(0.0, 1.0);
                        }
                        AnimationDirection::Backward => {
                            self.progress = (self.initial_progress
                                - ease_out_cubic(
                                    ((now - started_at).as_millis() as f32)
                                        / (backward_duration.duration_ms
                                            as f32),
                                ))
                            .clamp(0.0, 1.0);
                        }
                    },
                }
            }
            return true;
        }
        false
    }

    /// Update an [`AnimationTimeline`] for the "hover" effect of a [`Widget`], on a [`CursorMoved`] event.
    ///
    /// Return the need to request a redraw.
    pub fn on_cursor_moved(&mut self, is_mouse_over: bool) -> bool {
        if is_mouse_over {
            // Is it already running ?
            if self.started_at.is_some() {
                // This is when the cursor re-enters the widget's area
                // before the animation finishes
                if self.direction == AnimationDirection::Backward {
                    // Change animation direction
                    self.direction = AnimationDirection::Forward;
                    // Start from where the animation was at
                    self.initial_progress = self.progress;
                    self.started_at = Some(std::time::Instant::now());
                }
            } else {
                // Start the animation
                self.direction = AnimationDirection::Forward;
                self.started_at = Some(std::time::Instant::now());
                self.progress = 0.0;
                self.initial_progress = 0.0;
            }
            self.progress != 1.0
        } else if self.started_at.is_some() {
            // This is when the cursor leaves the widget's area
            match self.direction {
                AnimationDirection::Forward => {
                    // Change animation direction
                    self.direction = AnimationDirection::Backward;
                    // Start from where the animation was at
                    self.initial_progress = self.progress;
                    self.started_at = Some(std::time::Instant::now());
                    true
                }
                AnimationDirection::Backward => true,
            }
        } else {
            false
        }
    }

    /// Start a [`AnimationTimeline`].
    ///
    /// This is useful for instance to start a style animation when
    /// a button is pressed.
    pub fn start(&mut self) {
        self.started_at = Some(std::time::Instant::now());
        self.direction = AnimationDirection::Forward;
        self.progress = 0.0;
        self.initial_progress = 0.0;
    }

    /// Rewind a [`AnimationTimeline`].
    ///
    /// This is useful for instance to rewind a style animation when
    /// a [`Button`] has been pressed and is now released.
    pub fn rewind(&mut self) {
        self.started_at = Some(std::time::Instant::now());
        self.direction = AnimationDirection::Backward;
        self.initial_progress = self.progress;
    }
}

/// Based on Robert Penner's infamous easing equations, MIT license.
pub fn ease_out_cubic(t: f32) -> f32 {
    let p = t - 1f32;
    p * p * p + 1f32
}
