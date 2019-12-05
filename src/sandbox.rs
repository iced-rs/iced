use crate::{Application, Command, Element, Settings, Subscription};

/// A sandboxed [`Application`].
///
/// A [`Sandbox`] is just an [`Application`] that cannot run any asynchronous
/// actions.
///
/// If you do not need to leverage a [`Command`], you can use a [`Sandbox`]
/// instead of returning a [`Command::none`] everywhere.
///
/// [`Application`]: trait.Application.html
/// [`Sandbox`]: trait.Sandbox.html
/// [`Command`]: struct.Command.html
/// [`Command::none`]: struct.Command.html#method.none
///
/// # Example
/// We can use a [`Sandbox`] to run the [`Counter` example we implemented
/// before](index.html#overview), instead of an [`Application`]. We just need
/// to remove the use of [`Command`]:
///
/// ```no_run
/// use iced::{button, Button, Column, Element, Sandbox, Settings, Text};
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
/// impl Sandbox for Counter {
///     type Message = Message;
///
///     fn new() -> Self {
///         Self::default()
///     }
///
///     fn title(&self) -> String {
///         String::from("A simple counter")
///     }
///
///     fn update(&mut self, message: Message) {
///         match message {
///             Message::IncrementPressed => {
///                 self.value += 1;
///             }
///             Message::DecrementPressed => {
///                 self.value -= 1;
///             }
///         }
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
pub trait Sandbox {
    /// The type of __messages__ your [`Sandbox`] will produce.
    ///
    /// [`Sandbox`]: trait.Sandbox.html
    type Message: std::fmt::Debug + Send + Clone;

    /// Initializes the [`Sandbox`].
    ///
    /// Here is where you should return the initial state of your app.
    ///
    /// [`Sandbox`]: trait.Sandbox.html
    fn new() -> Self;

    /// Returns the current title of the [`Sandbox`].
    ///
    /// This title can be dynamic! The runtime will automatically update the
    /// title of your application when necessary.
    ///
    /// [`Sandbox`]: trait.Sandbox.html
    fn title(&self) -> String;

    /// Handles a __message__ and updates the state of the [`Sandbox`].
    ///
    /// This is where you define your __update logic__. All the __messages__,
    /// produced by user interactions, will be handled by this method.
    ///
    /// [`Sandbox`]: trait.Sandbox.html
    fn update(&mut self, message: Self::Message);

    /// Returns the widgets to display in the [`Sandbox`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    ///
    /// [`Sandbox`]: trait.Sandbox.html
    fn view(&mut self) -> Element<'_, Self::Message>;

    /// Runs the [`Sandbox`].
    ///
    /// This method will take control of the current thread and __will NOT
    /// return__.
    ///
    /// It should probably be that last thing you call in your `main` function.
    ///
    /// [`Sandbox`]: trait.Sandbox.html
    fn run(settings: Settings)
    where
        Self: 'static + Sized,
    {
        <Self as Application>::run(settings)
    }
}

impl<T> Application for T
where
    T: Sandbox,
{
    type Message = T::Message;

    fn new() -> (Self, Command<T::Message>) {
        (T::new(), Command::none())
    }

    fn title(&self) -> String {
        T::title(self)
    }

    fn update(&mut self, message: T::Message) -> Command<T::Message> {
        T::update(self, message);

        Command::none()
    }

    fn subscriptions(&self) -> Subscription<T::Message> {
        Subscription::none()
    }

    fn view(&mut self) -> Element<'_, T::Message> {
        T::view(self)
    }
}
