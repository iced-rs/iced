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
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
#![forbid(unsafe_code)]
#![forbid(rust_2018_idioms)]
use dodrio::bumpalo;
use std::{cell::RefCell, rc::Rc};

mod bus;
mod command;
mod element;
mod hasher;

pub mod css;
pub mod subscription;
pub mod widget;

pub use bus::Bus;
pub use command::Command;
pub use css::Css;
pub use dodrio;
pub use element::Element;
pub use hasher::Hasher;
pub use subscription::Subscription;

pub use iced_core::alignment;
pub use iced_core::keyboard;
pub use iced_core::mouse;
pub use iced_futures::executor;
pub use iced_futures::futures;

pub use iced_core::{
    Alignment, Background, Color, Font, Length, Padding, Point, Rectangle,
    Size, Vector,
};

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
    fn update(&mut self, message: Self::Message) -> Command<Self::Message>;

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

        let (sender, receiver) =
            iced_futures::futures::channel::mpsc::unbounded();

        let mut runtime = iced_futures::Runtime::new(
            Self::Executor::new().expect("Create executor"),
            sender.clone(),
        );

        let (app, command) = runtime.enter(|| Self::new(flags));

        let mut title = app.title();
        document.set_title(&title);

        run_command(command, &mut runtime);

        let application = Rc::new(RefCell::new(app));

        let instance = Instance {
            application: application.clone(),
            bus: Bus::new(sender),
        };

        let vdom = dodrio::Vdom::new(&body, instance);

        let event_loop = receiver.for_each(move |message| {
            let (command, subscription) = runtime.enter(|| {
                let command = application.borrow_mut().update(message);
                let subscription = application.borrow().subscription();

                (command, subscription)
            });

            let new_title = application.borrow().title();

            run_command(command, &mut runtime);
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

/// An interactive embedded web application.
///
/// This trait is the main entrypoint of Iced. Once implemented, you can run
/// your GUI application by simply calling [`run`](#method.run). It will either
/// take control of the `<body>' or of an HTML element of the document specified
/// by `container_id`.
///
/// An [`Embedded`](trait.Embedded.html) can execute asynchronous actions
/// by returning a [`Command`](struct.Command.html) in some of its methods.
pub trait Embedded {
    /// The [`Executor`] that will run commands and subscriptions.
    ///
    /// The [`executor::WasmBindgen`] can be a good choice for the Web.
    ///
    /// [`Executor`]: trait.Executor.html
    /// [`executor::Default`]: executor/struct.Default.html
    type Executor: Executor;

    /// The type of __messages__ your [`Embedded`] application will produce.
    ///
    /// [`Embedded`]: trait.Embedded.html
    type Message: Send;

    /// The data needed to initialize your [`Embedded`] application.
    ///
    /// [`Embedded`]: trait.Embedded.html
    type Flags;

    /// Initializes the [`Embedded`] application.
    ///
    /// Here is where you should return the initial state of your app.
    ///
    /// Additionally, you can return a [`Command`](struct.Command.html) if you
    /// need to perform some async action in the background on startup. This is
    /// useful if you want to load state from a file, perform an initial HTTP
    /// request, etc.
    ///
    /// [`Embedded`]: trait.Embedded.html
    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>)
    where
        Self: Sized;

    /// Handles a __message__ and updates the state of the [`Embedded`]
    /// application.
    ///
    /// This is where you define your __update logic__. All the __messages__,
    /// produced by either user interactions or commands, will be handled by
    /// this method.
    ///
    /// Any [`Command`] returned will be executed immediately in the background.
    ///
    /// [`Embedded`]: trait.Embedded.html
    /// [`Command`]: struct.Command.html
    fn update(&mut self, message: Self::Message) -> Command<Self::Message>;

    /// Returns the widgets to display in the [`Embedded`] application.
    ///
    /// These widgets can produce __messages__ based on user interaction.
    ///
    /// [`Embedded`]: trait.Embedded.html
    fn view(&mut self) -> Element<'_, Self::Message>;

    /// Returns the event [`Subscription`] for the current state of the embedded
    /// application.
    ///
    /// A [`Subscription`] will be kept alive as long as you keep returning it,
    /// and the __messages__ produced will be handled by
    /// [`update`](#tymethod.update).
    ///
    /// By default, this method returns an empty [`Subscription`].
    ///
    /// [`Subscription`]: struct.Subscription.html
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    /// Runs the [`Embedded`] application.
    ///
    /// [`Embedded`]: trait.Embedded.html
    fn run(flags: Self::Flags, container_id: Option<String>)
    where
        Self: 'static + Sized,
    {
        use futures::stream::StreamExt;
        use wasm_bindgen::JsCast;
        use web_sys::HtmlElement;

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let container: HtmlElement = container_id
            .map(|id| document.get_element_by_id(&id).unwrap())
            .map(|container| {
                container.dyn_ref::<HtmlElement>().unwrap().to_owned()
            })
            .unwrap_or_else(|| document.body().unwrap());

        let (sender, receiver) =
            iced_futures::futures::channel::mpsc::unbounded();

        let mut runtime = iced_futures::Runtime::new(
            Self::Executor::new().expect("Create executor"),
            sender.clone(),
        );

        let (app, command) = runtime.enter(|| Self::new(flags));
        run_command(command, &mut runtime);

        let application = Rc::new(RefCell::new(app));

        let instance = EmbeddedInstance {
            application: application.clone(),
            bus: Bus::new(sender),
        };

        let vdom = dodrio::Vdom::new(&container, instance);

        let event_loop = receiver.for_each(move |message| {
            let (command, subscription) = runtime.enter(|| {
                let command = application.borrow_mut().update(message);
                let subscription = application.borrow().subscription();

                (command, subscription)
            });

            run_command(command, &mut runtime);
            runtime.track(subscription);

            vdom.weak().schedule_render();

            futures::future::ready(())
        });

        wasm_bindgen_futures::spawn_local(event_loop);
    }
}

fn run_command<Message: 'static + Send, E: Executor>(
    command: Command<Message>,
    runtime: &mut iced_futures::Runtime<
        Hasher,
        (),
        E,
        iced_futures::futures::channel::mpsc::UnboundedSender<Message>,
        Message,
    >,
) {
    for action in command.actions() {
        match action {
            command::Action::Future(future) => {
                runtime.spawn(future);
            }
        }
    }
}

struct EmbeddedInstance<A: Embedded> {
    application: Rc<RefCell<A>>,
    bus: Bus<A::Message>,
}

impl<'a, A> dodrio::Render<'a> for EmbeddedInstance<A>
where
    A: Embedded,
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
