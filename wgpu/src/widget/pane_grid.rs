//! Let your users split regions of your application and organize layout dynamically.
//!
//! [![Pane grid - Iced](https://thumbs.gfycat.com/MixedFlatJellyfish-small.gif)](https://gfycat.com/mixedflatjellyfish)
use crate::Renderer;

pub use iced_native::pane_grid::{
    Axis, Direction, DragEvent, Focus, KeyPressEvent, Pane, ResizeEvent, Split,
    State,
};

/// A collection of panes distributed using either vertical or horizontal splits
/// to completely fill the space available.
///
/// [![Pane grid - Iced](https://thumbs.gfycat.com/MixedFlatJellyfish-small.gif)](https://gfycat.com/mixedflatjellyfish)
///
/// This is an alias of an `iced_native` pane grid with an `iced_wgpu::Renderer`.
pub type PaneGrid<'a, Message> = iced_native::PaneGrid<'a, Message, Renderer>;
