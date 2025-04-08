use crate::core::window;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Span {
    Boot,
    Update {
        message: String,
        commands_spawned: usize,
    },
    View {
        window: window::Id,
    },
    Layout {
        window: window::Id,
    },
    Interact {
        window: window::Id,
    },
    Draw {
        window: window::Id,
    },
    Present {
        window: window::Id,
    },
    Custom {
        window: window::Id,
        name: String,
    },
}

impl Span {
    pub fn stage(&self) -> Stage {
        match self {
            Span::Boot => Stage::Boot,
            Span::Update { .. } => Stage::Update,
            Span::View { window } => Stage::View(*window),
            Span::Layout { window } => Stage::Layout(*window),
            Span::Interact { window } => Stage::Interact(*window),
            Span::Draw { window } => Stage::Draw(*window),
            Span::Present { window } => Stage::Present(*window),
            Span::Custom { window, name } => {
                Stage::Custom(*window, name.clone())
            }
        }
    }
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
    Present(window::Id),
    Custom(window::Id, String),
}

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Stage::Boot => "Boot",
            Stage::Update => "Update",
            Stage::View(_) => "View",
            Stage::Layout(_) => "Layout",
            Stage::Interact(_) => "Interact",
            Stage::Draw(_) => "Draw",
            Stage::Present(_) => "Present",
            Stage::Custom(_, name) => name,
        })
    }
}
