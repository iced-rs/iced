# Examples
__Iced moves fast and the `master` branch can contain breaking changes!__ If
you want to learn about a specific release, check out [the release list].

[the release list]: https://github.com/hecrj/iced/releases

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
