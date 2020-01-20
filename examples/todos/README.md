## Todos

A todos tracker inspired by [TodoMVC]. It showcases dynamic layout, text input, checkboxes, scrollables, icons, and async actions! It automatically saves your tasks in the background, even if you did not finish typing them.

All the example code is located in the __[`main`]__ file.

<div align="center">
  <a href="https://gfycat.com/littlesanehalicore">
    <img src="https://thumbs.gfycat.com/LittleSaneHalicore-small.gif" height="400px">
  </a>
</div>

You can run the native version with `cargo run`:
```
cargo run --package todos
```
We have not yet implemented a `LocalStorage` version of the auto-save feature. Therefore, it does not work on web _yet_!

[`main`]: src/main.rs
[TodoMVC]: http://todomvc.com/
