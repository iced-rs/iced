//! Create and run daemons that run in the background.
use crate::application;
use crate::program::{self, Program};
use crate::shell;
use crate::theme;
use crate::window;
use crate::{Element, Executor, Font, Result, Settings, Subscription, Task};

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
    boot: impl application::Boot<State, Message>,
    update: impl application::Update<State, Message>,
    view: impl for<'a> self::View<'a, State, Message, Theme, Renderer>,
) -> Daemon<impl Program<State = State, Message = Message, Theme = Theme>>
where
    State: 'static,
    Message: program::Message + 'static,
    Theme: Default + theme::Base,
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
        Message: program::Message + 'static,
        Theme: Default + theme::Base,
        Renderer: program::Renderer,
        Boot: application::Boot<State, Message>,
        Update: application::Update<State, Message>,
        View: for<'a> self::View<'a, State, Message, Theme, Renderer>,
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

        fn boot(&self) -> (Self::State, Task<Self::Message>) {
            self.boot.boot()
        }

        fn update(
            &self,
            state: &mut Self::State,
            message: Self::Message,
        ) -> Task<Self::Message> {
            self.update.update(state, message).into()
        }

        fn view<'a>(
            &self,
            state: &'a Self::State,
            window: window::Id,
        ) -> Element<'a, Self::Message, Self::Theme, Self::Renderer> {
            self.view.view(state, window).into()
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
}

impl<P: Program> Daemon<P> {
    /// Runs the [`Daemon`].
    pub fn run(self) -> Result
    where
        Self: 'static,
    {
        #[cfg(all(feature = "debug", not(target_arch = "wasm32")))]
        let program = {
            iced_debug::init(iced_debug::Metadata {
                name: P::name(),
                theme: None,
                can_time_travel: cfg!(feature = "time-travel"),
            });

            iced_devtools::attach(self.raw)
        };

        #[cfg(any(not(feature = "debug"), target_arch = "wasm32"))]
        let program = self.raw;

        Ok(shell::run(program, self.settings, None)?)
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

    /// Sets the [`Title`] of the [`Daemon`].
    pub fn title(
        self,
        title: impl Title<P::State>,
    ) -> Daemon<
        impl Program<State = P::State, Message = P::Message, Theme = P::Theme>,
    > {
        Daemon {
            raw: program::with_title(self.raw, move |state, window| {
                title.title(state, window)
            }),
            settings: self.settings,
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
        }
    }

    /// Sets the theme logic of the [`Daemon`].
    pub fn theme(
        self,
        f: impl Fn(&P::State, window::Id) -> P::Theme,
    ) -> Daemon<
        impl Program<State = P::State, Message = P::Message, Theme = P::Theme>,
    > {
        Daemon {
            raw: program::with_theme(self.raw, f),
            settings: self.settings,
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
        }
    }

    /// Sets the scale factor of the [`Daemon`].
    pub fn scale_factor(
        self,
        f: impl Fn(&P::State, window::Id) -> f64,
    ) -> Daemon<
        impl Program<State = P::State, Message = P::Message, Theme = P::Theme>,
    > {
        Daemon {
            raw: program::with_scale_factor(self.raw, f),
            settings: self.settings,
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
        }
    }
}

/// The title logic of some [`Daemon`].
///
/// This trait is implemented both for `&static str` and
/// any closure `Fn(&State, window::Id) -> String`.
///
/// This trait allows the [`daemon`] builder to take any of them.
pub trait Title<State> {
    /// Produces the title of the [`Daemon`].
    fn title(&self, state: &State, window: window::Id) -> String;
}

impl<State> Title<State> for &'static str {
    fn title(&self, _state: &State, _window: window::Id) -> String {
        self.to_string()
    }
}

impl<T, State> Title<State> for T
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
pub trait View<'a, State, Message, Theme, Renderer> {
    /// Produces the widget of the [`Daemon`].
    fn view(
        &self,
        state: &'a State,
        window: window::Id,
    ) -> impl Into<Element<'a, Message, Theme, Renderer>>;
}

impl<'a, T, State, Message, Theme, Renderer, Widget>
    View<'a, State, Message, Theme, Renderer> for T
where
    T: Fn(&'a State, window::Id) -> Widget,
    State: 'static,
    Widget: Into<Element<'a, Message, Theme, Renderer>>,
{
    fn view(
        &self,
        state: &'a State,
        window: window::Id,
    ) -> impl Into<Element<'a, Message, Theme, Renderer>> {
        self(state, window)
    }
}
