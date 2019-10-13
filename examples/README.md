# Examples
__Iced moves fast and the `master` branch can contain breaking changes!__ If
you want to learn about a specific release, check out [the release list].

[the release list]: https://github.com/hecrj/iced/releases


## [Tour](tour.rs)
A simple UI tour showcasing different widgets that can be built using Iced.

The example can run both on native and web platforms, using the same GUI code!

[![Tour - Iced][gui_gif]][gui_gfycat]

[gui_gif]: https://thumbs.gfycat.com/VeneratedSourAurochs-small.gif
[gui_gfycat]: https://gfycat.com/veneratedsouraurochs

On native, the example uses:
  - [`iced_winit`], as a bridge between [`iced_native`] and [`winit`].
  - [`iced_wgpu`], a WIP Iced renderer built on top of [`wgpu`] and supporting
    Vulkan, Metal, D3D11, and D3D12 (OpenGL and WebGL soon!).

The web version uses [`iced_web`], which is still a work in progress. In
particular, the styling of elements is not finished yet (text color, alignment,
sizing, etc).

The __[`tour`]__ file contains all the code of the example! All the
cross-platform GUI is defined in terms of __state__, __messages__,
__update logic__ and __view logic__.

[`tour`]: tour.rs
[`iced_winit`]: ../winit
[`iced_native`]: ../native
[`iced_wgpu`]: ../wgpu
[`iced_web`]: ../web
[`winit`]: https://github.com/rust-windowing/winit
[`wgpu`]: https://github.com/gfx-rs/wgpu-rs

#### Running the native version
Simply use [Cargo](https://doc.rust-lang.org/cargo/reference/manifest.html#examples)
to run the example:

```
cargo run --example tour
```

#### Running the web version
```
TODO
```


## [Coffee]

Since [Iced was born in May], it has been powering the user interfaces in
[Coffee], an experimental 2D game engine.

If you want to give Iced a try without having to write your own renderer,
the __[`ui` module]__ in [Coffee] is probably your best choice as of now.

[![Tour - Coffee][coffee_gui_gif]][coffee_gui_gfycat]

[Iced was born in May]: https://github.com/hecrj/coffee/pull/35
[`ui` module]: https://docs.rs/coffee/0.3.2/coffee/ui/index.html
[Coffee]: https://github.com/hecrj/coffee
[coffee_gui_gif]: https://thumbs.gfycat.com/GloomyWeakHammerheadshark-small.gif
[coffee_gui_gfycat]: https://gfycat.com/gloomyweakhammerheadshark
