# Tour

A simple UI tour showcasing different widgets that can be built using Iced. It
also shows how the library can be integrated into an existing system.

The example is built on top of [`ggez`], a game library for Rust. Currently, it
is using a [personal fork] to [add a `FontCache` type] and
[fix some issues with HiDPI].

The implementation consists of different modules:
  - __[`tour`]__ contains the actual GUI code: __state__, __messages__,
    __update logic__ and __view logic__.
  - __[`renderer`]__ implements a simple renderer for each of the used widgets on
    top of the graphics module of [`ggez`].
  - __[`widget`]__ re-exposes Iced's built-in widgets with the renderer type parameter
    replaced with the implemented [`renderer`], for convenience.
  - __[`main`]__ integrates Iced with [`ggez`] and connects the [`tour`] with
    the [`renderer`].

```
cargo run --example tour
```

[![Tour - Iced][gui_gif]][gui_gfycat]

[`ggez`]: https://github.com/ggez/ggez
[`tour`]: tour.rs
[`renderer`]: renderer
[`widget`]: widget.rs
[`main`]: main.rs
[personal fork]: https://github.com/hecrj/ggez
[add a `FontCache` type]: https://github.com/ggez/ggez/pull/679
[fix some issues with HiDPI]: https://github.com/hecrj/ggez/commit/dfe2fd2423c51a6daf42c75f66dfaeaacd439fb1
[gui_gif]: https://thumbs.gfycat.com/VeneratedSourAurochs-small.gif
[gui_gfycat]: https://gfycat.com/veneratedsouraurochs
