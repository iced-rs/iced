use crate::pure::{self, Pure};
use crate::window;
use crate::{Color, Command, Executor, Settings, Subscription};

/// A pure version of [`Application`].
///
/// Unlike the impure version, the `view` method of this trait takes an
/// immutable reference to `self` and returns a pure [`Element`].
///
/// [`Application`]: crate::Application
/// [`Element`]: pure::Element
pub trait Application: Sized {
    /// The [`Executor`] that will run commands and subscriptions.
    ///
    /// The [default executor] can be a good starting point!
    ///
    /// [`Executor`]: Self::Executor
    /// [default executor]: crate::executor::Default
    type Executor: Executor;

    /// The type of __messages__ your [`Application`] will produce.
    type Message: std::fmt::Debug + Send;

    /// The data needed to initialize your [`Application`].
    type Flags;

    /// Initializes the [`Application`] with the flags provided to
    /// [`run`] as part of the [`Settings`].
    ///
    /// Here is where you should return the initial state of your app.
    ///
    /// Additionally, you can return a [`Command`] if you need to perform some
    /// async action in the background on startup. This is useful if you want to
    /// load state from a file, perform an initial HTTP request, etc.
    ///
    /// [`run`]: Self::run
    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>);

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

    /// Returns the widgets to display in the [`Application`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    fn view(&self) -> pure::Element<'_, Self::Message>;

    /// Returns the current [`Application`] mode.
    ///
    /// The runtime will automatically transition your application if a new mode
    /// is returned.
    ///
    /// Currently, the mode only has an effect in native platforms.
    ///
    /// By default, an application will run in windowed mode.
    fn mode(&self) -> window::Mode {
        window::Mode::Windowed
    }

    /// Returns the background color of the [`Application`].
    ///
    /// By default, it returns [`Color::WHITE`].
    fn background_color(&self) -> Color {
        Color::WHITE
    }

    /// Returns the scale factor of the [`Application`].
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

    /// Returns whether the [`Application`] should be terminated.
    ///
    /// By default, it returns `false`.
    fn should_exit(&self) -> bool {
        false
    }

    /// Runs the [`Application`].
    ///
    /// On native platforms, this method will take control of the current thread
    /// until the [`Application`] exits.
    ///
    /// On the web platform, this method __will NOT return__ unless there is an
    /// [`Error`] during startup.
    ///
    /// [`Error`]: crate::Error
    fn run(settings: Settings<Self::Flags>) -> crate::Result
    where
        Self: 'static,
    {
        <Instance<Self> as crate::Application>::run(settings)
    }
}

struct Instance<A: Application> {
    application: A,
    state: pure::State,
}

impl<A> crate::Application for Instance<A>
where
    A: Application,
    A::Message: 'static,
{
    type Executor = A::Executor;
    type Message = A::Message;
    type Flags = A::Flags;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let (application, command) = A::new(flags);

        (
            Instance {
                application,
                state: pure::State::new(),
            },
            command,
        )
    }

    fn title(&self) -> String {
        A::title(&self.application)
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        A::update(&mut self.application, message)
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        A::subscription(&self.application)
    }

    fn view(&mut self) -> crate::Element<'_, Self::Message> {
        let content = A::view(&self.application);

        Pure::new(&mut self.state, content).into()
    }

    fn mode(&self) -> window::Mode {
        A::mode(&self.application)
    }

    fn background_color(&self) -> Color {
        A::background_color(&self.application)
    }

    fn scale_factor(&self) -> f64 {
        A::scale_factor(&self.application)
    }

    fn should_exit(&self) -> bool {
        A::should_exit(&self.application)
    }
}
