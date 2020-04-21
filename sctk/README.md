# `iced_sctk`
[![Documentation](https://docs.rs/iced_sctk/badge.svg)][documentation]
[![Crates.io](https://img.shields.io/crates/v/iced_sctk.svg)](https://crates.io/crates/iced_sctk)
[![License](https://img.shields.io/crates/l/iced_sctk.svg)](https://github.com/hecrj/iced/blob/master/LICENSE)
[![project chat](https://img.shields.io/badge/chat-on_zulip-brightgreen.svg)](https://iced.zulipchat.com)

`iced_sctk` offers some convenient abstractions on top of [`iced_native`] to quickstart development when using [`sctk`].

It exposes a renderer-agnostic `Application` trait that can be implemented and then run with a simple call. The use of this trait is optional. A `conversion` module is provided for users that decide to implement a custom event loop.

![iced_sctk](../docs/graphs/sctk.png)

[documentation]: https://docs.rs/iced_sctk
[`iced_native`]: ../native
[`sctk`]: https://github.com/rust-windowing/sctk

## Installation
Add `iced_sctk` as a dependency in your `Cargo.toml`:

```toml
iced_sctk = "0.1"
```

__Iced moves fast and the `master` branch can contain breaking changes!__ If
you want to learn about a specific release, check out [the release list].

[the release list]: https://github.com/hecrj/iced/releases
