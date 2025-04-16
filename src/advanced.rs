//! Leverage advanced concepts like custom widgets.
pub mod subscription {
    //! Write your own subscriptions.
    pub use crate::runtime::futures::subscription::{
        Event, EventStream, Hasher, MacOS, PlatformSpecific, Recipe,
        from_recipe, into_recipes,
    };
}

pub mod widget {
    //! Create custom widgets and operate on them.
    pub use crate::core::widget::*;
    pub use crate::runtime::task::widget as operate;
}

pub use crate::core::Shell;
pub use crate::core::clipboard::{self, Clipboard};
pub use crate::core::image;
pub use crate::core::layout::{self, Layout};
pub use crate::core::mouse;
pub use crate::core::overlay::{self, Overlay};
pub use crate::core::renderer::{self, Renderer};
pub use crate::core::svg;
pub use crate::core::text::{self, Text};
pub use crate::renderer::graphics;

pub use widget::Widget;
