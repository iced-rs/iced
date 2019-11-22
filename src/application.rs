use crate::{Command, Element};

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
/// use iced::{button, Application, Button, Column, Command, Element, Text};
///
/// pub fn main() {
///     Counter::run()
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

    /// Returns the widgets to display in the [`Application`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    ///
    /// [`Application`]: trait.Application.html
    fn view(&mut self) -> Element<'_, Self::Message>;

    /// Runs the [`Application`].
    ///
    /// This method will take control of the current thread and __will NOT
    /// return__.
    ///
    /// It should probably be that last thing you call in your `main` function.
    ///
    /// [`Application`]: trait.Application.html
    fn run()
    where
        Self: 'static,
    {
        #[cfg(not(target_arch = "wasm32"))]
        <Instance<Self> as iced_winit::Application>::run();

        #[cfg(target_arch = "wasm32")]
        iced_web::Application::run(Instance(self));
    }
}

struct Instance<A: Application>(A);

#[cfg(not(target_arch = "wasm32"))]
impl<A> iced_winit::Application for Instance<A>
where
    A: Application,
{
    type Renderer = iced_wgpu::Renderer;
    type Message = A::Message;

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

    fn update(&mut self, message: Self::Message) {
        self.0.update(message);
    }

    fn view(&mut self) -> Element<Self::Message> {
        self.0.view()
    }
}
