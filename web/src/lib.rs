//! A web runtime for Iced, targetting the DOM.
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
//! cargo build --package tour --target wasm32-unknown-unknown
//! wasm-bindgen ../target/wasm32-unknown-unknown/debug/tour.wasm --out-dir tour --web
//! ```
//!
//! Then, we need to create an `.html` file to load our application:
//!
//! ```html
//! <!DOCTYPE html>
//! <html>
//!   <head>
//!     <meta http-equiv="Content-type" content="text/html; charset=utf-8"/>
//!     <meta name="viewport" content="width=device-width, initial-scale=1">
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
//! [`tour` example]: https://github.com/hecrj/iced/tree/0.3/examples/tour
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
#![forbid(unsafe_code)]
#![forbid(rust_2018_idioms)]
use dodrio::bumpalo;
use std::{cell::RefCell, rc::Rc};

mod bus;
mod clipboard;
mod element;
mod hasher;

pub mod css;
pub mod subscription;
pub mod widget;

pub use bus::Bus;
pub use clipboard::Clipboard;
pub use css::Css;
pub use dodrio;
pub use element::Element;
pub use hasher::Hasher;
pub use iced_core::{
    keyboard, menu, mouse, Align, Background, Color, Font, HorizontalAlignment,
    Length, Menu, Padding, Point, Rectangle, Size, Vector, VerticalAlignment,
};
pub use iced_futures::{executor, futures, Command};
pub use subscription::Subscription;

#[doc(no_inline)]
pub use widget::*;

#[doc(no_inline)]
pub use executor::Executor;

/// An interactive web application.
///
/// This trait is the main entrypoint of Iced. Once implemented, you can run
/// your GUI application by simply calling [`run`](#method.run). It will take
/// control of the `<title>` and the `<body>` of the   document.
///
/// An [`Application`](trait.Application.html) can execute asynchronous actions
/// by returning a [`Command`](struct.Command.html) in some of its methods.
pub trait Application {
    /// The [`Executor`] that will run commands and subscriptions.
    type Executor: Executor;

    /// The type of __messages__ your [`Application`] will produce.
    type Message: Send;

    /// The data needed to initialize your [`Application`].
    type Flags;

    /// Initializes the [`Application`].
    ///
    /// Here is where you should return the initial state of your app.
    ///
    /// Additionally, you can return a [`Command`] if you need to perform some
    /// async action in the background on startup. This is useful if you want to
    /// load state from a file, perform an initial HTTP request, etc.
    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>)
    where
        Self: Sized;

    /// Returns the current title of the [`Application`].
    ///
    /// This title can be dynamic! The runtime will automatically update the
    /// title of your application when necessary.
    fn title(&self) -> String;

    /// Handles a __message__ and updates the state of the [`Application`].
    ///
    /// This is where you define your __update logic__. All the __messages__,
    /// produced by either user interactions or commands, will be handled by
    /// this method.
    ///
    /// Any [`Command`] returned will be executed immediately in the background.
    fn update(
        &mut self,
        message: Self::Message,
        clipboard: &mut Clipboard,
    ) -> Command<Self::Message>;

    /// Returns the widgets to display in the [`Application`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    fn view(&mut self) -> Element<'_, Self::Message>;

    /// Returns the event [`Subscription`] for the current state of the
    /// application.
    ///
    /// A [`Subscription`] will be kept alive as long as you keep returning it,
    /// and the __messages__ produced will be handled by
    /// [`update`](#tymethod.update).
    ///
    /// By default, this method returns an empty [`Subscription`].
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    /// Runs the [`Application`].
    fn run(flags: Self::Flags)
    where
        Self: 'static + Sized,
    {
        use futures::stream::StreamExt;

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();

        let mut clipboard = Clipboard::new();

        let (sender, receiver) =
            iced_futures::futures::channel::mpsc::unbounded();

        let mut runtime = iced_futures::Runtime::new(
            Self::Executor::new().expect("Create executor"),
            sender.clone(),
        );

        let (app, command) = runtime.enter(|| Self::new(flags));

        let mut title = app.title();
        document.set_title(&title);

        runtime.spawn(command);

        let application = Rc::new(RefCell::new(app));

        let instance = Instance {
            application: application.clone(),
            bus: Bus::new(sender),
        };

        let vdom = dodrio::Vdom::new(&body, instance);

        let event_loop = receiver.for_each(move |message| {
            let (command, subscription) = runtime.enter(|| {
                let command =
                    application.borrow_mut().update(message, &mut clipboard);
                let subscription = application.borrow().subscription();

                (command, subscription)
            });

            let new_title = application.borrow().title();

            runtime.spawn(command);
            runtime.track(subscription);

            if title != new_title {
                document.set_title(&new_title);

                title = new_title;
            }

            vdom.weak().schedule_render();

            futures::future::ready(())
        });

        wasm_bindgen_futures::spawn_local(event_loop);
    }
}

struct Instance<A: Application> {
    application: Rc<RefCell<A>>,
    bus: Bus<A::Message>,
}

impl<'a, A> dodrio::Render<'a> for Instance<A>
where
    A: Application,
{
    fn render(
        &self,
        context: &mut dodrio::RenderContext<'a>,
    ) -> dodrio::Node<'a> {
        use dodrio::builder::*;

        let mut ui = self.application.borrow_mut();
        let element = ui.view();
        let mut css = Css::new();

        let node = element.widget.node(context.bump, &self.bus, &mut css);

        div(context.bump)
            .attr("style", "width: 100%; height: 100%")
            .children(vec![css.node(context.bump), node])
            .finish()
    }
}
