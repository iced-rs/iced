# Examples

__Iced moves fast and the `master` branch can contain breaking changes!__ If
you want to learn about a specific release, check out [the release list].

[the release list]: https://github.com/hecrj/iced/releases


## [Tour](tour)

A simple UI tour showcasing different widgets that can be built using Iced. It
also shows how the library can be integrated into an existing system.

The example is built on top of [`ggez`], a game library for Rust. Currently, it
is using a [personal fork] to [add a `FontCache` type] and
[fix some issues with HiDPI].

```
cargo run --example tour
```

[![Tour - Iced][gui_gif]][gui_gfycat]

[`ggez`]: https://github.com/ggez/ggez
[personal fork]: https://github.com/hecrj/ggez
[add a `FontCache` type]: https://github.com/ggez/ggez/pull/679
[fix some issues with HiDPI]: https://github.com/hecrj/ggez/commit/dfe2fd2423c51a6daf42c75f66dfaeaacd439fb1
[gui_gif]: https://thumbs.gfycat.com/VeneratedSourAurochs-small.gif
[gui_gfycat]: https://gfycat.com/veneratedsouraurochs


## [Coffee]

Since [Iced was born in May], it has been powering the user interfaces in
[Coffee], an experimental 2D game engine.

If you want to give Iced a try without having to write your own renderer,
the [`ui` module] in [Coffee] is probably your best choice as of now.

[![Tour - Coffee][coffee_gui_gif]][coffee_gui_gfycat]

[Iced was born in May]: https://github.com/hecrj/coffee/pull/35
[`ui` module]: https://docs.rs/coffee/0.3.2/coffee/ui/index.html
[Coffee]: https://github.com/hecrj/coffee
[coffee_gui_gif]: https://thumbs.gfycat.com/GloomyWeakHammerheadshark-small.gif
[coffee_gui_gfycat]: https://gfycat.com/gloomyweakhammerheadshark
