## Todos

A todos tracker inspired by [TodoMVC]. It showcases dynamic layout, text input, checkboxes, scrollables, icons, and async actions! It automatically saves your tasks in the background, even if you did not finish typing them.

All the example code is located in the __[`main`]__ file.

<div align="center">
  <a href="https://iced.rs/examples/todos.mp4">
    <img src="https://iced.rs/examples/todos.gif">
  </a>
</div>

You can run the native version with `cargo run`:
```
cargo run --package todos
```

The web version can be run with [`trunk`]:

```
cd examples/todos
trunk serve
```

[`main`]: src/main.rs
[TodoMVC]: http://todomvc.com/
[`trunk`]: https://trunkrs.dev/
