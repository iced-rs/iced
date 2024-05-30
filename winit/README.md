# `iced_winit`
[![Documentation](https://docs.rs/iced_winit/badge.svg)][documentation]
[![Crates.io](https://img.shields.io/crates/v/iced_winit.svg)](https://crates.io/crates/iced_winit)
[![License](https://img.shields.io/crates/l/iced_winit.svg)](https://github.com/iced-rs/iced/blob/master/LICENSE)
[![Discord Server](https://img.shields.io/discord/628993209984614400?label=&labelColor=6A7EC2&logo=discord&logoColor=ffffff&color=7389D8)](https://discord.gg/3xZJ65GAhd)

`iced_winit` offers some convenient abstractions on top of [`iced_native`] to quickstart development when using [`winit`].

It exposes a renderer-agnostic `Application` trait that can be implemented and then run with a simple call. The use of this trait is optional. A `conversion` module is provided for users that decide to implement a custom event loop.

<p align="center">
  <img alt="The native target" src="../docs/graphs/native.png" width="80%">
</p>

[documentation]: https://docs.rs/iced_winit
[`iced_native`]: ../native
[`winit`]: https://github.com/rust-windowing/winit
