## Tour

A simple UI tour that can run both on native platforms and the web! It showcases different widgets that can be built using Iced.

The __[`main`]__ file contains all the code of the example! All the cross-platform GUI is defined in terms of __state__, __messages__, __update logic__ and __view logic__.

<div align="center">
  <a href="https://iced.rs/examples/tour.mp4">
    <img src="https://iced.rs/examples/tour.gif">
  </a>
</div>

[`main`]: src/main.rs
[`iced_winit`]: ../../winit
[`iced_native`]: ../../native
[`iced_wgpu`]: ../../wgpu
[`iced_web`]: https://github.com/iced-rs/iced_web
[`winit`]: https://github.com/rust-windowing/winit
[`wgpu`]: https://github.com/gfx-rs/wgpu-rs

You can run the native version with `cargo run`:
```
cargo run --package tour
```

The web version can be run with [`trunk`]:

```
cd examples/tour
trunk serve
```

[`trunk`]: https://trunkrs.dev/
