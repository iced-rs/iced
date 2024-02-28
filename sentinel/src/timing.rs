use crate::core::time::{Duration, SystemTime};
use crate::core::window;

use serde::{Deserialize, Serialize};
use std::fmt;

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

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boot => write!(f, "Boot"),
            Self::Update => write!(f, "Update"),
            Self::View(_) => write!(f, "View"),
            Self::Layout(_) => write!(f, "Layout"),
            Self::Interact(_) => write!(f, "Interact"),
            Self::Draw(_) => write!(f, "Draw"),
            Self::Render(_) => write!(f, "Render"),
            Self::Custom(_, name) => f.write_str(name),
        }
    }
}
