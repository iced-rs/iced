# Examples
__Iced moves fast and the `master` branch can contain breaking changes!__ If
you want to learn about a specific release, check out [the release list].

[the release list]: https://github.com/hecrj/iced/releases

## [Tour](tour.rs)

A simple UI tour that can run both on native platforms and the web! It showcases different widgets that can be built using Iced.

The __[`tour`]__ file contains all the code of the example! All the cross-platform GUI is defined in terms of __state__, __messages__, __update logic__ and __view logic__.

<div align="center">
  <a href="https://gfycat.com/politeadorableiberianmole">
    <img src="https://thumbs.gfycat.com/PoliteAdorableIberianmole-small.gif">
  </a>
</div>

[`tour`]: tour.rs
[`iced_winit`]: ../winit
[`iced_native`]: ../native
[`iced_wgpu`]: ../wgpu
[`iced_web`]: ../web
[`winit`]: https://github.com/rust-windowing/winit
[`wgpu`]: https://github.com/gfx-rs/wgpu-rs

You can run the native version with `cargo run`:
```
cargo run --example tour
```

The web version can be run by following [the usage instructions of `iced_web`] or by accessing [iced.rs](https://iced.rs/)!

[the usage instructions of `iced_web`]: ../web#usage


## [Todos](todos.rs)

A simple todos tracker inspired by [TodoMVC]. It showcases dynamic layout, text input, checkboxes, scrollables, icons, and async actions! It automatically saves your tasks in the background, even if you did not finish typing them.

All the example code is located in the __[`todos`]__ file.

<div align="center">
  <a href="https://gfycat.com/littlesanehalicore">
    <img src="https://thumbs.gfycat.com/LittleSaneHalicore-small.gif" height="400px">
  </a>
</div>

You can run the native version with `cargo run`:
```
cargo run --example todos
```
We have not yet implemented a `LocalStorage` version of the auto-save feature. Therefore, it does not work on web _yet_!

[`todos`]: todos.rs
[TodoMVC]: http://todomvc.com/

## [Coffee]

Since [Iced was born in May], it has been powering the user interfaces in
[Coffee], an experimental 2D game engine.


<div align="center">
  <a href="https://gfycat.com/gloomyweakhammerheadshark">
    <img src="https://thumbs.gfycat.com/GloomyWeakHammerheadshark-small.gif">
  </a>
</div>

[Iced was born in May]: https://github.com/hecrj/coffee/pull/35
[`ui` module]: https://docs.rs/coffee/0.3.2/coffee/ui/index.html
[Coffee]: https://github.com/hecrj/coffee
