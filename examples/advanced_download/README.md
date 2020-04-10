## Advanced download

A application that asynchronously downloads multiple dummy file of 100 MB and tracks the download progress or cancel the download task respectively.

The example implements a custom `Subscription` in the __[`download`](src/download.rs)__ module. This subscription downloads and produces messages that can be used to keep track of its progress.

You can run it with `cargo run`:

```
cargo run --package advanced_download
```
