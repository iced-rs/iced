# `iced_native`
[![Documentation](https://docs.rs/iced_native/badge.svg)][documentation]
[![Crates.io](https://img.shields.io/crates/v/iced_native.svg)](https://crates.io/crates/iced_native)
[![License](https://img.shields.io/crates/l/iced_native.svg)](https://github.com/hecrj/iced/blob/master/LICENSE)
[![project chat](https://img.shields.io/badge/chat-on_zulip-brightgreen.svg)](https://iced.zulipchat.com)

`iced_native` takes [`iced_core`] and builds a native runtime on top of it, featuring:
- A custom layout engine, greatly inspired by [`druid`]
- Event handling for all the built-in widgets
- A renderer-agnostic API

To achieve this, it introduces a bunch of reusable interfaces:
- A `Widget` trait, which is used to implement new widgets: from layout requirements to event and drawing logic.
- A bunch of `Renderer` traits, meant to keep the crate renderer-agnostic.
- A `Windowed` trait, leveraging [`raw-window-handle`], which can be implemented by graphical renderers that target _windows_. Window-based shells (like [`iced_winit`]) can use this trait to stay renderer-agnostic.

<p align="center">
  <img alt="The native target" src="../docs/graphs/native.png" width="80%">
</p>

[documentation]: https://docs.rs/iced_native
[`iced_core`]: ../core
[`iced_winit`]: ../winit
[`druid`]: https://github.com/xi-editor/druid
[`raw-window-handle`]: https://github.com/rust-windowing/raw-window-handle

## Installation
Add `iced_native` as a dependency in your `Cargo.toml`:

```toml
iced_native = "0.4"
```

__Iced moves fast and the `master` branch can contain breaking changes!__ If
you want to learn about a specific release, check out [the release list].

[the release list]: https://github.com/hecrj/iced/releases
