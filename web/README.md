# `iced_web`
[![Documentation](https://docs.rs/iced_web/badge.svg)][documentation]
[![Crates.io](https://img.shields.io/crates/v/iced_web.svg)](https://crates.io/crates/iced_web)
[![License](https://img.shields.io/crates/l/iced_web.svg)](https://github.com/hecrj/iced/blob/master/LICENSE)
[![project chat](https://img.shields.io/badge/chat-on_zulip-brightgreen.svg)](https://iced.zulipchat.com)

`iced_web` takes [`iced_core`] and builds a WebAssembly runtime on top. It achieves this by introducing a `Widget` trait that can be used to produce VDOM nodes.

The crate is currently a __very experimental__, simple abstraction layer over [`dodrio`].

![iced_core](../docs/graphs/web.png)

[documentation]: https://docs.rs/iced_web
[`iced_core`]: ../core
[`dodrio`]: https://github.com/fitzgen/dodrio
