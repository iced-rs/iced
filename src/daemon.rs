//! Create and run daemons that run in the background.
use crate::application;
use crate::message;
use crate::program::{self, Program};
use crate::shell;
use crate::theme;
use crate::window;
use crate::{
    Element, Executor, Font, Preset, Result, Settings, Subscription, Task,
    Theme,
};

use iced_debug as debug;

use std::borrow::Cow;

/// Creates an iced [`Daemon`] given its boot, update, and view logic.
///
/// A [`Daemon`] will not open a window by default, but will run silently
/// instead until a [`Task`] from [`window::open`] is returned by its update logic.
///
/// Furthermore, a [`Daemon`] will not stop running when all its windows are closed.
/// In order to completely terminate a [`Daemon`], its process must be interrupted or
/// its update logic must produce a [`Task`] from [`exit`].
///
/// [`exit`]: crate::exit
pub fn daemon<State, Message, Theme, Renderer>(
    boot: impl application::BootFn<State, Message>,
    update: impl application::UpdateFn<State, Message>,
    view: impl for<'a> ViewFn<'a, State, Message, Theme, Renderer>,
) -> Daemon<impl Program<State = State, Message = Message, Theme = Theme>>
where
    State: 'static,
    Message: Send + 'static,
    Theme: theme::Base,
    Renderer: program::Renderer,
{
    use std::marker::PhantomData;

    struct Instance<State, Message, Theme, Renderer, Boot, Update, View> {
        boot: Boot,
        update: Update,
        view: View,
        _state: PhantomData<State>,
        _message: PhantomData<Message>,
        _theme: PhantomData<Theme>,
        _renderer: PhantomData<Renderer>,
    }

    impl<State, Message, Theme, Renderer, Boot, Update, View> Program
        for Instance<State, Message, Theme, Renderer, Boot, Update, View>
    where
        Message: Send + 'static,
        Theme: theme::Base,
        Renderer: program::Renderer,
        Boot: application::BootFn<State, Message>,
        Update: application::UpdateFn<State, Message>,
        View: for<'a> self::ViewFn<'a, State, Message, Theme, Renderer>,
    {
        type State = State;
        type Message = Message;
        type Theme = Theme;
        type Renderer = Renderer;
        type Executor = iced_futures::backend::default::Executor;

        fn name() -> &'static str {
            let name = std::any::type_name::<State>();

            name.split("::").next().unwrap_or("a_cool_daemon")
        }

        fn settings(&self) -> Settings {
            Settings::default()
        }

        fn window(&self) -> Option<iced_core::window::Settings> {
            None
        }

        fn boot(&self) -> (Self::State, Task<Self::Message>) {
            self.boot.boot()
        }

        fn update(
            &self,
            state: &mut Self::State,
            message: Self::Message,
        ) -> Task<Self::Message> {
            self.update.update(state, message)
        }

        fn view<'a>(
            &self,
            state: &'a Self::State,
            window: window::Id,
        ) -> Element<'a, Self::Message, Self::Theme, Self::Renderer> {
            self.view.view(state, window)
        }
    }

    Daemon {
        raw: Instance {
            boot,
            update,
            view,
            _state: PhantomData,
            _message: PhantomData,
            _theme: PhantomData,
            _renderer: PhantomData,
        },
        settings: Settings::default(),
        presets: Vec::new(),
    }
}

/// The underlying definition and configuration of an iced daemon.
///
/// You can use this API to create and run iced applications
/// step by step—without coupling your logic to a trait
/// or a specific type.
///
/// You can create a [`Daemon`] with the [`daemon`] helper.
#[derive(Debug)]
pub struct Daemon<P: Program> {
    raw: P,
    settings: Settings,
    presets: Vec<Preset<P::State, P::Message>>,
}

impl<P: Program> Daemon<P> {
    /// Runs the [`Daemon`].
    pub fn run(self) -> Result
    where
        Self: 'static,
        P::Message: message::MaybeDebug + message::MaybeClone,
    {
        #[cfg(feature = "debug")]
        iced_debug::init(iced_debug::Metadata {
            name: P::name(),
            theme: None,
            can_time_travel: cfg!(feature = "time-travel"),
        });

        #[cfg(feature = "tester")]
        let program = iced_tester::attach(self);

        #[cfg(all(feature = "debug", not(feature = "tester")))]
        let program = iced_devtools::attach(self);

        #[cfg(not(any(feature = "tester", feature = "debug")))]
        let program = self;

        Ok(shell::run(program)?)
    }

    /// Sets the [`Settings`] that will be used to run the [`Daemon`].
    pub fn settings(self, settings: Settings) -> Self {
        Self { settings, ..self }
    }

    /// Sets the [`Settings::antialiasing`] of the [`Daemon`].
    pub fn antialiasing(self, antialiasing: bool) -> Self {
        Self {
            settings: Settings {
                antialiasing,
                ..self.settings
            },
            ..self
        }
    }

    /// Sets the default [`Font`] of the [`Daemon`].
    pub fn default_font(self, default_font: Font) -> Self {
        Self {
            settings: Settings {
                default_font,
                ..self.settings
            },
            ..self
        }
    }

    /// Adds a font to the list of fonts that will be loaded at the start of the [`Daemon`].
    pub fn font(mut self, font: impl Into<Cow<'static, [u8]>>) -> Self {
        self.settings.fonts.push(font.into());
        self
    }

    /// Sets the title of the [`Daemon`].
    pub fn title(
        self,
        title: impl TitleFn<P::State>,
    ) -> Daemon<
        impl Program<State = P::State, Message = P::Message, Theme = P::Theme>,
    > {
        Daemon {
            raw: program::with_title(self.raw, move |state, window| {
                title.title(state, window)
            }),
            settings: self.settings,
            presets: self.presets,
        }
    }

    /// Sets the subscription logic of the [`Daemon`].
    pub fn subscription(
        self,
        f: impl Fn(&P::State) -> Subscription<P::Message>,
    ) -> Daemon<
        impl Program<State = P::State, Message = P::Message, Theme = P::Theme>,
    > {
        Daemon {
            raw: program::with_subscription(self.raw, f),
            settings: self.settings,
            presets: self.presets,
        }
    }

    /// Sets the theme logic of the [`Daemon`].
    pub fn theme(
        self,
        f: impl ThemeFn<P::State, P::Theme>,
    ) -> Daemon<
        impl Program<State = P::State, Message = P::Message, Theme = P::Theme>,
    > {
        Daemon {
            raw: program::with_theme(self.raw, move |state, window| {
                f.theme(state, window)
            }),
            settings: self.settings,
            presets: self.presets,
        }
    }

    /// Sets the style logic of the [`Daemon`].
    pub fn style(
        self,
        f: impl Fn(&P::State, &P::Theme) -> theme::Style,
    ) -> Daemon<
        impl Program<State = P::State, Message = P::Message, Theme = P::Theme>,
    > {
        Daemon {
            raw: program::with_style(self.raw, f),
            settings: self.settings,
            presets: self.presets,
        }
    }

    /// Sets the scale factor of the [`Daemon`].
    pub fn scale_factor(
        self,
        f: impl Fn(&P::State, window::Id) -> f32,
    ) -> Daemon<
        impl Program<State = P::State, Message = P::Message, Theme = P::Theme>,
    > {
        Daemon {
            raw: program::with_scale_factor(self.raw, f),
            settings: self.settings,
            presets: self.presets,
        }
    }

    /// Sets the executor of the [`Daemon`].
    pub fn executor<E>(
        self,
    ) -> Daemon<
        impl Program<State = P::State, Message = P::Message, Theme = P::Theme>,
    >
    where
        E: Executor,
    {
        Daemon {
            raw: program::with_executor::<P, E>(self.raw),
            settings: self.settings,
            presets: self.presets,
        }
    }

    /// Sets the boot presets of the [`Daemon`].
    ///
    /// Presets can be used to override the default booting strategy
    /// of your application during testing to create reproducible
    /// environments.
    pub fn presets(
        self,
        presets: impl IntoIterator<Item = Preset<P::State, P::Message>>,
    ) -> Self {
        Self {
            presets: presets.into_iter().collect(),
            ..self
        }
    }
}

impl<P: Program> Program for Daemon<P> {
    type State = P::State;
    type Message = P::Message;
    type Theme = P::Theme;
    type Renderer = P::Renderer;
    type Executor = P::Executor;

    fn name() -> &'static str {
        P::name()
    }

    fn settings(&self) -> Settings {
        self.settings.clone()
    }

    fn window(&self) -> Option<window::Settings> {
        None
    }

    fn boot(&self) -> (Self::State, Task<Self::Message>) {
        self.raw.boot()
    }

    fn update(
        &self,
        state: &mut Self::State,
        message: Self::Message,
    ) -> Task<Self::Message> {
        debug::hot(|| self.raw.update(state, message))
    }

    fn view<'a>(
        &self,
        state: &'a Self::State,
        window: window::Id,
    ) -> Element<'a, Self::Message, Self::Theme, Self::Renderer> {
        debug::hot(|| self.raw.view(state, window))
    }

    fn title(&self, state: &Self::State, window: window::Id) -> String {
        debug::hot(|| self.raw.title(state, window))
    }

    fn subscription(&self, state: &Self::State) -> Subscription<Self::Message> {
        debug::hot(|| self.raw.subscription(state))
    }

    fn theme(
        &self,
        state: &Self::State,
        window: iced_core::window::Id,
    ) -> Option<Self::Theme> {
        debug::hot(|| self.raw.theme(state, window))
    }

    fn style(&self, state: &Self::State, theme: &Self::Theme) -> theme::Style {
        debug::hot(|| self.raw.style(state, theme))
    }

    fn scale_factor(&self, state: &Self::State, window: window::Id) -> f32 {
        debug::hot(|| self.raw.scale_factor(state, window))
    }

    fn presets(&self) -> &[Preset<Self::State, Self::Message>] {
        &self.presets
    }
}

/// The title logic of some [`Daemon`].
///
/// This trait is implemented both for `&static str` and
/// any closure `Fn(&State, window::Id) -> String`.
///
/// This trait allows the [`daemon`] builder to take any of them.
pub trait TitleFn<State> {
    /// Produces the title of the [`Daemon`].
    fn title(&self, state: &State, window: window::Id) -> String;
}

impl<State> TitleFn<State> for &'static str {
    fn title(&self, _state: &State, _window: window::Id) -> String {
        self.to_string()
    }
}

impl<T, State> TitleFn<State> for T
where
    T: Fn(&State, window::Id) -> String,
{
    fn title(&self, state: &State, window: window::Id) -> String {
        self(state, window)
    }
}

/// The view logic of some [`Daemon`].
///
/// This trait allows the [`daemon`] builder to take any closure that
/// returns any `Into<Element<'_, Message>>`.
pub trait ViewFn<'a, State, Message, Theme, Renderer> {
    /// Produces the widget of the [`Daemon`].
    fn view(
        &self,
        state: &'a State,
        window: window::Id,
    ) -> Element<'a, Message, Theme, Renderer>;
}

impl<'a, T, State, Message, Theme, Renderer, Widget>
    ViewFn<'a, State, Message, Theme, Renderer> for T
where
    T: Fn(&'a State, window::Id) -> Widget,
    State: 'static,
    Widget: Into<Element<'a, Message, Theme, Renderer>>,
{
    fn view(
        &self,
        state: &'a State,
        window: window::Id,
    ) -> Element<'a, Message, Theme, Renderer> {
        self(state, window).into()
    }
}

/// The theme logic of some [`Daemon`].
///
/// Any implementors of this trait can be provided as an argument to
/// [`Daemon::theme`].
///
/// `iced` provides two implementors:
/// - the built-in [`Theme`] itself
/// - and any `Fn(&State, window::Id) -> impl Into<Option<Theme>>`.
pub trait ThemeFn<State, Theme> {
    /// Returns the theme of the [`Daemon`] for the current state and window.
    ///
    /// If `None` is returned, `iced` will try to use a theme that
    /// matches the system color scheme.
    fn theme(&self, state: &State, window: window::Id) -> Option<Theme>;
}

impl<State> ThemeFn<State, Theme> for Theme {
    fn theme(&self, _state: &State, _window: window::Id) -> Option<Theme> {
        Some(self.clone())
    }
}

impl<F, T, State, Theme> ThemeFn<State, Theme> for F
where
    F: Fn(&State, window::Id) -> T,
    T: Into<Option<Theme>>,
{
    fn theme(&self, state: &State, window: window::Id) -> Option<Theme> {
        (self)(state, window).into()
    }
}
