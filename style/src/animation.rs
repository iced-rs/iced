//! Add animations to widgets.

/// Hover animation of the widget
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct HoverPressedAnimation {
    /// Animation direction: forward means it goes from non-hovered to hovered state
    pub direction: AnimationDirection,
    /// The instant the animation was started at (`None` if it is not running)
    pub started_at: Option<std::time::Instant>,
    /// The progress of the animationn, between 0.0 and 1.0
    pub animation_progress: f32,
    /// The progress the animation has been started at
    pub initial_progress: f32,
    /// The type of effect for the animation
    pub effect: AnimationEffect,
}

/// The type of effect for the animation
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AnimationEffect {
    /// The background color of the widget fades into the other color when hovered or pressed
    #[default]
    Fade,
    /// The background color of the widget instantly changes into the other color when hovered or pressed
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
/// Direction of the animation
pub enum AnimationDirection {
    #[default]
    /// The animation goes forward
    Forward,
    /// The animation goes backward
    Backward,
}

impl HoverPressedAnimation {
    /// Create a hover animation with the given transision effect
    pub fn new(effect: AnimationEffect) -> Self {
        Self {
            effect,
            ..Default::default()
        }
    }

    /// Check if the animation is running
    pub fn is_running(&self) -> bool {
        self.started_at.is_some()
    }

    /// Update the animation progress, if necessary, and returns the need to request a redraw.
    pub fn on_redraw_request_update(
        &mut self,
        animation_duration_ms: u16,
        now: std::time::Instant,
    ) -> bool {
        // Is the animation running ?
        if let Some(started_at) = self.started_at {
            if self.animation_progress >= 1.0 || animation_duration_ms == 0 {
                self.animation_progress = 1.0;
            }

            // Reset if ended
            if self.animation_progress == 0.0
                && self.direction == AnimationDirection::Backward
            {
                self.started_at = None;
            } else {
                // Evaluate new progress
                match &mut self.effect {
                    AnimationEffect::Fade => match self.direction {
                        AnimationDirection::Forward => {
                            let progress_since_start =
                                ((now - started_at).as_millis() as f64)
                                    / (animation_duration_ms as f64);
                            self.animation_progress = (self.initial_progress
                                + progress_since_start as f32)
                                .clamp(0.0, 1.0);
                        }
                        AnimationDirection::Backward => {
                            let progress_since_start =
                                ((now - started_at).as_millis() as f64)
                                    / (animation_duration_ms as f64);
                            self.animation_progress = (self.initial_progress
                                - progress_since_start as f32)
                                .clamp(0.0, 1.0);
                        }
                    },
                    AnimationEffect::None => {}
                }
            }
            return true;
        }
        false
    }

    /// Update the hovered state and return the need to request a redraw.
    pub fn on_cursor_moved_update(&mut self, is_mouse_over: bool) -> bool {
        if is_mouse_over {
            // Is it already running ?
            if self.started_at.is_some() {
                // This is when the cursor re-enters the widget's area before the animation finishes
                if self.direction == AnimationDirection::Backward {
                    // Change animation direction
                    self.direction = AnimationDirection::Forward;
                    // Start from where the animation was at
                    self.initial_progress = self.animation_progress;
                    self.started_at = Some(std::time::Instant::now());
                }
            } else {
                // Start the animation
                self.direction = AnimationDirection::Forward;
                self.started_at = Some(std::time::Instant::now());
                self.animation_progress = 0.0;
                self.initial_progress = 0.0;
            }
            self.animation_progress != 1.0
        } else if self.started_at.is_some() {
            // This is when the cursor leaves the widget's area
            match self.direction {
                AnimationDirection::Forward => {
                    // Change animation direction
                    self.direction = AnimationDirection::Backward;
                    // Start from where the animation was at
                    self.initial_progress = self.animation_progress;
                    self.started_at = Some(std::time::Instant::now());
                    true
                }
                AnimationDirection::Backward => true,
            }
        } else {
            false
        }
    }
}
