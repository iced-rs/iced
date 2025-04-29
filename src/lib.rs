//! iced is a cross-platform GUI library focused on simplicity and type-safety.
//! Inspired by [Elm].
//!
//! [Elm]: https://elm-lang.org/
//!
//! # Disclaimer
//! iced is __experimental__ software. If you expect the documentation to hold your hand
//! as you learn the ropes, you are in for a frustrating experience.
//!
//! The library leverages Rust to its full extent: ownership, borrowing, lifetimes, futures,
//! streams, first-class functions, trait bounds, closures, and more. This documentation
//! is not meant to teach you any of these. Far from it, it will assume you have __mastered__
//! all of them.
//!
//! Furthermore—just like Rust—iced is very unforgiving. It will not let you easily cut corners.
//! The type signatures alone can be used to learn how to use most of the library.
//! Everything is connected.
//!
//! Therefore, iced is easy to learn for __advanced__ Rust programmers; but plenty of patient
//! beginners have learned it and had a good time with it. Since it leverages a lot of what
//! Rust has to offer in a type-safe way, it can be a great way to discover Rust itself.
//!
//! If you don't like the sound of that, you expect to be spoonfed, or you feel frustrated
//! and struggle to use the library; then I recommend you to wait patiently until [the book]
//! is finished.
//!
//! [the book]: https://book.iced.rs
//!
//! # The Pocket Guide
//! Start by calling [`run`]:
//!
//! ```no_run,standalone_crate
//! pub fn main() -> iced::Result {
//!     iced::run(update, view)
//! }
//! # fn update(state: &mut (), message: ()) {}
//! # fn view(state: &()) -> iced::Element<()> { iced::widget::text("").into() }
//! ```
//!
//! Define an `update` function to __change__ your state:
//!
//! ```standalone_crate
//! fn update(counter: &mut u64, message: Message) {
//!     match message {
//!         Message::Increment => *counter += 1,
//!     }
//! }
//! # #[derive(Clone)]
//! # enum Message { Increment }
//! ```
//!
//! Define a `view` function to __display__ your state:
//!
//! ```standalone_crate
//! use iced::widget::{button, text};
//! use iced::Element;
//!
//! fn view(counter: &u64) -> Element<Message> {
//!     button(text(counter)).on_press(Message::Increment).into()
//! }
//! # #[derive(Clone)]
//! # enum Message { Increment }
//! ```
//!
//! And create a `Message` enum to __connect__ `view` and `update` together:
//!
//! ```standalone_crate
//! #[derive(Debug, Clone)]
//! enum Message {
//!     Increment,
//! }
//! ```
//!
//! ## Custom State
//! You can define your own struct for your state:
//!
//! ```standalone_crate
//! #[derive(Default)]
//! struct Counter {
//!     value: u64,
//! }
//! ```
//!
//! But you have to change `update` and `view` accordingly:
//!
//! ```standalone_crate
//! # struct Counter { value: u64 }
//! # #[derive(Clone)]
//! # enum Message { Increment }
//! # use iced::widget::{button, text};
//! # use iced::Element;
//! fn update(counter: &mut Counter, message: Message) {
//!     match message {
//!         Message::Increment => counter.value += 1,
//!     }
//! }
//!
//! fn view(counter: &Counter) -> Element<Message> {
//!     button(text(counter.value)).on_press(Message::Increment).into()
//! }
//! ```
//!
//! ## Widgets and Elements
//! The `view` function must return an [`Element`]. An [`Element`] is just a generic [`widget`].
//!
//! The [`widget`] module contains a bunch of functions to help you build
//! and use widgets.
//!
//! Widgets are configured using the builder pattern:
//!
//! ```standalone_crate
//! # struct Counter { value: u64 }
//! # #[derive(Clone)]
//! # enum Message { Increment }
//! use iced::widget::{button, column, text};
//! use iced::Element;
//!
//! fn view(counter: &Counter) -> Element<Message> {
//!     column![
//!         text(counter.value).size(20),
//!         button("Increment").on_press(Message::Increment),
//!     ]
//!     .spacing(10)
//!     .into()
//! }
//! ```
//!
//! A widget can be turned into an [`Element`] by calling `into`.
//!
//! Widgets and elements are generic over the message type they produce. The
//! [`Element`] returned by `view` must have the same `Message` type as
//! your `update`.
//!
//! ## Layout
//! There is no unified layout system in iced. Instead, each widget implements
//! its own layout strategy.
//!
//! Building your layout will often consist in using a combination of
//! [rows], [columns], and [containers]:
//!
//! ```standalone_crate
//! # struct State;
//! # enum Message {}
//! use iced::widget::{column, container, row};
//! use iced::{Fill, Element};
//!
//! fn view(state: &State) -> Element<Message> {
//!     container(
//!         column![
//!             "Top",
//!             row!["Left", "Right"].spacing(10),
//!             "Bottom"
//!         ]
//!         .spacing(10)
//!     )
//!     .padding(10)
//!     .center_x(Fill)
//!     .center_y(Fill)
//!     .into()
//! }
//! ```
//!
//! Rows and columns lay out their children horizontally and vertically,
//! respectively. [Spacing] can be easily added between elements.
//!
//! Containers position or align a single widget inside their bounds.
//!
//! [rows]: widget::Row
//! [columns]: widget::Column
//! [containers]: widget::Container
//! [Spacing]: widget::Column::spacing
//!
//! ## Sizing
//! The width and height of widgets can generally be defined using a [`Length`].
//!
//! - [`Fill`] will make the widget take all the available space in a given axis.
//! - [`Shrink`] will make the widget use its intrinsic size.
//!
//! Most widgets use a [`Shrink`] sizing strategy by default, but will inherit
//! a [`Fill`] strategy from their children.
//!
//! A fixed numeric [`Length`] in [`Pixels`] can also be used:
//!
//! ```standalone_crate
//! # struct State;
//! # enum Message {}
//! use iced::widget::container;
//! use iced::Element;
//!
//! fn view(state: &State) -> Element<Message> {
//!     container("I am 300px tall!").height(300).into()
//! }
//! ```
//!
//! ## Theming
//! The default [`Theme`] of an application can be changed by defining a `theme`
//! function and leveraging the [`Application`] builder, instead of directly
//! calling [`run`]:
//!
//! ```no_run,standalone_crate
//! # struct State;
//! use iced::Theme;
//!
//! pub fn main() -> iced::Result {
//!     iced::application(new, update, view)
//!         .theme(theme)
//!         .run()
//! }
//!
//! fn new() -> State {
//!     // ...
//!     # State
//! }
//!
//! fn theme(state: &State) -> Theme {
//!     Theme::TokyoNight
//! }
//! # fn update(state: &mut State, message: ()) {}
//! # fn view(state: &State) -> iced::Element<()> { iced::widget::text("").into() }
//! ```
//!
//! The `theme` function takes the current state of the application, allowing the
//! returned [`Theme`] to be completely dynamic—just like `view`.
//!
//! There are a bunch of built-in [`Theme`] variants at your disposal, but you can
//! also [create your own](Theme::custom).
//!
//! ## Styling
//! As with layout, iced does not have a unified styling system. However, all
//! of the built-in widgets follow the same styling approach.
//!
//! The appearance of a widget can be changed by calling its `style` method:
//!
//! ```standalone_crate
//! # struct State;
//! # enum Message {}
//! use iced::widget::container;
//! use iced::Element;
//!
//! fn view(state: &State) -> Element<Message> {
//!     container("I am a rounded box!").style(container::rounded_box).into()
//! }
//! ```
//!
//! The `style` method of a widget takes a closure that, given the current active
//! [`Theme`], returns the widget style:
//!
//! ```standalone_crate
//! # struct State;
//! # #[derive(Clone)]
//! # enum Message {}
//! use iced::widget::button;
//! use iced::{Element, Theme};
//!
//! fn view(state: &State) -> Element<Message> {
//!     button("I am a styled button!").style(|theme: &Theme, status| {
//!         let palette = theme.extended_palette();
//!
//!         match status {
//!             button::Status::Active => {
//!                 button::Style::default()
//!                    .with_background(palette.success.strong.color)
//!             }
//!             _ => button::primary(theme, status),
//!         }
//!     })
//!     .into()
//! }
//! ```
//!
//! Widgets that can be in multiple different states will also provide the closure
//! with some [`Status`], allowing you to use a different style for each state.
//!
//! You can extract the [`Palette`] colors of a [`Theme`] with the [`palette`] or
//! [`extended_palette`] methods.
//!
//! Most widgets provide styling functions for your convenience in their respective modules;
//! like [`container::rounded_box`], [`button::primary`], or [`text::danger`].
//!
//! [`Status`]: widget::button::Status
//! [`palette`]: Theme::palette
//! [`extended_palette`]: Theme::extended_palette
//! [`container::rounded_box`]: widget::container::rounded_box
//! [`button::primary`]: widget::button::primary
//! [`text::danger`]: widget::text::danger
//!
//! ## Concurrent Tasks
//! The `update` function can _optionally_ return a [`Task`].
//!
//! A [`Task`] can be leveraged to perform asynchronous work, like running a
//! future or a stream:
//!
//! ```standalone_crate
//! # #[derive(Clone)]
//! # struct Weather;
//! use iced::Task;
//!
//! struct State {
//!     weather: Option<Weather>,
//! }
//!
//! enum Message {
//!    FetchWeather,
//!    WeatherFetched(Weather),
//! }
//!
//! fn update(state: &mut State, message: Message) -> Task<Message> {
//!     match message {
//!         Message::FetchWeather => Task::perform(
//!             fetch_weather(),
//!             Message::WeatherFetched,
//!         ),
//!         Message::WeatherFetched(weather) => {
//!             state.weather = Some(weather);
//!
//!             Task::none()
//!        }
//!     }
//! }
//!
//! async fn fetch_weather() -> Weather {
//!     // ...
//!     # unimplemented!()
//! }
//! ```
//!
//! Tasks can also be used to interact with the iced runtime. Some modules
//! expose functions that create tasks for different purposes—like [changing
//! window settings](window#functions), [focusing a widget](widget::focus_next), or
//! [querying its visible bounds](widget::container::visible_bounds).
//!
//! Like futures and streams, tasks expose [a monadic interface](Task::then)—but they can also be
//! [mapped](Task::map), [chained](Task::chain), [batched](Task::batch), [canceled](Task::abortable),
//! and more.
//!
//! ## Passive Subscriptions
//! Applications can subscribe to passive sources of data—like time ticks or runtime events.
//!
//! You will need to define a `subscription` function and use the [`Application`] builder:
//!
//! ```no_run,standalone_crate
//! # struct State;
//! use iced::window;
//! use iced::{Size, Subscription};
//!
//! #[derive(Debug, Clone)]
//! enum Message {
//!     WindowResized(Size),
//! }
//!
//! pub fn main() -> iced::Result {
//!     iced::application(new, update, view)
//!         .subscription(subscription)
//!         .run()
//! }
//!
//! fn subscription(state: &State) -> Subscription<Message> {
//!     window::resize_events().map(|(_id, size)| Message::WindowResized(size))
//! }
//! # fn new() -> State { State }
//! # fn update(state: &mut State, message: Message) {}
//! # fn view(state: &State) -> iced::Element<Message> { iced::widget::text("").into() }
//! ```
//!
//! A [`Subscription`] is [a _declarative_ builder of streams](Subscription#the-lifetime-of-a-subscription)
//! that are not allowed to end on their own. Only the `subscription` function
//! dictates the active subscriptions—just like `view` fully dictates the
//! visible widgets of your user interface, at every moment.
//!
//! As with tasks, some modules expose convenient functions that build a [`Subscription`] for you—like
//! [`time::every`] which can be used to listen to time, or [`keyboard::on_key_press`] which will notify you
//! of any key presses. But you can also create your own with [`Subscription::run`] and [`run_with`].
//!
//! [`run_with`]: Subscription::run_with
//!
//! ## Scaling Applications
//! The `update`, `view`, and `Message` triplet composes very nicely.
//!
//! A common pattern is to leverage this composability to split an
//! application into different screens:
//!
//! ```standalone_crate
//! # mod contacts {
//! #     use iced::{Element, Task};
//! #     pub struct Contacts;
//! #     impl Contacts {
//! #         pub fn update(&mut self, message: Message) -> Action { unimplemented!() }
//! #         pub fn view(&self) -> Element<Message> { unimplemented!() }
//! #     }
//! #     #[derive(Debug, Clone)]
//! #     pub enum Message {}
//! #     pub enum Action { None, Run(Task<Message>), Chat(()) }
//! # }
//! # mod conversation {
//! #     use iced::{Element, Task};
//! #     pub struct Conversation;
//! #     impl Conversation {
//! #         pub fn new(contact: ()) -> (Self, Task<Message>) { unimplemented!() }
//! #         pub fn update(&mut self, message: Message) -> Task<Message> { unimplemented!() }
//! #         pub fn view(&self) -> Element<Message> { unimplemented!() }
//! #     }
//! #     #[derive(Debug, Clone)]
//! #     pub enum Message {}
//! # }
//! use contacts::Contacts;
//! use conversation::Conversation;
//!
//! use iced::{Element, Task};
//!
//! struct State {
//!     screen: Screen,
//! }
//!
//! enum Screen {
//!     Contacts(Contacts),
//!     Conversation(Conversation),
//! }
//!
//! enum Message {
//!    Contacts(contacts::Message),
//!    Conversation(conversation::Message)
//! }
//!
//! fn update(state: &mut State, message: Message) -> Task<Message> {
//!     match message {
//!         Message::Contacts(message) => {
//!             if let Screen::Contacts(contacts) = &mut state.screen {
//!                 let action = contacts.update(message);
//!
//!                 match action {
//!                     contacts::Action::None => Task::none(),
//!                     contacts::Action::Run(task) => task.map(Message::Contacts),
//!                     contacts::Action::Chat(contact) => {
//!                         let (conversation, task) = Conversation::new(contact);
//!
//!                         state.screen = Screen::Conversation(conversation);
//!
//!                         task.map(Message::Conversation)
//!                     }
//!                  }
//!             } else {
//!                 Task::none()    
//!             }
//!         }
//!         Message::Conversation(message) => {
//!             if let Screen::Conversation(conversation) = &mut state.screen {
//!                 conversation.update(message).map(Message::Conversation)
//!             } else {
//!                 Task::none()    
//!             }
//!         }
//!     }
//! }
//!
//! fn view(state: &State) -> Element<Message> {
//!     match &state.screen {
//!         Screen::Contacts(contacts) => contacts.view().map(Message::Contacts),
//!         Screen::Conversation(conversation) => conversation.view().map(Message::Conversation),
//!     }
//! }
//! ```
//!
//! The `update` method of a screen can return an `Action` enum that can be leveraged by the parent to
//! execute a task or transition to a completely different screen altogether. The variants of `Action` can
//! have associated data. For instance, in the example above, the `Conversation` screen is created when
//! `Contacts::update` returns an `Action::Chat` with the selected contact.
//!
//! Effectively, this approach lets you "tell a story" to connect different screens together in a type safe
//! way.
//!
//! Furthermore, functor methods like [`Task::map`], [`Element::map`], and [`Subscription::map`] make composition
//! seamless.
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/bdf0430880f5c29443f5f0a0ae4895866dfef4c6/docs/logo.svg"
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(docsrs, feature(doc_cfg))]
use iced_widget::graphics;
use iced_widget::renderer;
use iced_winit as shell;
use iced_winit::core;
use iced_winit::program;
use iced_winit::runtime;

pub use iced_futures::futures;
pub use iced_futures::stream;

#[cfg(not(any(
    target_arch = "wasm32",
    feature = "thread-pool",
    feature = "tokio",
    feature = "smol"
)))]
compile_error!(
    "No futures executor has been enabled! You must enable an \
    executor feature.\n\
    Available options: thread-pool, tokio, or smol."
);

#[cfg(feature = "highlighter")]
pub use iced_highlighter as highlighter;

#[cfg(feature = "wgpu")]
pub use iced_renderer::wgpu::wgpu;

mod error;

pub mod application;
pub mod daemon;
pub mod time;
pub mod window;

#[cfg(feature = "advanced")]
pub mod advanced;

pub use crate::core::alignment;
pub use crate::core::animation;
pub use crate::core::border;
pub use crate::core::color;
pub use crate::core::gradient;
pub use crate::core::padding;
pub use crate::core::theme;
pub use crate::core::{
    Alignment, Animation, Background, Border, Color, ContentFit, Degrees,
    Function, Gradient, Length, Padding, Pixels, Point, Radians, Rectangle,
    Rotation, Settings, Shadow, Size, Theme, Transformation, Vector, never,
};
pub use crate::runtime::exit;
pub use iced_futures::Subscription;

pub use Alignment::Center;
pub use Length::{Fill, FillPortion, Shrink};
pub use alignment::Horizontal::{Left, Right};
pub use alignment::Vertical::{Bottom, Top};

pub mod debug {
    //! Debug your applications.
    pub use iced_debug::{Span, time, time_with};
}

pub mod task {
    //! Create runtime tasks.
    pub use crate::runtime::task::{Handle, Task};

    #[cfg(feature = "sipper")]
    pub use crate::runtime::task::{Never, Sipper, Straw, sipper, stream};
}

pub mod clipboard {
    //! Access the clipboard.
    pub use crate::runtime::clipboard::{
        read, read_primary, write, write_primary,
    };
}

pub mod executor {
    //! Choose your preferred executor to power your application.
    pub use iced_futures::Executor;
    pub use iced_futures::backend::default::Executor as Default;
}

pub mod font {
    //! Load and use fonts.
    pub use crate::core::font::*;
    pub use crate::runtime::font::*;
}

pub mod event {
    //! Handle events of a user interface.
    pub use crate::core::event::{Event, Status};
    pub use iced_futures::event::{
        listen, listen_raw, listen_url, listen_with,
    };
}

pub mod keyboard {
    //! Listen and react to keyboard events.
    pub use crate::core::keyboard::key;
    pub use crate::core::keyboard::{Event, Key, Location, Modifiers};
    pub use iced_futures::keyboard::{on_key_press, on_key_release};
}

pub mod mouse {
    //! Listen and react to mouse events.
    pub use crate::core::mouse::{
        Button, Cursor, Event, Interaction, ScrollDelta,
    };
}

#[cfg(feature = "system")]
pub mod system {
    //! Retrieve system information.
    pub use crate::runtime::system::Information;
    pub use crate::shell::system::*;
}

pub mod overlay {
    //! Display interactive elements on top of other widgets.

    /// A generic overlay.
    ///
    /// This is an alias of an [`overlay::Element`] with a default `Renderer`.
    ///
    /// [`overlay::Element`]: crate::core::overlay::Element
    pub type Element<
        'a,
        Message,
        Theme = crate::Renderer,
        Renderer = crate::Renderer,
    > = crate::core::overlay::Element<'a, Message, Theme, Renderer>;

    pub use iced_widget::overlay::*;
}

pub mod touch {
    //! Listen and react to touch events.
    pub use crate::core::touch::{Event, Finger};
}

#[allow(hidden_glob_reexports)]
pub mod widget {
    //! Use the built-in widgets or create your own.
    pub use iced_widget::*;

    // We hide the re-exported modules by `iced_widget`
    mod core {}
    mod graphics {}
    mod native {}
    mod renderer {}
    mod style {}
    mod runtime {}
}

pub use application::Application;
pub use daemon::Daemon;
pub use error::Error;
pub use event::Event;
pub use executor::Executor;
pub use font::Font;
pub use program::Program;
pub use renderer::Renderer;
pub use task::Task;

#[doc(inline)]
pub use application::application;
#[doc(inline)]
pub use daemon::daemon;

/// A generic widget.
///
/// This is an alias of an `iced_native` element with a default `Renderer`.
pub type Element<
    'a,
    Message,
    Theme = crate::Theme,
    Renderer = crate::Renderer,
> = crate::core::Element<'a, Message, Theme, Renderer>;

/// The result of running an iced program.
pub type Result = std::result::Result<(), Error>;

/// Runs a basic iced application with default [`Settings`] given its title,
/// update, and view logic.
///
/// This is equivalent to chaining [`application()`] with [`Application::run`].
///
/// # Example
/// ```no_run,standalone_crate
/// use iced::widget::{button, column, text, Column};
///
/// pub fn main() -> iced::Result {
///     iced::run(update, view)
/// }
///
/// #[derive(Debug, Clone)]
/// enum Message {
///     Increment,
/// }
///
/// fn update(value: &mut u64, message: Message) {
///     match message {
///         Message::Increment => *value += 1,
///     }
/// }
///
/// fn view(value: &u64) -> Column<Message> {
///     column![
///         text(value),
///         button("+").on_press(Message::Increment),
///     ]
/// }
/// ```
pub fn run<State, Message, Theme, Renderer>(
    update: impl application::Update<State, Message> + 'static,
    view: impl for<'a> application::View<'a, State, Message, Theme, Renderer>
    + 'static,
) -> Result
where
    State: Default + 'static,
    Message: program::Message + 'static,
    Theme: Default + theme::Base + 'static,
    Renderer: program::Renderer + 'static,
{
    application(State::default, update, view).run()
}
