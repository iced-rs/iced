# `iced_wgpu`
[![Documentation](https://docs.rs/iced_wgpu/badge.svg)][documentation]
[![Crates.io](https://img.shields.io/crates/v/iced_wgpu.svg)](https://crates.io/crates/iced_wgpu)
[![License](https://img.shields.io/crates/l/iced_wgpu.svg)](https://github.com/iced-rs/iced/blob/master/LICENSE)
[![Discord Server](https://img.shields.io/discord/628993209984614400?label=&labelColor=6A7EC2&logo=discord&logoColor=ffffff&color=7389D8)](https://discord.gg/3xZJ65GAhd)

`iced_wgpu` is a [`wgpu`] renderer for [`iced_native`]. For now, it is the default renderer of Iced on [native platforms].

[`wgpu`] supports most modern graphics backends: Vulkan, Metal, and DX12 (OpenGL and WebGL are still WIP). Additionally, it will support the incoming [WebGPU API].

Currently, `iced_wgpu` supports the following primitives:
- Text, which is rendered using [`wgpu_glyph`]. No shaping at all.
- Quads or rectangles, with rounded borders and a solid background color.
- Clip areas, useful to implement scrollables or hide overflowing content.
- Images and SVG, loaded from memory or the file system.
- Meshes of triangles, useful to draw geometry freely.

<p align="center">
  <img alt="The native target" src="../docs/graphs/native.png" width="80%">
</p>

[documentation]: https://docs.rs/iced_wgpu
[`iced_native`]: ../native
[`wgpu`]: https://github.com/gfx-rs/wgpu
[native platforms]: https://github.com/gfx-rs/wgpu#supported-platforms
[WebGPU API]: https://gpuweb.github.io/gpuweb/
[`wgpu_glyph`]: https://github.com/hecrj/wgpu_glyph

## Installation
Add `iced_wgpu` as a dependency in your `Cargo.toml`:

```toml
iced_wgpu = "0.4"
```

__Iced moves fast and the `master` branch can contain breaking changes!__ If
you want to learn about a specific release, check out [the release list].

[the release list]: https://github.com/iced-rs/iced/releases

## Current limitations

The current implementation is quite naive; it uses:

- A different pipeline/shader for each primitive
- A very simplistic layer model: every `Clip` primitive will generate new layers
- _Many_ render passes instead of preparing everything upfront
- A glyph cache that is trimmed incorrectly when there are multiple layers (a [`glyph_brush`] limitation)

Some of these issues are already being worked on! If you want to help, [get in touch!]

[get in touch!]: ../CONTRIBUTING.md
[`glyph_brush`]: https://github.com/alexheretic/glyph-brush
