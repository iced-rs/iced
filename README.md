Iced
&nbsp;
[![Build Status](https://travis-ci.org/hecrj/iced.svg?branch=master)](https://travis-ci.org/hecrj/iced)
[![Documentation](https://docs.rs/iced/badge.svg)](https://docs.rs/iced)
[![Crates.io](https://img.shields.io/crates/v/iced.svg)](https://crates.io/crates/iced)
[![License](https://img.shields.io/crates/l/iced.svg)](https://github.com/hecrj/iced/blob/master/LICENSE)
-------------------

A simple GUI runtime for Rust, heavily inspired by Elm.

__Iced is in a very early stage of development.__ Many [basic features are still
missing] and there are probably _many_ bugs. [Feel free to contribute!]

[basic features are still missing]: https://github.com/hecrj/iced/issues?q=is%3Aissue+is%3Aopen+label%3Afeature
[Feel free to contribute!]: #contributing--feedback

[![UI Tour - Coffee][gui_gif]][gui_gfycat]

[gui_gif]: https://thumbs.gfycat.com/GloomyWeakHammerheadshark-small.gif
[gui_gfycat]: https://gfycat.com/gloomyweakhammerheadshark

## Features
  * Simple, easy-to-use, _macro-free_ API
  * Responsive, flexbox-based layouting
  * Type-safe, reactive programming model
  * Many built-in widgets
  * Custom widget support
  * Renderer-agnostic runtime

## Installation
Add `iced` as a dependency in your `Cargo.toml`:

```toml
iced = "0.1"
```

__Iced moves fast and the `master` branch can contain breaking changes!__ If
you want to learn about a specific release, check out [the release list].

[the release list]: https://github.com/hecrj/iced/releases

## Overview
Iced is heavily inspired by [Elm] and [The Elm Architecture] and it expects you
to split user interfaces into four different concepts:

  * __State__ — the state of your application
  * __Messages__ — user interactions or meaningful events that you care
  about
  * __View logic__ — a way to display your __state__ as widgets that
  may produce __messages__ on user interaction
  * __Update logic__ — a way to react to __messages__ and update your
  __state__

Let's say we want to build an interactive counter that can be incremented and
decremented using two different buttons.

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

Now that we have state, what are the user interactions we care about? The
button presses! These are our __messages__:

```rust
#[derive(Debug, Clone, Copy)]
pub enum Message {
    IncrementPressed,
    DecrementPressed,
}
```

Next, let's put it all together in our __view logic__:

```rust
use iced::{Button, Column, Text};

impl Counter {
    fn view(&mut self) -> Column<Message> {
        // We use a column: a simple vertical layout
        Column::new()
            .push(
                // The increment button. We tell it to produce an
                // `IncrementPressed` message when pressed
                Button::new(&mut self.increment_button, "+")
                    .on_press(Message::IncrementPressed),
            )
            .push(
                // We show the value of the counter here
                Text::new(&self.value.to_string()).size(50),
            )
            .push(
                // The decrement button. We tell it to produce a
                // `DecrementPressed` message when pressed
                Button::new(&mut self.decrement_button, "-")
                    .on_press(Message::DecrementPressed),
            )
    }
}
```

Finally, we need to be able to react to the __messages__ and mutate our
__state__ accordingly in our __update logic__:

```rust
impl Counter {
    // ...

    fn update(&mut self, message: Message) {
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

And that's everything! We just wrote a whole user interface. Iced is now able
to:

  1. Take the result of our __view logic__ and build a user interface.
  1. Process events representing user interactions and produce __messages__ for
     our __update logic__.
  1. Draw the resulting user interface using our own custom __renderer__.

Browse the [documentation] and the [examples] to learn more!

[documentation]: https://docs.rs/iced
[examples]: https://github.com/hecrj/iced/tree/master/examples

## Gallery
[![UI Tour - Coffee][gui_gif]][gui_gfycat]

[gui_gif]: https://thumbs.gfycat.com/GloomyWeakHammerheadshark-small.gif
[gui_gfycat]: https://gfycat.com/gloomyweakhammerheadshark

## Implementation details
Iced was originally born as part of [Coffee], a 2D game engine I am working on,
as an attempt at bringing the simplicity of [Elm] and [The Elm Architecture]
into Rust.

Currently, Iced builds upon
  * [`stretch`] for flexbox-based layouting.
  * [`nalgebra`] for the `Point` type.

[`stretch`]: https://github.com/vislyhq/stretch
[`nalgebra`]: https://github.com/rustsim/nalgebra

[Coffee]: https://github.com/hecrj/coffee
[Elm]: https://elm-lang.org/
[The Elm Architecture]: https://guide.elm-lang.org/architecture/

## Contributing / Feedback
If you want to contribute, you are more than welcome to be a part of the
project! Check out the current [issues] if you want to find something to work
on. Try to share you thoughts first! Feel free to open a new issue if you want
to discuss new ideas.

Any kind of feedback is welcome! You can open an issue or, if you want to talk,
you can find me (and a bunch of awesome folks) over the `#gui-and-ui` channel in
the [Rust Community Discord]. I go by `@lone_scientist` there.

[issues]: https://github.com/hecrj/iced/issues
[Rust Community Discord]: https://bit.ly/rust-community
