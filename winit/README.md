# `iced_winit`
[![Documentation](https://docs.rs/iced_winit/badge.svg)][documentation]
[![Crates.io](https://img.shields.io/crates/v/iced_winit.svg)](https://crates.io/crates/iced_winit)
[![License](https://img.shields.io/crates/l/iced_winit.svg)](https://github.com/hecrj/iced/blob/master/LICENSE)
[![project chat](https://img.shields.io/badge/chat-on_zulip-brightgreen.svg)](https://iced.zulipchat.com)

`iced_winit` offers some convenient abstractions on top of [`iced_native`] to quickstart development when using [`winit`].

It exposes a renderer-agnostic `Application` trait that can be implemented and then run with a simple call. The use of this trait is optional. A `conversion` module is provided for users that decide to implement a custom event loop.

<p align="center">
  <img alt="The native target" src="../docs/graphs/native.png" width="80%">
</p>

[documentation]: https://docs.rs/iced_winit
[`iced_native`]: ../native
[`winit`]: https://github.com/rust-windowing/winit

## Installation
Add `iced_winit` as a dependency in your `Cargo.toml`:

```toml
iced_winit = "0.3"
```

__Iced moves fast and the `master` branch can contain breaking changes!__ If
you want to learn about a specific release, check out [the release list].

[the release list]: https://github.com/hecrj/iced/releases
