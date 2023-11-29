## Pane grid

A grid of panes that can be split, resized, and reorganized.

This example showcases the `PaneGrid` widget, which features:

* Vertical and horizontal splits
* Tracking of the last active pane
* Mouse-based resizing
* Drag and drop to reorganize panes
* Hotkey support
* Configurable modifier keys
* API to perform actions programmatically (`split`, `swap`, `resize`, etc.)

The __[`main`]__ file contains all the code of the example.

<div align="center">
  <img src="https://iced.rs/examples/pane_grid.gif">
</div>

You can run it with `cargo run`:
```
cargo run --package pane_grid
```

[`main`]: src/main.rs
