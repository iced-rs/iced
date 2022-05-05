//! Stateless, pure widgets for iced.
//!
//! # The Elm Architecture, purity, and continuity
//! As you may know, applications made with `iced` use [The Elm Architecture].
//!
//! In a nutshell, this architecture defines the initial state of the application, a way to `view` it, and a way to `update` it after a user interaction. The `update` logic is called after a meaningful user interaction, which in turn updates the state of the application. Then, the `view` logic is executed to redisplay the application.
//!
//! Since `view` logic is only run after an `update`, all of the mutations to the application state must only happen in the `update` logic. If the application state changes anywhere else, the `view` logic will not be rerun and, therefore, the previously generated `view` may stay outdated.
//!
//! However, the `Application` trait in `iced` defines `view` as:
//!
//! ```ignore
//! pub trait Application {
//!     fn view(&mut self) -> Element<Self::Message>;
//! }
//! ```
//!
//! As a consequence, the application state can be mutated in `view` logic. The `view` logic in `iced` is __impure__.
//!
//! This impurity is necessary because `iced` puts the burden of widget __continuity__ on its users. In other words, it's up to you to provide `iced` with the internal state of each widget every time `view` is called.
//!
//! If we take a look at the classic `counter` example:
//!
//! ```ignore
//! struct Counter {
//!     value: i32,
//!     increment_button: button::State,
//!     decrement_button: button::State,
//! }
//!
//! // ...
//!
//! impl Counter {
//!     pub fn view(&mut self) -> Column<Message> {
//!         Column::new()
//!             .push(
//!                 Button::new(&mut self.increment_button, Text::new("+"))
//!                     .on_press(Message::IncrementPressed),
//!             )
//!             .push(Text::new(self.value.to_string()).size(50))
//!             .push(
//!                 Button::new(&mut self.decrement_button, Text::new("-"))
//!                     .on_press(Message::DecrementPressed),
//!             )
//!     }
//! }
//! ```
//!
//! We can see how we need to keep track of the `button::State` of each `Button` in our `Counter` state and provide a mutable reference to the widgets in our `view` logic. The widgets produced by `view` are __stateful__.
//!
//! While this approach forces users to keep track of widget state and causes impurity, I originally chose it because it allows `iced` to directly consume the widget tree produced by `view`. Since there is no internal state decoupled from `view` maintained by the runtime, `iced` does not need to compare (e.g. reconciliate) widget trees in order to ensure continuity.
//!
//! # Stateless widgets
//! As the library matures, the need for some kind of persistent widget data (see #553) between `view` calls becomes more apparent (e.g. incremental rendering, animations, accessibility, etc.).
//!
//! If we are going to end up having persistent widget data anyways... There is no reason to have impure, stateful widgets anymore!
//!
//! And so I started exploring and ended up creating a new subcrate called `iced_pure`, which introduces a completely stateless implementation for every widget in `iced`.
//!
//! With the help of this crate, we can now write a pure `counter` example:
//!
//! ```ignore
//! struct Counter {
//!     value: i32,
//! }
//!
//! // ...
//!
//! impl Counter {
//!     fn view(&self) -> Column<Message> {
//!         Column::new()
//!             .push(Button::new("Increment").on_press(Message::IncrementPressed))
//!             .push(Text::new(self.value.to_string()).size(50))
//!             .push(Button::new("Decrement").on_press(Message::DecrementPressed))
//!     }
//! }
//! ```
//!
//! Notice how we no longer need to keep track of the `button::State`! The widgets in `iced_pure` do not take any mutable application state in `view`. They are __stateless__ widgets. As a consequence, we do not need mutable access to `self` in `view` anymore. `view` becomes __pure__.
//!
//! [The Elm Architecture]: https://guide.elm-lang.org/architecture/
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
#![deny(missing_docs)]
#![deny(unused_results)]
#![forbid(unsafe_code)]
#![forbid(rust_2018_idioms)]

pub mod flex;
pub mod helpers;
pub mod overlay;
pub mod widget;

mod element;

pub use element::Element;
pub use helpers::*;
pub use widget::Widget;

use iced_native::event::{self, Event};
use iced_native::layout::{self, Layout};
use iced_native::mouse;
use iced_native::renderer;
use iced_native::{Clipboard, Length, Point, Rectangle, Shell};

/// A bridge between impure and pure widgets.
///
/// If you already have an existing `iced` application, you do not need to switch completely to the new traits in order to benefit from the `pure` module. Instead, you can leverage the new `Pure` widget to include `pure` widgets in your impure `Application`.
///
/// For instance, let's say we want to use our pure `Counter` in an impure application:
///
/// ```ignore
/// use iced_pure::{self, Pure};
///
/// struct Impure {
///     state: pure::State,
///     counter: Counter,
/// }
///
/// impl Sandbox for Impure {
///     // ...
///
///     pub fn view(&mut self) -> Element<Self::Message> {
///         Pure::new(&mut self.state, self.counter.view()).into()
///     }
/// }
/// ```
///
/// [`Pure`] acts as a bridge between pure and impure widgets. It is completely opt-in and can be used to slowly migrate your application to the new architecture.
///
/// The purification of your application may trigger a bunch of important refactors, since it's far easier to keep your data decoupled from the GUI state with stateless widgets. For this reason, I recommend starting small in the most nested views of your application and slowly expand the purity upwards.
pub struct Pure<'a, Message, Renderer> {
    state: &'a mut State,
    element: Element<'a, Message, Renderer>,
}

impl<'a, Message, Renderer> Pure<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced_native::Renderer + 'a,
{
    /// Creates a new [`Pure`] widget with the given [`State`] and impure [`Element`].
    pub fn new(
        state: &'a mut State,
        content: impl Into<Element<'a, Message, Renderer>>,
    ) -> Self {
        let element = content.into();
        let _ = state.diff(&element);

        Self { state, element }
    }
}

/// The internal state of a [`Pure`] widget.
pub struct State {
    state_tree: widget::Tree,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    /// Creates a new [`State`] for a [`Pure`] widget.
    pub fn new() -> Self {
        Self {
            state_tree: widget::Tree::empty(),
        }
    }

    fn diff<Message, Renderer>(
        &mut self,
        new_element: &Element<'_, Message, Renderer>,
    ) {
        self.state_tree.diff(new_element);
    }
}

impl<'a, Message, Renderer> iced_native::Widget<Message, Renderer>
    for Pure<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced_native::Renderer + 'a,
{
    fn width(&self) -> Length {
        self.element.as_widget().width()
    }

    fn height(&self) -> Length {
        self.element.as_widget().height()
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.element.as_widget().layout(renderer, limits)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.element.as_widget_mut().on_event(
            &mut self.state.state_tree,
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            shell,
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        self.element.as_widget().draw(
            &self.state.state_tree,
            renderer,
            style,
            layout,
            cursor_position,
            viewport,
        )
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.element.as_widget().mouse_interaction(
            &self.state.state_tree,
            layout,
            cursor_position,
            viewport,
            renderer,
        )
    }

    fn overlay(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        self.element.as_widget_mut().overlay(
            &mut self.state.state_tree,
            layout,
            renderer,
        )
    }
}

impl<'a, Message, Renderer> Into<iced_native::Element<'a, Message, Renderer>>
    for Pure<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced_native::Renderer + 'a,
{
    fn into(self) -> iced_native::Element<'a, Message, Renderer> {
        iced_native::Element::new(self)
    }
}
