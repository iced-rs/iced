# `iced_wgpu`
[![Documentation](https://docs.rs/iced_wgpu/badge.svg)][documentation]
[![Crates.io](https://img.shields.io/crates/v/iced_wgpu.svg)](https://crates.io/crates/iced_wgpu)
[![License](https://img.shields.io/crates/l/iced_wgpu.svg)](https://github.com/iced-rs/iced/blob/master/LICENSE)
[![Discord Server](https://img.shields.io/discord/628993209984614400?label=&labelColor=6A7EC2&logo=discord&logoColor=ffffff&color=7389D8)](https://discord.gg/3xZJ65GAhd)

`iced_wgpu` is a [`wgpu`] renderer for [`iced_runtime`]. For now, it is the default renderer of Iced on [native platforms].

[`wgpu`] supports most modern graphics backends: Vulkan, Metal, DX12, OpenGL, and WebGPU.

<p align="center">
  <img alt="The native target" src="../docs/graphs/native.png" width="80%">
</p>

[documentation]: https://docs.rs/iced_wgpu
[`iced_runtime`]: ../runtime
[`wgpu`]: https://github.com/gfx-rs/wgpu
[native platforms]: https://github.com/gfx-rs/wgpu#supported-platforms
[WebGPU API]: https://gpuweb.github.io/gpuweb/
[`wgpu_glyph`]: https://github.com/hecrj/wgpu_glyph
