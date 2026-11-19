# iced_layer_shell

An experimental Wayland layer-shell runner for Iced applications. It creates a
single `wlr-layer-shell` surface and renders Iced with WGPU, making it useful
for panels, overlays, and desktop widgets.

This is a backend in this Iced checkout, not a drop-in replacement for Iced’s
usual windowing backend. It currently requires Wayland, WGPU, and a compositor
that supports both `wlr-layer-shell` and either `ext-background-effect` or KDE
KWin blur.

## Enable it

Enable the `layer-shell` feature on Iced. For an app alongside this repository’s
`iced/` checkout:

```toml
[dependencies]
iced = { path = "./iced", default-features = false, features = [
    "layer-shell",
    "thread-pool",
    "wayland",
] }
```

The layer-shell runner itself uses WGPU. The application entry point can use the
usual Iced builder, then call `run_layer_shell` instead of `run`:

```rust,ignore
iced::application("Panel", update, view)
    .window_size((560, 300))
    .run_layer_shell()?;
```

The layer surface is centered, placed on the top layer, and has no keyboard
interactivity by default. Transparent rendering is supported. Startup currently
fails if the compositor does not expose one of the supported blur protocols.

## Show and hide

To control visibility from another thread, pass a standard-library channel
receiver to `run_layer_shell_with_visibility`:

```rust,ignore
let (sender, receiver) = std::sync::mpsc::channel();

// Send commands from another thread:
sender.send(iced::VisibilityCommand::Hide)?;
sender.send(iced::VisibilityCommand::Show)?;

iced::application("Panel", update, view)
    .run_layer_shell_with_visibility(receiver)?;
```

`Hide` moves the mapped surface to the background layer and clears its input
region. `Show` restores it to the top layer and redraws it. This avoids
unmapping/remapping and the associated configure round-trip. A background-layer
surface may still be visible on an otherwise uncovered desktop.

## Current limits

- One layer surface per application; no regular multi-window support.
- WGPU renderer only.
- Async Iced task actions and unsupported task actions return errors.
- Plain-text clipboard read/write is supported.
- Layer placement, keyboard interactivity, and blur are currently fixed by the
  runner rather than exposed as configuration options.
- Blur depends on compositor protocol support; the runner reports an error
  when neither supported protocol is available.

The demo in the repository root is a runnable example of this backend.
