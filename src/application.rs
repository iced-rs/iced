use crate::window;
use crate::{
    Clipboard, Color, Command, Element, Executor, Settings, Subscription,
};

/// An interactive cross-platform application.
///
/// This trait is the main entrypoint of Iced. Once implemented, you can run
/// your GUI application by simply calling [`run`](#method.run).
///
/// - On native platforms, it will run in its own window.
/// - On the web, it will take control of the `<title>` and the `<body>` of the
///   document.
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
/// [The repository has a bunch of examples] that use the [`Application`] trait:
///
/// - [`clock`], an application that uses the [`Canvas`] widget to draw a clock
/// and its hands to display the current time.
/// - [`download_progress`], a basic application that asynchronously downloads
/// a dummy file of 100 MB and tracks the download progress.
/// - [`events`], a log of native events displayed using a conditional
/// [`Subscription`].
/// - [`game_of_life`], an interactive version of the [Game of Life], invented
/// by [John Horton Conway].
/// - [`pokedex`], an application that displays a random Pokédex entry (sprite
/// included!) by using the [PokéAPI].
/// - [`solar_system`], an animated solar system drawn using the [`Canvas`] widget
/// and showcasing how to compose different transforms.
/// - [`stopwatch`], a watch with start/stop and reset buttons showcasing how
/// to listen to time.
/// - [`todos`], a todos tracker inspired by [TodoMVC].
///
/// [The repository has a bunch of examples]: https://github.com/hecrj/iced/tree/0.2/examples
/// [`clock`]: https://github.com/hecrj/iced/tree/0.2/examples/clock
/// [`download_progress`]: https://github.com/hecrj/iced/tree/0.2/examples/download_progress
/// [`events`]: https://github.com/hecrj/iced/tree/0.2/examples/events
/// [`game_of_life`]: https://github.com/hecrj/iced/tree/0.2/examples/game_of_life
/// [`pokedex`]: https://github.com/hecrj/iced/tree/0.2/examples/pokedex
/// [`solar_system`]: https://github.com/hecrj/iced/tree/0.2/examples/solar_system
/// [`stopwatch`]: https://github.com/hecrj/iced/tree/0.2/examples/stopwatch
/// [`todos`]: https://github.com/hecrj/iced/tree/0.2/examples/todos
/// [`Sandbox`]: crate::Sandbox
/// [`Canvas`]: crate::widget::Canvas
/// [PokéAPI]: https://pokeapi.co/
/// [TodoMVC]: http://todomvc.com/
///
/// ## A simple "Hello, world!"
///
/// If you just want to get started, here is a simple [`Application`] that
/// says "Hello, world!":
///
/// ```no_run
/// use iced::{executor, Application, Clipboard, Command, Element, Settings, Text};
///
/// pub fn main() -> iced::Result {
///     Hello::run(Settings::default())
/// }
///
/// struct Hello;
///
/// impl Application for Hello {
///     type Executor = executor::Default;
///     type Message = ();
///     type Flags = ();
///
///     fn new(_flags: ()) -> (Hello, Command<Self::Message>) {
///         (Hello, Command::none())
///     }
///
///     fn title(&self) -> String {
///         String::from("A cool application")
///     }
///
///     fn update(&mut self, _message: Self::Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
///         Command::none()
///     }
///
///     fn view(&mut self) -> Element<Self::Message> {
///         Text::new("Hello, world!").into()
///     }
/// }
/// ```
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
    fn update(
        &mut self,
        message: Self::Message,
        clipboard: &mut Clipboard,
    ) -> Command<Self::Message>;

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
    fn view(&mut self) -> Element<'_, Self::Message>;

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
    /// and __will NOT return__ unless there is an [`Error`] during startup.
    ///
    /// It should probably be that last thing you call in your `main` function.
    ///
    /// [`Error`]: crate::Error
    fn run(settings: Settings<Self::Flags>) -> crate::Result
    where
        Self: 'static,
    {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let renderer_settings = crate::renderer::Settings {
                default_font: settings.default_font,
                default_text_size: settings.default_text_size,
                antialiasing: if settings.antialiasing {
                    Some(crate::renderer::settings::Antialiasing::MSAAx4)
                } else {
                    None
                },
                ..crate::renderer::Settings::from_env()
            };

            Ok(crate::runtime::application::run::<
                Instance<Self>,
                Self::Executor,
                crate::renderer::window::Compositor,
            >(settings.into(), renderer_settings)?)
        }

        #[cfg(target_arch = "wasm32")]
        {
            <Instance<Self> as iced_web::Application>::run(settings.flags);

            Ok(())
        }
    }
}

struct Instance<A: Application>(A);

#[cfg(not(target_arch = "wasm32"))]
impl<A> iced_winit::Program for Instance<A>
where
    A: Application,
{
    type Renderer = crate::renderer::Renderer;
    type Message = A::Message;
    type Clipboard = iced_winit::Clipboard;

    fn update(
        &mut self,
        message: Self::Message,
        clipboard: &mut iced_winit::Clipboard,
    ) -> Command<Self::Message> {
        self.0.update(message, clipboard)
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        self.0.view()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<A> crate::runtime::Application for Instance<A>
where
    A: Application,
{
    type Flags = A::Flags;

    fn new(flags: Self::Flags) -> (Self, Command<A::Message>) {
        let (app, command) = A::new(flags);

        (Instance(app), command)
    }

    fn title(&self) -> String {
        self.0.title()
    }

    fn mode(&self) -> iced_winit::Mode {
        match self.0.mode() {
            window::Mode::Windowed => iced_winit::Mode::Windowed,
            window::Mode::Fullscreen => iced_winit::Mode::Fullscreen,
            window::Mode::Hidden => iced_winit::Mode::Hidden,
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        self.0.subscription()
    }

    fn background_color(&self) -> Color {
        self.0.background_color()
    }

    fn scale_factor(&self) -> f64 {
        self.0.scale_factor()
    }

    fn should_exit(&self) -> bool {
        self.0.should_exit()
    }
}

#[cfg(target_arch = "wasm32")]
impl<A> iced_web::Application for Instance<A>
where
    A: Application,
{
    type Executor = A::Executor;
    type Message = A::Message;
    type Flags = A::Flags;

    fn new(flags: Self::Flags) -> (Self, Command<A::Message>) {
        let (app, command) = A::new(flags);

        (Instance(app), command)
    }

    fn title(&self) -> String {
        self.0.title()
    }

    fn update(
        &mut self,
        message: Self::Message,
        clipboard: &mut Clipboard,
    ) -> Command<Self::Message> {
        self.0.update(message, clipboard)
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        self.0.subscription()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        self.0.view()
    }
}
