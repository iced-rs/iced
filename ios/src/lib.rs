pub use iced_futures::{executor, futures, Command};

#[doc(no_inline)]
pub use executor::Executor;

pub type Align = ();
pub type Background = ();
pub type Color = ();
pub type Font = ();
pub type HorizontalAlignment = ();
pub type Length = ();
pub type Point = ();
pub type Size = ();

//pub type Subscription<T> = iced_futures::Subscription<Hasher, Event, T>;
pub type Subscription<T> = iced_futures::Subscription<std::collections::hash_map::DefaultHasher, (), T>;
pub type Vector = ();
pub type VerticalAlignment = ();

#[allow(missing_debug_implementations)]
pub struct Element<'a, Message> {
    pub(crate) widget: Box<dyn Widget<Message> + 'a>,
}

pub trait Widget<Message> {
}

pub trait Application: Sized {
    type Executor: Executor;

    /// The type of __messages__ your [`Application`] will produce.
    ///
    /// [`Application`]: trait.Application.html
    type Message: std::fmt::Debug + Send;

    /// The data needed to initialize your [`Application`].
    ///
    /// [`Application`]: trait.Application.html
    type Flags;

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
    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>)
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

    /// Returns the event [`Subscription`] for the current state of the
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

    /// Runs the [`Application`].
    ///
    /// [`Application`]: trait.Application.html
    fn run(flags: Self::Flags)
    where
        Self: 'static + Sized,
    {
        use winit::{
            event::{self, WindowEvent},
            event_loop::{ControlFlow, EventLoop},
            window::WindowBuilder,
        };
        //let event_loop = EventLoop::with_user_event();
        let event_loop = EventLoop::new();
        /*
        let mut runtime = {
            let executor = Self::Executor::new().expect("Create executor");

            Runtime::new(executor, Proxy::new(event_loop.create_proxy()))
        };
        let (mut application, init_command) =
            runtime.enter(|| Self::new(flags));
        runtime.spawn(init_command);
        let mut title = application.title();
        let mut mode = application.mode();
        */

        let window = {
            let mut window_builder = WindowBuilder::new();

            //let (width, height) = settings.window.size;

            window_builder = window_builder
                .with_title("foobar");
                /*
                .with_inner_size(winit::dpi::LogicalSize { width, height })
                .with_resizable(settings.window.resizable)
                .with_decorations(settings.window.decorations);
                */
            window_builder.build(&event_loop).expect("Open window")
        };

        window.request_redraw();

        event_loop.run(move |event, _, control_flow| match event {
            event::Event::MainEventsCleared => {
                window.request_redraw();
            }
            event::Event::UserEvent(message) => {
                //external_messages.push(message);
            }
            event::Event::RedrawRequested(_) => {
            }
            event::Event::WindowEvent {
                event: window_event,
                ..
            } => {
            }
            _ => {
                *control_flow = ControlFlow::Wait;
            }
        })
    }
}
