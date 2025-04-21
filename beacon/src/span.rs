use crate::core::window;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Span {
    Boot,
    Update {
        number: usize,
        message: String,
        tasks: usize,
        subscriptions: usize,
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
    Prepare {
        window: window::Id,
        primitive: Primitive,
    },
    Render {
        window: window::Id,
        primitive: Primitive,
    },
    Present {
        window: window::Id,
    },
    Custom {
        name: String,
    },
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
)]
pub enum Primitive {
    Quad,
    Triangle,
    Shader,
    Text,
    Image,
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
            Span::Prepare { primitive, .. } => Stage::Prepare(*primitive),
            Span::Render { primitive, .. } => Stage::Render(*primitive),
            Span::Present { window } => Stage::Present(*window),
            Span::Custom { name, .. } => Stage::Custom(name.clone()),
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
    Prepare(Primitive),
    Render(Primitive),
    Custom(String),
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
            Stage::Prepare(_) => "Prepare",
            Stage::Render(_) => "Render",
            Stage::Present(_) => "Present",
            Stage::Custom(name) => name,
        })
    }
}
