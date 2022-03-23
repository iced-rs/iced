## Game of Life

An interactive version of the [Game of Life], invented by [John Horton Conway].

It runs a simulation in a background thread while allowing interaction with a `Canvas` that displays an infinite grid with zooming, panning, and drawing support.

The __[`main`]__ file contains the relevant code of the example.

<div align="center">
  <a href="https://gfycat.com/WhichPaltryChick">
    <img src="https://thumbs.gfycat.com/WhichPaltryChick-size_restricted.gif">
  </a>
</div>

You can run it with `cargo run`:
```
cargo run --package game_of_life
```

[`main`]: src/main.rs
[Game of Life]: https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life
[John Horton Conway]: https://en.wikipedia.org/wiki/John_Horton_Conway
