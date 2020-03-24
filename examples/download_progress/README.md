## Download progress

A basic application that asynchronously downloads a dummy file of 100 MB and tracks the download progress.

The example implements a custom `Subscription` in the __[`download`](src/download.rs)__ module. This subscription downloads and produces messages that can be used to keep track of its progress.

<div align="center">
  <a href="https://gfycat.com/wildearlyafricanwilddog">
    <img src="https://thumbs.gfycat.com/WildEarlyAfricanwilddog-small.gif">
  </a>
</div>

You can run it with `cargo run`:

```
cargo run --package download_progress
```
