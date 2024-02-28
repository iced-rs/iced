use crate::core::time::{Duration, SystemTime};
use crate::core::window;

use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Timing {
    pub stage: Stage,
    pub start: SystemTime,
    pub duration: Duration,
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Stage {
    Boot,
    Update,
    View(window::Id),
    Layout(window::Id),
    Interact(window::Id),
    Draw(window::Id),
    Render(window::Id),
    Custom(window::Id, String),
}
