## `wgpu` integration

A demonstration of how to integrate Iced in an existing [`wgpu`] application.

The __[`main`]__ file contains all the code of the example.

<div align="center">
  <a href="https://gfycat.com/nicemediocrekodiakbear">
    <img src="https://thumbs.gfycat.com/NiceMediocreKodiakbear-small.gif">
  </a>
</div>

You can run it with `cargo run`:
```
cargo run --package integration_wgpu
```

### How to run this example with WebGL backend
NOTE: Currently, WebGL backend is is still experimental, so expect bugs.

```sh
# 0. Install prerequisites
cargo install wasm-bindgen-cli https
# 1. cd to the current folder
# 2. Compile wasm module
cargo build -p integration_wgpu --target wasm32-unknown-unknown
# 3. Invoke wasm-bindgen
wasm-bindgen ../../target/wasm32-unknown-unknown/debug/integration_wgpu.wasm --out-dir . --target web --no-typescript
# 4. run http server
http
# 5. Open 127.0.0.1:8000 in browser
```


[`main`]: src/main.rs
[`wgpu`]: https://github.com/gfx-rs/wgpu
