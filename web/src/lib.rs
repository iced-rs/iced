//! A web runtime for Iced, targetting the DOM.
//!
//! ![`iced_web` crate graph](https://github.com/hecrj/iced/blob/cae26cb7bc627f4a5b3bcf1cd023a0c552e8c65e/docs/graphs/web.png?raw=true)
//!
//! `iced_web` takes [`iced_core`] and builds a WebAssembly runtime on top. It
//! achieves this by introducing a `Widget` trait that can be used to produce
//! VDOM nodes.
//!
//! The crate is currently a __very experimental__, simple abstraction layer
//! over [`dodrio`].
//!
//! [`iced_core`]: https://github.com/hecrj/iced/tree/master/core
//! [`dodrio`]: https://github.com/fitzgen/dodrio
//!
//! # Usage
//! The current build process is a bit involved, as [`wasm-pack`] does not
//! currently [support building binary crates](https://github.com/rustwasm/wasm-pack/issues/734).
//!
//! Therefore, we instead build using the `wasm32-unknown-unknown` target and
//! use the [`wasm-bindgen`] CLI to generate appropriate bindings.
//!
//! For instance, let's say we want to build the [`tour` example]:
//!
//! ```bash
//! cd examples
//! cargo build --example tour --target wasm32-unknown-unknown
//! wasm-bindgen ../target/wasm32-unknown-unknown/debug/examples/tour.wasm --out-dir tour --web
//! ```
//!
//! Then, we need to create an `.html` file to load our application:
//!
//! ```html
//! <!DOCTYPE html>
//! <html>
//!   <head>
//!     <meta http-equiv="Content-type" content="text/html; charset=utf-8"/>
//!     <title>Tour - Iced</title>
//!   </head>
//!   <body>
//!     <script type="module">
//!       import init from "./tour/tour.js";
//!
//!       init('./tour/tour_bg.wasm');
//!     </script>
//!   </body>
//! </html>
//! ```
//!
//! Finally, we serve it using an HTTP server and access it with our browser.
//!
//! [`wasm-pack`]: https://github.com/rustwasm/wasm-pack
//! [`wasm-bindgen`]: https://github.com/rustwasm/wasm-bindgen
//! [`tour` example]: https://github.com/hecrj/iced/blob/master/examples/tour.rs
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
#![deny(unsafe_code)]
#![deny(rust_2018_idioms)]
use dodrio::bumpalo;
use std::cell::RefCell;

mod bus;
mod element;

pub mod style;
pub mod widget;

pub use bus::Bus;
pub use dodrio;
pub use element::Element;
pub use iced_core::{
    Align, Background, Color, Command, Font, HorizontalAlignment, Length,
    VerticalAlignment,
};
pub use style::Style;
pub use widget::*;

/// An interactive web application.
///
/// This trait is the main entrypoint of Iced. Once implemented, you can run
/// your GUI application by simply calling [`run`](#method.run). It will take
/// control of the `<title>` and the `<body>` of the   document.
///
/// An [`Application`](trait.Application.html) can execute asynchronous actions
/// by returning a [`Command`](struct.Command.html) in some of its methods.
pub trait Application {
    /// The type of __messages__ your [`Application`] will produce.
    ///
    /// [`Application`]: trait.Application.html
    type Message;

    /// Initializes the [`Application`].
    ///
    /// Here is where you should return the initial state of your app.
    ///
    /// Additionally, you can return a [`Command`](struct.Command.html) if you
    /// need to perform some async action in the background on startup. This is
    /// useful if you want to load state from a file, perform an initial HTTP
    /// request, etc.
    ///
    /// [`Application`]: trait.Application.html
    fn new() -> (Self, Command<Self::Message>)
    where
        Self: Sized;

    /// Returns the current title of the [`Application`].
    ///
    /// This title can be dynamic! The runtime will automatically update the
    /// title of your application when necessary.
    ///
    /// [`Application`]: trait.Application.html
    fn title(&self) -> String;

    /// Handles a __message__ and updates the state of the [`Application`].
    ///
    /// This is where you define your __update logic__. All the __messages__,
    /// produced by either user interactions or commands, will be handled by
    /// this method.
    ///
    /// Any [`Command`] returned will be executed immediately in the background.
    ///
    /// [`Application`]: trait.Application.html
    /// [`Command`]: struct.Command.html
    fn update(&mut self, message: Self::Message) -> Command<Self::Message>;

    /// Returns the widgets to display in the [`Application`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    ///
    /// [`Application`]: trait.Application.html
    fn view(&mut self) -> Element<'_, Self::Message>;

    /// Runs the [`Application`].
    ///
    /// [`Application`]: trait.Application.html
    fn run()
    where
        Self: 'static + Sized,
    {
        // TODO: Spawn command
        let (app, _command) = Self::new();

        let instance = Instance::new(app);

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();
        let vdom = dodrio::Vdom::new(&body, instance);

        vdom.forget();
    }
}

struct Instance<Message> {
    ui: RefCell<Box<dyn Application<Message = Message>>>,
}

impl<Message> Instance<Message> {
    fn new(ui: impl Application<Message = Message> + 'static) -> Self {
        Self {
            ui: RefCell::new(Box::new(ui)),
        }
    }

    fn update(&mut self, message: Message) {
        // TODO: Spawn command
        let _command = self.ui.borrow_mut().update(message);
    }
}

impl<Message> dodrio::Render for Instance<Message>
where
    Message: 'static,
{
    fn render<'a, 'bump>(
        &'a self,
        bump: &'bump bumpalo::Bump,
    ) -> dodrio::Node<'bump>
    where
        'a: 'bump,
    {
        use dodrio::builder::*;

        let mut ui = self.ui.borrow_mut();
        let element = ui.view();
        let mut style_sheet = style::Sheet::new();

        let node = element.widget.node(bump, &Bus::new(), &mut style_sheet);

        div(bump)
            .attr("style", "width: 100%; height: 100%")
            .children(vec![style_sheet.node(bump), node])
            .finish()
    }
}
