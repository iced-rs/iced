//! Let your users split regions of your application and organize layout dynamically.
//!
//! [![Pane grid - Iced](https://thumbs.gfycat.com/MixedFlatJellyfish-small.gif)](https://gfycat.com/mixedflatjellyfish)
//!
//! # Example
//! The [`pane_grid` example] showcases how to use a [`PaneGrid`] with resizing,
//! drag and drop, and hotkey support.
//!
//! [`pane_grid` example]: https://github.com/hecrj/iced/tree/0.1/examples/pane_grid
//! [`PaneGrid`]: type.PaneGrid.html
use crate::Renderer;

pub use iced_native::pane_grid::{
    Axis, Configuration, Direction, DragEvent, Focus, Node, Pane, ResizeEvent,
    Split, State,
};

/// A collection of panes distributed using either vertical or horizontal splits
/// to completely fill the space available.
///
/// [![Pane grid - Iced](https://thumbs.gfycat.com/MixedFlatJellyfish-small.gif)](https://gfycat.com/mixedflatjellyfish)
///
/// This is an alias of an `iced_native` pane grid with an `iced_wgpu::Renderer`.
pub type PaneGrid<'a, Message> = iced_native::PaneGrid<'a, Message, Renderer>;

/// The content of a [`Pane`].
///
/// [`Pane`]: struct.Pane.html
pub type Content<'a, Message> =
    iced_native::pane_grid::Content<'a, Message, Renderer>;

/// The title bar of a [`Pane`].
///
/// [`Pane`]: struct.Pane.html
pub type TitleBar<'a, Message> =
    iced_native::pane_grid::TitleBar<'a, Message, Renderer>;
