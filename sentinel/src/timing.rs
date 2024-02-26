use crate::core::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Timing {
    pub stage: Stage,
    pub duration: Duration,
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Stage {
    Boot,
    Update,
    View,
    Layout,
    Interact,
    Draw,
    Render,
    Custom(String),
}
