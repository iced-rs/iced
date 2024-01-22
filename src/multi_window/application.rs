use crate::style::application::StyleSheet;
use crate::window;
use crate::{Command, Element, Executor, Settings, Subscription};

/// An interactive cross-platform multi-window application.
///
/// This trait is the main entrypoint of Iced. Once implemented, you can run
/// your GUI application by simply calling [`run`](#method.run).
///
/// - On native platforms, it will run in its own windows.
/// - On the web, it will take control of the `<title>` and the `<body>` of the
///   document and display only the contents of the `window::Id::MAIN` window.
///
/// An [`Application`] can execute asynchronous actions by returning a
/// [`Command`] in some of its methods. If you do not intend to perform any
/// background work in your program, the [`Sandbox`] trait offers a simplified
/// interface.
///
/// When using an [`Application`] with the `debug` feature enabled, a debug view
/// can be toggled by pressing `F12`.
///
/// # Examples
/// See the `examples/multi-window` example to see this multi-window `Application` trait in action.
///
/// ## A simple "Hello, world!"
///
/// If you just want to get started, here is a simple [`Application`] that
/// says "Hello, world!":
///
/// ```no_run
/// use iced::{executor, window};
/// use iced::{Command, Element, Settings, Theme};
/// use iced::multi_window::{self, Application};
///
/// pub fn main() -> iced::Result {
///     Hello::run(Settings::default())
/// }
///
/// struct Hello;
///
/// impl multi_window::Application for Hello {
///     type Executor = executor::Default;
///     type Flags = ();
///     type Message = ();
///     type Theme = Theme;
///
///     fn new(_flags: ()) -> (Hello, Command<Self::Message>) {
///         (Hello, Command::none())
///     }
///
///     fn title(&self, _window: window::Id) -> String {
///         String::from("A cool application")
///     }
///
///     fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
///         Command::none()
///     }
///
///     fn view(&self, _window: window::Id) -> Element<Self::Message> {
///         "Hello, world!".into()
///     }
/// }
/// ```
///
/// [`Sandbox`]: crate::Sandbox
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

    /// The theme of your [`Application`].
    type Theme: Default + StyleSheet;

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

    /// Returns the current title of the `window` of the [`Application`].
    ///
    /// This title can be dynamic! The runtime will automatically update the
    /// title of your window when necessary.
    fn title(&self, window: window::Id) -> String;

    /// Handles a __message__ and updates the state of the [`Application`].
    ///
    /// This is where you define your __update logic__. All the __messages__,
    /// produced by either user interactions or commands, will be handled by
    /// this method.
    ///
    /// Any [`Command`] returned will be executed immediately in the background.
    fn update(&mut self, message: Self::Message) -> Command<Self::Message>;

    /// Returns the widgets to display in the `window` of the [`Application`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    fn view(
        &self,
        window: window::Id,
    ) -> Element<'_, Self::Message, Self::Theme, crate::Renderer>;

    /// Returns the current [`Theme`] of the `window` of the [`Application`].
    ///
    /// [`Theme`]: Self::Theme
    #[allow(unused_variables)]
    fn theme(&self, window: window::Id) -> Self::Theme {
        Self::Theme::default()
    }

    /// Returns the current `Style` of the [`Theme`].
    ///
    /// [`Theme`]: Self::Theme
    fn style(&self) -> <Self::Theme as StyleSheet>::Style {
        <Self::Theme as StyleSheet>::Style::default()
    }

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

    /// Returns the scale factor of the `window` of the [`Application`].
    ///
    /// It can be used to dynamically control the size of the UI at runtime
    /// (i.e. zooming).
    ///
    /// For instance, a scale factor of `2.0` will make widgets twice as big,
    /// while a scale factor of `0.5` will shrink them to half their size.
    ///
    /// By default, it returns `1.0`.
    #[allow(unused_variables)]
    fn scale_factor(&self, window: window::Id) -> f64 {
        1.0
    }

    /// Runs the multi-window [`Application`].
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
        #[allow(clippy::needless_update)]
        let renderer_settings = crate::renderer::Settings {
            default_font: settings.default_font,
            default_text_size: settings.default_text_size,
            antialiasing: if settings.antialiasing {
                Some(crate::graphics::Antialiasing::MSAAx4)
            } else {
                None
            },
            ..crate::renderer::Settings::default()
        };

        Ok(crate::shell::multi_window::run::<
            Instance<Self>,
            Self::Executor,
            crate::renderer::Compositor,
        >(settings.into(), renderer_settings)?)
    }
}

struct Instance<A: Application>(A);

impl<A> crate::runtime::multi_window::Program for Instance<A>
where
    A: Application,
{
    type Message = A::Message;
    type Theme = A::Theme;
    type Renderer = crate::Renderer;

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        self.0.update(message)
    }

    fn view(
        &self,
        window: window::Id,
    ) -> Element<'_, Self::Message, Self::Theme, Self::Renderer> {
        self.0.view(window)
    }
}

impl<A> crate::shell::multi_window::Application for Instance<A>
where
    A: Application,
{
    type Flags = A::Flags;

    fn new(flags: Self::Flags) -> (Self, Command<A::Message>) {
        let (app, command) = A::new(flags);

        (Instance(app), command)
    }

    fn title(&self, window: window::Id) -> String {
        self.0.title(window)
    }

    fn theme(&self, window: window::Id) -> A::Theme {
        self.0.theme(window)
    }

    fn style(&self) -> <A::Theme as StyleSheet>::Style {
        self.0.style()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        self.0.subscription()
    }

    fn scale_factor(&self, window: window::Id) -> f64 {
        self.0.scale_factor(window)
    }
}
