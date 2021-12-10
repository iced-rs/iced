<div align="center">

<img src="docs/logo.svg" width="140px" />

# iced

[![Documentation](https://docs.rs/iced/badge.svg)][documentation]
[![Crates.io](https://img.shields.io/crates/v/iced.svg)](https://crates.io/crates/iced)
[![License](https://img.shields.io/crates/l/iced.svg)](https://github.com/hecrj/iced/blob/master/LICENSE)
[![Downloads](https://img.shields.io/crates/d/iced.svg)](https://crates.io/crates/iced)
[![Test Status](https://github.com/hecrj/iced/workflows/Test/badge.svg?event=push)](https://github.com/hecrj/iced/actions)
[![Discord Server](https://img.shields.io/discord/628993209984614400?label=&labelColor=6A7EC2&logo=discord&logoColor=ffffff&color=7389D8)](https://discord.gg/3xZJ65GAhd)

A cross-platform GUI library for Rust focused on simplicity and type-safety.
Inspired by [Elm].

<a href="https://gfycat.com/littlesanehalicore">
  <img src="https://thumbs.gfycat.com/LittleSaneHalicore-small.gif" height="350px">
</a>
<a href="https://gfycat.com/politeadorableiberianmole">
  <img src="https://thumbs.gfycat.com/PoliteAdorableIberianmole-small.gif">
</a>

</div>

## Features
  * Simple, easy-to-use, batteries-included API
  * Type-safe, reactive programming model
  * [Cross-platform support] (Windows, macOS, Linux, and [the Web])
  * Responsive layout
  * Built-in widgets (including [text inputs], [scrollables], and more!)
  * Custom widget support (create your own!)
  * [Debug overlay with performance metrics]
  * First-class support for async actions (use futures!)
  * [Modular ecosystem] split into reusable parts:
    * A [renderer-agnostic native runtime] enabling integration with existing systems
    * A [built-in renderer] supporting Vulkan, Metal, DX11, and DX12
    * A [windowing shell]
    * A [web runtime] leveraging the DOM

__iced is currently experimental software.__ [Take a look at the roadmap],
[check out the issues], and [feel free to contribute!]

[Cross-platform support]: https://github.com/hecrj/iced/blob/master/docs/images/todos_desktop.jpg?raw=true
[the Web]: https://iced.rs/
[text inputs]: https://gfycat.com/alertcalmcrow-rust-gui
[scrollables]: https://gfycat.com/perkybaggybaboon-rust-gui
[Debug overlay with performance metrics]: https://gfycat.com/incredibledarlingbee
[Modular ecosystem]: https://github.com/hecrj/iced/blob/master/ECOSYSTEM.md
[renderer-agnostic native runtime]: https://github.com/hecrj/iced/tree/master/native
[`wgpu`]: https://github.com/gfx-rs/wgpu-rs
[built-in renderer]: https://github.com/hecrj/iced/tree/master/wgpu
[windowing shell]: https://github.com/hecrj/iced/tree/master/winit
[`dodrio`]: https://github.com/fitzgen/dodrio
[web runtime]: https://github.com/hecrj/iced/tree/master/web
[Take a look at the roadmap]: https://github.com/hecrj/iced/blob/master/ROADMAP.md
[check out the issues]: https://github.com/hecrj/iced/issues
[feel free to contribute!]: #contributing--feedback

## Installation
Add `iced` as a dependency in your `Cargo.toml`:

```toml
iced = "0.3"
```

__iced moves fast and the `master` branch can contain breaking changes!__ If
you want to learn about a specific release, check out [the release list].

[the release list]: https://github.com/hecrj/iced/releases

## Overview
Inspired by [The Elm Architecture], iced expects you to split user interfaces
into four different concepts:

  * __State__ — the state of your application
  * __Messages__ — user interactions or meaningful events that you care
  about
  * __View logic__ — a way to display your __state__ as widgets that
  may produce __messages__ on user interaction
  * __Update logic__ — a way to react to __messages__ and update your
  __state__

We can build something to see how this works! Let's say we want a simple counter
that can be incremented and decremented using two buttons.

We start by modelling the __state__ of our application:

```rust
use iced::button;

struct Counter {
    // The counter value
    value: i32,

    // The local state of the two buttons
    increment_button: button::State,
    decrement_button: button::State,
}
```

Next, we need to define the possible user interactions of our counter:
the button presses. These interactions are our __messages__:

```rust
#[derive(Debug, Clone, Copy)]
pub enum Message {
    IncrementPressed,
    DecrementPressed,
}
```

Now, let's show the actual counter by putting it all together in our
__view logic__:

```rust
use iced::{Button, Column, Text};

impl Counter {
    pub fn view(&mut self) -> Column<Message> {
        // We use a column: a simple vertical layout
        Column::new()
            .push(
                // The increment button. We tell it to produce an
                // `IncrementPressed` message when pressed
                Button::new(&mut self.increment_button, Text::new("+"))
                    .on_press(Message::IncrementPressed),
            )
            .push(
                // We show the value of the counter here
                Text::new(self.value.to_string()).size(50),
            )
            .push(
                // The decrement button. We tell it to produce a
                // `DecrementPressed` message when pressed
                Button::new(&mut self.decrement_button, Text::new("-"))
                    .on_press(Message::DecrementPressed),
            )
    }
}
```

Finally, we need to be able to react to any produced __messages__ and change our
__state__ accordingly in our __update logic__:

```rust
impl Counter {
    // ...

    pub fn update(&mut self, message: Message) {
        match message {
            Message::IncrementPressed => {
                self.value += 1;
            }
            Message::DecrementPressed => {
                self.value -= 1;
            }
        }
    }
}
```

And that's everything! We just wrote a whole user interface. iced is now able
to:

  1. Take the result of our __view logic__ and layout its widgets.
  1. Process events from our system and produce __messages__ for our
     __update logic__.
  1. Draw the resulting user interface.

Browse the [documentation] and the [examples] to learn more!

## Implementation details
iced was originally born as an attempt at bringing the simplicity of [Elm] and
[The Elm Architecture] into [Coffee], a 2D game engine I am working on.

The core of the library was implemented during May 2019 in [this pull request].
[The first alpha version] was eventually released as
[a renderer-agnostic GUI library]. The library did not provide a renderer and
implemented the current [tour example] on top of [`ggez`], a game library.

Since then, the focus has shifted towards providing a batteries-included,
end-user-oriented GUI library, while keeping [the ecosystem] modular:

<p align="center">
  <a href="https://github.com/hecrj/iced/blob/master/ECOSYSTEM.md">
    <img alt="iced ecosystem" src="docs/graphs/ecosystem.png" width="80%">
  </a>
</p>

[this pull request]: https://github.com/hecrj/coffee/pull/35
[The first alpha version]: https://github.com/hecrj/iced/tree/0.1.0-alpha
[a renderer-agnostic GUI library]: https://www.reddit.com/r/rust/comments/czzjnv/iced_a_rendereragnostic_gui_library_focused_on/
[tour example]: https://github.com/hecrj/iced/blob/master/examples/README.md#tour
[`ggez`]: https://github.com/ggez/ggez
[the ecosystem]: https://github.com/hecrj/iced/blob/master/ECOSYSTEM.md

## Contributing / Feedback
Contributions are greatly appreciated! If you want to contribute, please
read our [contributing guidelines] for more details.

Feedback is also welcome! You can open an issue or, if you want to talk,
come chat to our [Discord server]. Moreover, you can find me (and a bunch of
awesome folks) over the `#games-and-graphics` and `#gui-and-ui` channels in
the [Rust Community Discord]. I go by `lone_scientist#9554` there.

## Sponsors
The development of iced is sponsored by the [Cryptowatch] team at [Kraken.com]

[documentation]: https://docs.rs/iced/
[examples]: https://github.com/hecrj/iced/tree/master/examples
[Coffee]: https://github.com/hecrj/coffee
[Elm]: https://elm-lang.org/
[The Elm Architecture]: https://guide.elm-lang.org/architecture/
[the current issues]: https://github.com/hecrj/iced/issues
[contributing guidelines]: https://github.com/hecrj/iced/blob/master/CONTRIBUTING.md
[Discord server]: https://discord.gg/3xZJ65GAhd
[Rust Community Discord]: https://bit.ly/rust-community
[Cryptowatch]: https://cryptowat.ch/charts
[Kraken.com]: https://kraken.com/
