//! Add animations to widgets.

#[derive(Debug, Clone, Copy, PartialEq, Default)]
/// Direction of the animation
pub enum AnimationDirection {
    #[default]
    /// The animation goes forward
    Forward,
    /// The animation goes backward
    Backward,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Hover animation
pub struct Hover {
    /// Animation direction: forward means it goes from non-hovered to hovered state
    pub direction: AnimationDirection,
    /// The instant the animation was started at
    pub started_at: std::time::Instant,
    /// The progress of the animationn, between 0.0 and 1.0
    pub animation_progress: f32,
    /// The progress the animation has been started at
    pub initial_progress: f32,
}
