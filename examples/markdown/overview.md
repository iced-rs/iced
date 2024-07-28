# Overview

Inspired by [The Elm Architecture], Iced expects you to split user interfaces into four different concepts:

* __State__ — the state of your application
* __Messages__ — user interactions or meaningful events that you care about
* __View logic__ — a way to display your __state__ as widgets that may produce __messages__ on user interaction
* __Update logic__ — a way to react to __messages__ and update your __state__

We can build something to see how this works! Let's say we want a simple counter that can be incremented and decremented using two buttons.

We start by modelling the __state__ of our application:

```rust
#[derive(Default)]
struct Counter {
    value: i32,
}
```

Next, we need to define the possible user interactions of our counter: the button presses. These interactions are our __messages__:

```rust
#[derive(Debug, Clone, Copy)]
pub enum Message {
    Increment,
    Decrement,
}
```

Now, let's show the actual counter by putting it all together in our __view logic__:

```rust
use iced::widget::{button, column, text, Column};

impl Counter {
    pub fn view(&self) -> Column<Message> {
        // We use a column: a simple vertical layout
        column![
            // The increment button. We tell it to produce an
            // `Increment` message when pressed
            button("+").on_press(Message::Increment),

            // We show the value of the counter here
            text(self.value).size(50),

            // The decrement button. We tell it to produce a
            // `Decrement` message when pressed
            button("-").on_press(Message::Decrement),
        ]
    }
}
```

Finally, we need to be able to react to any produced __messages__ and change our __state__ accordingly in our __update logic__:

```rust
impl Counter {
    // ...

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Increment => {
                self.value += 1;
            }
            Message::Decrement => {
                self.value -= 1;
            }
        }
    }
}
```

And that's everything! We just wrote a whole user interface. Let's run it:

```rust
fn main() -> iced::Result {
    iced::run("A cool counter", Counter::update, Counter::view)
}
```

Iced will automatically:

  1. Take the result of our __view logic__ and layout its widgets.
  1. Process events from our system and produce __messages__ for our __update logic__.
  1. Draw the resulting user interface.

Read the [book], the [documentation], and the [examples] to learn more!

[book]: https://book.iced.rs/
[documentation]: https://docs.rs/iced/
[examples]: https://github.com/iced-rs/iced/tree/master/examples#examples
[The Elm Architecture]: https://guide.elm-lang.org/architecture/
