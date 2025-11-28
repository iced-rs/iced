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
    Present {
        window: window::Id,
        prepare: present::Stage,
        render: present::Stage,
        layers: usize,
    },
    Custom {
        name: String,
    },
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
    Prepare(present::Primitive),
    Render(present::Primitive),
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

pub mod present {
    use crate::core::time::Duration;

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct Stage {
        pub quads: Duration,
        pub triangles: Duration,
        pub shaders: Duration,
        pub text: Duration,
        pub images: Duration,
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
}
