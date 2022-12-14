use crate::window;

/// layer surface events
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerEvent {
    /// layer surface Done
    Done,
    /// layer surface focused
    Focused,
    /// layer_surface unfocused
    Unfocused,
}
