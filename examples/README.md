# Examples

__Iced moves fast and the `master` branch can contain breaking changes!__ If
you want to learn about a specific release, check out [the release list].

[the release list]: https://github.com/hecrj/iced/releases


## [Tour](tour)

A simple UI tour showcasing different widgets that can be built using Iced. It
also shows how the library can be integrated into an existing system.

The example can run both on native and web platforms, using the same GUI code!

The native renderer of the example is built on top of [`ggez`], a game library
for Rust. Currently, it is using a [personal fork] to [add a `FontCache` type]
and [fix some issues with HiDPI].

The web version uses `iced_web` directly. This crate is still a work in
progress. In particular, the styling of elements is not finished yet
(text color, alignment, sizing, etc).

The implementation consists of different modules:
  - __[`tour`]__ contains the actual cross-platform GUI code: __state__,
    __messages__, __update logic__ and __view logic__.
  - __[`iced_ggez`]__ implements a simple renderer for each of the used widgets
    on top of the graphics module of [`ggez`].
  - __[`widget`]__ conditionally re-exposes the correct platform widgets based
    on the target architecture.
  - __[`main`]__ integrates Iced with [`ggez`] and connects the [`tour`] with
    the [`renderer`].
  - __[`lib`]__ exposes the [`tour`] types and conditionally implements the
    WebAssembly entrypoint in the [`web`] module.

The conditional compilation awkwardness from targetting both native and web
platforms should be handled seamlessly by the `iced` crate in the near future!

If you want to run it as a native app:

```
cd examples/tour
cargo run
```

If you want to run it on web, you will need [`wasm-pack`]:

```
cd examples/tour
wasm-pack build --target web
```

Then, simply serve the directory with any HTTP server. For instance:

```
python3 -m http.server
```

[![Tour - Iced][gui_gif]][gui_gfycat]

[`ggez`]: https://github.com/ggez/ggez
[`tour`]: tour/src/tour.rs
[`iced_ggez`]: tour/src/iced_ggez
[`widget`]: tour/src/widget.rs
[`main`]: tour/src/main.rs
[`lib`]: tour/src/lib.rs
[`web`]: tour/src/web.rs
[personal fork]: https://github.com/hecrj/ggez
[add a `FontCache` type]: https://github.com/ggez/ggez/pull/679
[fix some issues with HiDPI]: https://github.com/hecrj/ggez/commit/dfe2fd2423c51a6daf42c75f66dfaeaacd439fb1
[gui_gif]: https://thumbs.gfycat.com/VeneratedSourAurochs-small.gif
[gui_gfycat]: https://gfycat.com/veneratedsouraurochs


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
