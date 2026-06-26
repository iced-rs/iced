# Virtual Keyboard

Demonstrates virtual keyboard support for iced applications running in the browser via WebAssembly.

On mobile browsers the OS keyboard only appears when a native `<input>` element is focused. iced renders into a `<canvas>`, so this example relies on the `TextAgent` — a hidden `<input>` managed by `iced_winit` — to bridge keyboard and IME events back into iced's event loop. No changes to your application code are needed; the virtual keyboard appears automatically whenever a [`text_input`] widget is focused.

[`text_input`]: https://docs.rs/iced/latest/iced/widget/fn.text_input.html

## Running

### Native

```sh
cargo run --package virtual_keyboard
```

### Web

Install [trunk] once:

```sh
cargo install trunk
```

Then serve the example:

```sh
cd examples/virtual_keyboard
trunk serve --open
```

For mobile testing on the same network:

```sh
trunk serve --address 0.0.0.0
# Open http://<your-machine-ip>:8080 on the device
```

[trunk]: https://trunkrs.dev

## Testing

```sh
cargo test --package virtual_keyboard
```

## IME support

CJK and other input-method languages work through the standard browser composition event sequence (`compositionstart` → `compositionupdate` → `compositionend`), which is mapped to iced's `InputMethod` events. The preedit overlay is rendered by the `text_input` widget itself, matching the behaviour on native platforms.
