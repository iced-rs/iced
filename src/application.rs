use crate::{window, Command, Element, Executor, Settings, Subscription};

/// An interactive cross-platform application.
///
/// This trait is the main entrypoint of Iced. Once implemented, you can run
/// your GUI application by simply calling [`run`](#method.run).
///
/// - On native platforms, it will run in its own window.
/// - On the web, it will take control of the `<title>` and the `<body>` of the
///   document.
///
/// An [`Application`](trait.Application.html) can execute asynchronous actions
/// by returning a [`Command`](struct.Command.html) in some of its methods. If
/// you do not intend to perform any background work in your program, the
/// [`Sandbox`](trait.Sandbox.html) trait offers a simplified interface.
///
/// # Example
/// Let's say we want to run the [`Counter` example we implemented
/// before](index.html#overview). We just need to fill in the gaps:
///
/// ```no_run
/// use iced::{button, executor, Application, Button, Column, Command, Element, Settings, Text};
///
/// pub fn main() {
///     Counter::run(Settings::default())
/// }
///
/// #[derive(Default)]
/// struct Counter {
///     value: i32,
///     increment_button: button::State,
///     decrement_button: button::State,
/// }
///
/// #[derive(Debug, Clone, Copy)]
/// enum Message {
///     IncrementPressed,
///     DecrementPressed,
/// }
///
/// impl Application for Counter {
///     type Executor = executor::Null;
///     type Message = Message;
///
///     fn new() -> (Self, Command<Message>) {
///         (Self::default(), Command::none())
///     }
///
///     fn title(&self) -> String {
///         String::from("A simple counter")
///     }
///
///     fn update(&mut self, message: Message) -> Command<Message> {
///         match message {
///             Message::IncrementPressed => {
///                 self.value += 1;
///             }
///             Message::DecrementPressed => {
///                 self.value -= 1;
///             }
///         }
///
///         Command::none()
///     }
///
///     fn view(&mut self) -> Element<Message> {
///         Column::new()
///             .push(
///                 Button::new(&mut self.increment_button, Text::new("Increment"))
///                     .on_press(Message::IncrementPressed),
///             )
///             .push(
///                 Text::new(self.value.to_string()).size(50),
///             )
///             .push(
///                 Button::new(&mut self.decrement_button, Text::new("Decrement"))
///                     .on_press(Message::DecrementPressed),
///             )
///             .into()
///     }
/// }
/// ```
pub trait Application: Sized {
    /// The [`Executor`] that will run commands and subscriptions.
    ///
    /// The [`executor::Default`] can be a good starting point!
    ///
    /// [`Executor`]: trait.Executor.html
    /// [`executor::Default`]: executor/struct.Default.html
    type Executor: Executor;

    /// The type of __messages__ your [`Application`] will produce.
    ///
    /// [`Application`]: trait.Application.html
    type Message: std::fmt::Debug + Send;

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
    fn new() -> (Self, Command<Self::Message>);

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

    /// Returns the widgets to display in the [`Application`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    ///
    /// [`Application`]: trait.Application.html
    fn view(&mut self) -> Element<'_, Self::Message>;

    /// Returns the current [`Application`] mode.
    ///
    /// The runtime will automatically transition your application if a new mode
    /// is returned.
    ///
    /// Currently, the mode only has an effect in native platforms.
    ///
    /// By default, an application will run in windowed mode.
    ///
    /// [`Application`]: trait.Application.html
    fn mode(&self) -> window::Mode {
        window::Mode::Windowed
    }

    /// Runs the [`Application`].
    ///
    /// This method will take control of the current thread and __will NOT
    /// return__.
    ///
    /// It should probably be that last thing you call in your `main` function.
    ///
    /// [`Application`]: trait.Application.html
    fn run(_settings: Settings)
    where
        Self: 'static,
    {
        #[cfg(not(target_arch = "wasm32"))]
        <Instance<Self> as iced_winit::Application>::run(
            _settings.into(),
            iced_wgpu::Settings {
                default_font: _settings.default_font,
            },
        );

        #[cfg(target_arch = "wasm32")]
        <Instance<Self> as iced_web::Application>::run();
    }
}

struct Instance<A: Application>(A);

#[cfg(not(target_arch = "wasm32"))]
impl<A> iced_winit::Application for Instance<A>
where
    A: Application,
{
    type Backend = iced_wgpu::window::Backend;
    type Executor = A::Executor;
    type Message = A::Message;

    fn new() -> (Self, Command<A::Message>) {
        let (app, command) = A::new();

        (Instance(app), command)
    }

    fn title(&self) -> String {
        self.0.title()
    }

    fn mode(&self) -> iced_winit::Mode {
        match self.0.mode() {
            window::Mode::Windowed => iced_winit::Mode::Windowed,
            window::Mode::Fullscreen => iced_winit::Mode::Fullscreen,
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        self.0.update(message)
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        self.0.subscription()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        self.0.view()
    }
}

#[cfg(target_arch = "wasm32")]
impl<A> iced_web::Application for Instance<A>
where
    A: Application,
{
    type Message = A::Message;
    type Executor = A::Executor;

    fn new() -> (Self, Command<A::Message>) {
        let (app, command) = A::new();

        (Instance(app), command)
    }

    fn title(&self) -> String {
        self.0.title()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        self.0.update(message)
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        self.0.subscription()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        self.0.view()
    }
}
