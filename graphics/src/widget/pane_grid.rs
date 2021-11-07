//! Let your users split regions of your application and organize layout dynamically.
//!
//! [![Pane grid - Iced](https://thumbs.gfycat.com/MixedFlatJellyfish-small.gif)](https://gfycat.com/mixedflatjellyfish)
//!
//! # Example
//! The [`pane_grid` example] showcases how to use a [`PaneGrid`] with resizing,
//! drag and drop, and hotkey support.
//!
//! [`pane_grid` example]: https://github.com/hecrj/iced/tree/0.3/examples/pane_grid
use crate::Renderer;

pub use iced_native::widget::pane_grid::{
    Axis, Configuration, Content, Direction, DragEvent, Node, Pane,
    ResizeEvent, Split, State, TitleBar,
};

pub use iced_style::pane_grid::{Line, StyleSheet};

/// A collection of panes distributed using either vertical or horizontal splits
/// to completely fill the space available.
///
/// [![Pane grid - Iced](https://thumbs.gfycat.com/MixedFlatJellyfish-small.gif)](https://gfycat.com/mixedflatjellyfish)
///
/// This is an alias of an `iced_native` pane grid with an `iced_wgpu::Renderer`.
pub type PaneGrid<'a, Message, Backend> =
    iced_native::widget::PaneGrid<'a, Message, Renderer<Backend>>;
