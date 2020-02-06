## Tour

A simple UI tour that can run both on native platforms and the web! It showcases different widgets that can be built using Iced.

The **[`main`]** file contains all the code of the example! All the cross-platform GUI is defined in terms of **state**, **messages**, **update logic** and **view logic**.

<div align="center">
  <a href="https://gfycat.com/politeadorableiberianmole">
    <img src="https://thumbs.gfycat.com/PoliteAdorableIberianmole-small.gif">
  </a>
</div>

[`main`]: src/main.rs
[`iced_winit`]: ../../winit
[`iced_native`]: ../../native
[`iced_wgpu`]: ../../wgpu
[`iced_web`]: ../../web
[`winit`]: https://github.com/rust-windowing/winit
[`wgpu`]: https://github.com/gfx-rs/wgpu-rs

You can run the native version with `cargo run`:

```
cargo run --package tour
```

If you are using vscode, you can follow [this guide](https://stackoverflow.com/questions/37586216/step-by-step-interactive-debugger-for-rust?answertab=active#tab-top) to learn how to start with only pressing F5.

Your `launch.json` would like this:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Run in library 'tour'",
      "cargo": {
        "args": ["run", "--package", "tour"],
        "filter": {
          "name": "tour"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

The web version can be run by following [the usage instructions of `iced_web`] or by accessing [iced.rs](https://iced.rs/)!

[the usage instructions of `iced_web`]: ../../web#usage
