use crate::{Command, Element, Executor, Settings, Subscription};

/// wayland sandbox
pub mod sandbox;
pub use iced_native::application::{Appearance, StyleSheet};
pub use iced_native::command::platform_specific::wayland as actions;
pub use iced_sctk::{application::SurfaceIdWrapper, commands::*, command::*, settings::*};

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

    /// The theme of your [`Application`].
    type Theme: Default + StyleSheet;

    /// The data needed to initialize your [`Application`].
    type Flags: Clone;

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

    /// Returns the current [`Theme`] of the [`Application`].
    ///
    /// [`Theme`]: Self::Theme
    fn theme(&self) -> Self::Theme {
        Self::Theme::default()
    }

    /// Returns the current [`Style`] of the [`Theme`].
    ///
    /// [`Style`]: <Self::Theme as StyleSheet>::Style
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

    /// Returns the widgets to display in the [`Application`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    fn view(
        &self,
        id: SurfaceIdWrapper,
    ) -> Element<'_, Self::Message, crate::Renderer<Self::Theme>>;

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

    /// window was requested to close
    fn close_requested(&self, id: SurfaceIdWrapper) -> Self::Message;

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
        #[allow(clippy::needless_update)]
        let renderer_settings = crate::renderer::Settings {
            default_font: settings.default_font,
            default_text_size: settings.default_text_size,
            text_multithreading: settings.text_multithreading,
            antialiasing: if settings.antialiasing {
                Some(crate::renderer::settings::Antialiasing::MSAAx4)
            } else {
                None
            },
            ..crate::renderer::Settings::from_env()
        };

        Ok(crate::runtime::run::<
            Instance<Self>,
            Self::Executor,
            crate::renderer::window::Compositor<Self::Theme>,
        >(settings.into(), renderer_settings)?)
    }
}

struct Instance<A: Application>(A);

impl<A> crate::runtime::Application for Instance<A>
where
    A: Application,
{
    type Flags = A::Flags;
    type Renderer = crate::Renderer<A::Theme>;
    type Message = A::Message;

    fn new(flags: Self::Flags) -> (Self, Command<A::Message>) {
        let (app, command) = A::new(flags);

        (Instance(app), command)
    }

    fn title(&self) -> String {
        self.0.title()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        self.0.update(message)
    }

    fn view(
        &self,
        id: SurfaceIdWrapper,
    ) -> Element<'_, Self::Message, Self::Renderer> {
        self.0.view(id)
    }

    fn theme(&self) -> A::Theme {
        self.0.theme()
    }

    fn style(&self) -> <A::Theme as StyleSheet>::Style {
        self.0.style()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        self.0.subscription()
    }

    fn scale_factor(&self) -> f64 {
        self.0.scale_factor()
    }

    fn should_exit(&self) -> bool {
        self.0.should_exit()
    }

    fn close_requested(&self, id: SurfaceIdWrapper) -> Self::Message {
        self.0.close_requested(id)
    }
}
