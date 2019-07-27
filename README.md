# Iced

[![Build Status](https://travis-ci.org/hecrj/iced.svg?branch=master)](https://travis-ci.org/hecrj/iced)
[![Documentation](https://docs.rs/iced/badge.svg)](https://docs.rs/iced)
[![Crates.io](https://img.shields.io/crates/v/iced.svg)](https://crates.io/crates/iced)
[![License](https://img.shields.io/crates/l/iced.svg)](https://github.com/hecrj/iced/blob/master/LICENSE)

An opinionated GUI runtime for Rust, heavily inspired by Elm.

## Features
  * Simple, easy to use API
  * Responsive, flexbox-based layouting
  * Type-safe, reactive programming model without weak references
  * Built-in widgets
  * Custom widget support
  * Renderer-agnostic runtime

## Usage
Add `iced` as a dependency in your `Cargo.toml`:

```toml
iced = "0.1"
```

__Iced moves fast and the `master` branch can contain breaking changes!__ If
you want to learn about a specific release, check out [the release list].

[the release list]: https://github.com/hecrj/iced/releases

## Overview
Here is an example showcasing an interactive counter that can be incremented and
decremented using two different buttons:

```rust
use iced::{button, Button, Column, Text};
use crate::MyRenderer;

struct Counter {
    // The counter value
    value: i32,

    // Local state of the two counter buttons
    // This is internal widget state that may change outside our update
    // logic
    increment_button: button::State,
    decrement_button: button::State,
}

// The user interactions we are interested on
#[derive(Debug, Clone, Copy)]
pub enum Message {
    IncrementPressed,
    DecrementPressed,
}

impl Counter {
    // The update logic, called when a message is produced
    fn react(&mut self, message: Message) {
        // We update the counter value after an interaction here
        match message {
            Message::IncrementPressed => {
                self.value += 1;
            }
            Message::DecrementPressed => {
                self.value -= 1;
            }
        }
    }

    // The layout logic, describing the different components of the counter
    fn layout(&mut self, window: &Window) -> Element<Message, MyRenderer> {
        // We use a column so the elements inside are laid out vertically
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
            .into() // We can return a generic `Element` and avoid breaking
                    // changes if we redesign the counter in the future.
    }
}
```
