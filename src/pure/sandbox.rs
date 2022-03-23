use crate::pure;
use crate::{Color, Command, Error, Settings, Subscription};

/// A pure version of [`Sandbox`].
///
/// Unlike the impure version, the `view` method of this trait takes an
/// immutable reference to `self` and returns a pure [`Element`].
///
/// [`Sandbox`]: crate::Sandbox
/// [`Element`]: pure::Element
pub trait Sandbox {
    /// The type of __messages__ your [`Sandbox`] will produce.
    type Message: std::fmt::Debug + Send;

    /// Initializes the [`Sandbox`].
    ///
    /// Here is where you should return the initial state of your app.
    fn new() -> Self;

    /// Returns the current title of the [`Sandbox`].
    ///
    /// This title can be dynamic! The runtime will automatically update the
    /// title of your application when necessary.
    fn title(&self) -> String;

    /// Handles a __message__ and updates the state of the [`Sandbox`].
    ///
    /// This is where you define your __update logic__. All the __messages__,
    /// produced by user interactions, will be handled by this method.
    fn update(&mut self, message: Self::Message);

    /// Returns the widgets to display in the [`Sandbox`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    fn view(&self) -> pure::Element<'_, Self::Message>;

    /// Returns the background color of the [`Sandbox`].
    ///
    /// By default, it returns [`Color::WHITE`].
    fn background_color(&self) -> Color {
        Color::WHITE
    }

    /// Returns the scale factor of the [`Sandbox`].
    ///
    /// It can be used to dynamically control the size of the UI at runtime
    /// (i.e. zooming).
    ///
    /// For instance, a scale factor of `2.0` will make widgets twice as big,
    /// while a scale factor of `0.5` will shrink them to half their size.
    ///
    /// By default, it returns `1.0`.
    fn scale_factor(&self) -> f64 {
        1.0
    }

    /// Returns whether the [`Sandbox`] should be terminated.
    ///
    /// By default, it returns `false`.
    fn should_exit(&self) -> bool {
        false
    }

    /// Runs the [`Sandbox`].
    ///
    /// On native platforms, this method will take control of the current thread
    /// and __will NOT return__.
    ///
    /// It should probably be that last thing you call in your `main` function.
    fn run(settings: Settings<()>) -> Result<(), Error>
    where
        Self: 'static + Sized,
    {
        <Self as pure::Application>::run(settings)
    }
}

impl<T> pure::Application for T
where
    T: Sandbox,
{
    type Executor = iced_futures::backend::null::Executor;
    type Flags = ();
    type Message = T::Message;

    fn new(_flags: ()) -> (Self, Command<T::Message>) {
        (T::new(), Command::none())
    }

    fn title(&self) -> String {
        T::title(self)
    }

    fn update(&mut self, message: T::Message) -> Command<T::Message> {
        T::update(self, message);

        Command::none()
    }

    fn subscription(&self) -> Subscription<T::Message> {
        Subscription::none()
    }

    fn view(&self) -> pure::Element<'_, T::Message> {
        T::view(self)
    }

    fn background_color(&self) -> Color {
        T::background_color(self)
    }

    fn scale_factor(&self) -> f64 {
        T::scale_factor(self)
    }

    fn should_exit(&self) -> bool {
        T::should_exit(self)
    }
}
