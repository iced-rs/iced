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
use std::{cell::RefCell, rc::Rc};

mod bus;
mod element;
mod hasher;

pub mod style;
pub mod subscription;
pub mod widget;

pub use bus::Bus;
pub use dodrio;
pub use element::Element;
pub use hasher::Hasher;
pub use iced_core::{
    Align, Background, Color, Font, HorizontalAlignment, Length,
    VerticalAlignment,
};
pub use iced_futures::{futures, Command};
pub use style::Style;
pub use subscription::Subscription;
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
        let (app, command) = Self::new();

        let instance = Instance::new(app);
        instance.run(command);
    }
}

struct Instance<Message> {
    title: String,
    ui: Rc<RefCell<Box<dyn Application<Message = Message>>>>,
    vdom: Rc<RefCell<Option<dodrio::VdomWeak>>>,
}

impl<Message> Clone for Instance<Message> {
    fn clone(&self) -> Self {
        Self {
            title: self.title.clone(),
            ui: Rc::clone(&self.ui),
            vdom: Rc::clone(&self.vdom),
        }
    }
}

impl<Message> Instance<Message>
where
    Message: 'static,
{
    fn new(ui: impl Application<Message = Message> + 'static) -> Self {
        Self {
            title: ui.title(),
            ui: Rc::new(RefCell::new(Box::new(ui))),
            vdom: Rc::new(RefCell::new(None)),
        }
    }

    fn update(&mut self, message: Message) {
        let command = self.ui.borrow_mut().update(message);
        let title = self.ui.borrow().title();

        self.spawn(command);

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();

        if self.title != title {
            document.set_title(&title);

            self.title = title;
        }
    }

    fn spawn(&mut self, command: Command<Message>) {
        use futures::FutureExt;

        for future in command.futures() {
            let mut instance = self.clone();

            let future = future.map(move |message| {
                instance.update(message);

                if let Some(ref vdom) = *instance.vdom.borrow() {
                    vdom.schedule_render();
                }
            });

            wasm_bindgen_futures::spawn_local(future);
        }
    }

    fn run(mut self, command: Command<Message>) {
        let window = web_sys::window().unwrap();

        let document = window.document().unwrap();
        document.set_title(&self.title);

        let body = document.body().unwrap();

        let weak = self.vdom.clone();
        self.spawn(command);

        let vdom = dodrio::Vdom::new(&body, self);
        *weak.borrow_mut() = Some(vdom.weak());

        vdom.forget();
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
